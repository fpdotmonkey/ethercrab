use crate::{
    al_control::AlControl,
    command::Command,
    error::{Error, PduError},
    pdi::PdiOffset,
    pdu_loop::{CheckWorkingCounter, PduLoop, PduResponse},
    register::RegisterAddress,
    slave::Slave,
    slave_group::SlaveGroupContainer,
    slave_state::SlaveState,
    timer_factory::TimerFactory,
    PduData, PduRead, BASE_SLAVE_ADDR,
};
use core::{any::type_name, fmt::Debug};
use core::{cell::RefCell, marker::PhantomData, time::Duration};
use packed_struct::PackedStruct;

pub struct Client<'client, const MAX_FRAMES: usize, const MAX_PDU_DATA: usize, TIMEOUT> {
    // TODO: un-pub
    pub pdu_loop: &'client PduLoop<MAX_FRAMES, MAX_PDU_DATA, TIMEOUT>,
    num_slaves: RefCell<u16>,
    _timeout: PhantomData<TIMEOUT>,
    _pd: PhantomData<&'client ()>,
}

unsafe impl<'client, const MAX_FRAMES: usize, const MAX_PDU_DATA: usize, TIMEOUT> Sync
    for Client<'client, MAX_FRAMES, MAX_PDU_DATA, TIMEOUT>
{
}

impl<'client, const MAX_FRAMES: usize, const MAX_PDU_DATA: usize, TIMEOUT>
    Client<'client, MAX_FRAMES, MAX_PDU_DATA, TIMEOUT>
where
    TIMEOUT: TimerFactory,
{
    pub fn new(pdu_loop: &'client PduLoop<MAX_FRAMES, MAX_PDU_DATA, TIMEOUT>) -> Self {
        // MSRV: Make `MAX_FRAMES` a `u8` when `generic_const_exprs` is stablised
        assert!(
            MAX_FRAMES <= u8::MAX.into(),
            "Packet indexes are u8s, so cache array cannot be any bigger than u8::MAX"
        );

        Self {
            pdu_loop,
            // slaves: UnsafeCell::new(heapless::Vec::new()),
            num_slaves: RefCell::new(0),
            _timeout: PhantomData,
            _pd: PhantomData,
        }
    }

    /// Write zeroes to every slave's memory in chunks of [`MAX_PDU_DATA`].
    async fn blank_memory(&self, start: impl Into<u16>, len: u16) -> Result<(), Error> {
        let start: u16 = start.into();
        let step = MAX_PDU_DATA;
        let range = start..(start + len);

        for chunk_start in range.step_by(step) {
            self.write_service(
                Command::Bwr {
                    address: 0,
                    register: chunk_start,
                },
                [0u8; MAX_PDU_DATA],
            )
            .await?;
        }

        Ok(())
    }

    async fn reset_slaves(&self) -> Result<(), Error> {
        // Reset slaves to init
        self.bwr(
            RegisterAddress::AlControl,
            AlControl::reset().pack().unwrap(),
        )
        .await?;

        // Clear FMMUs. FMMU memory section is 0xff (255) bytes long - see ETG1000.4 Table 57
        self.blank_memory(RegisterAddress::Fmmu0, 0xff).await?;

        // Clear SMs. SM memory section is 0x7f bytes long - see ETG1000.4 Table 59
        self.blank_memory(RegisterAddress::Sm0, 0x7f).await?;

        Ok(())
    }

    /// Detect slaves and set their configured station addresses.
    pub async fn init<G, O>(
        &self,
        mut groups: G,
        mut group_filter: impl FnMut(&mut G, Slave),
    ) -> Result<G, Error>
    where
        G: for<'a> SlaveGroupContainer<'a, MAX_FRAMES, MAX_PDU_DATA, TIMEOUT, O>,
        O: core::future::Future<Output = ()>,
    {
        self.reset_slaves().await?;

        // Each slave increments working counter, so we can use it as a total count of slaves
        let (_res, num_slaves) = self.brd::<u8>(RegisterAddress::Type).await?;

        *self.num_slaves.borrow_mut() = num_slaves;

        // Set configured address for all discovered slaves
        for slave_idx in 0..num_slaves {
            let configured_address = BASE_SLAVE_ADDR + slave_idx;

            self.apwr(
                slave_idx,
                RegisterAddress::ConfiguredStationAddress,
                configured_address,
            )
            .await?
            .wkc(1, "set station address")?;

            let slave = Slave::new(&self, configured_address).await?;

            group_filter(&mut groups, slave);
        }

        let mut offset = PdiOffset::default();

        // Loop through groups and configure the slaves in each one.
        for i in 0..groups.num_groups() {
            // TODO: Better error type for broken group index calculation
            let mut group = groups.group(i).ok_or_else(|| Error::Other)?;

            offset = group.configure_from_eeprom(offset, &self).await?;

            log::debug!("After group #{i} offset: {:?}", offset);
        }

        self.wait_for_state(SlaveState::SafeOp).await?;

        Ok(groups)
    }

    pub fn num_slaves(&self) -> usize {
        usize::from(*self.num_slaves.borrow())
    }

    /// Request the same state for all slaves.
    pub async fn request_slave_state(&self, desired_state: SlaveState) -> Result<(), Error> {
        let num_slaves = *self.num_slaves.borrow();

        self.bwr(
            RegisterAddress::AlControl,
            AlControl::new(desired_state).pack().unwrap(),
        )
        .await?
        .wkc(num_slaves as u16, "set all slaves state")?;

        self.wait_for_state(desired_state).await
    }

    pub async fn wait_for_state(&self, desired_state: SlaveState) -> Result<(), Error> {
        let num_slaves = *self.num_slaves.borrow();

        // TODO: Configurable timeout depending on current -> next states
        crate::timeout::<TIMEOUT, _, _>(Duration::from_millis(5000), async {
            loop {
                let status = self
                    .brd::<AlControl>(RegisterAddress::AlStatus)
                    .await?
                    .wkc(num_slaves as u16, "read all slaves state")?;
                if status.state == desired_state {
                    break Result::<(), Error>::Ok(());
                }

                TIMEOUT::timer(Duration::from_millis(10)).await;
            }
        })
        .await
    }

    // TODO: Dedupe with write_service when refactoring allows
    async fn read_service<T>(&self, command: Command) -> Result<PduResponse<T>, Error>
    where
        T: PduRead,
        <T as PduRead>::Error: Debug,
    {
        let (data, working_counter) = self.pdu_loop.pdu_tx(command, &[], T::len()).await?;

        let res = T::try_from_slice(&data).map_err(|e| {
            log::error!(
                "PDU data decode: {:?}, T: {} data {:?}",
                e,
                type_name::<T>(),
                data
            );

            PduError::Decode
        })?;

        Ok((res, working_counter))
    }

    // TODO: Support different I and O types; some things can return different data
    async fn write_service<T>(&self, command: Command, value: T) -> Result<PduResponse<T>, Error>
    where
        T: PduData,
    {
        let (data, working_counter) = self
            .pdu_loop
            .pdu_tx(command, value.as_slice(), T::len())
            .await?;

        let res = T::try_from_slice(&data).map_err(|_| PduError::Decode)?;

        Ok((res, working_counter))
    }

    pub async fn brd<T>(&self, register: RegisterAddress) -> Result<PduResponse<T>, Error>
    where
        T: PduRead,
        <T as PduRead>::Error: Debug,
    {
        self.read_service(Command::Brd {
            // Address is always zero when sent from master
            address: 0,
            register: register.into(),
        })
        .await
    }

    /// Broadcast write.
    pub async fn bwr<T>(&self, register: RegisterAddress, value: T) -> Result<PduResponse<T>, Error>
    where
        T: PduData,
    {
        self.write_service(
            Command::Bwr {
                address: 0,
                register: register.into(),
            },
            value,
        )
        .await
    }

    /// Auto Increment Physical Read.
    pub async fn aprd<T>(
        &self,
        address: u16,
        register: RegisterAddress,
    ) -> Result<PduResponse<T>, Error>
    where
        T: PduRead,
        <T as PduRead>::Error: Debug,
    {
        self.read_service(Command::Aprd {
            address: 0u16.wrapping_sub(address),
            register: register.into(),
        })
        .await
    }

    /// Auto Increment Physical Write.
    pub async fn apwr<T>(
        &self,
        address: u16,
        register: RegisterAddress,
        value: T,
    ) -> Result<PduResponse<T>, Error>
    where
        T: PduData,
    {
        self.write_service(
            Command::Apwr {
                address: 0u16.wrapping_sub(address),
                register: register.into(),
            },
            value,
        )
        .await
    }

    /// Configured address read.
    pub async fn fprd<T>(
        &self,
        address: u16,
        register: RegisterAddress,
    ) -> Result<PduResponse<T>, Error>
    where
        T: PduRead,
        <T as PduRead>::Error: Debug,
    {
        self.read_service(Command::Fprd {
            address,
            register: register.into(),
        })
        .await
    }

    /// Configured address write.
    pub async fn fpwr<T>(
        &self,
        address: u16,
        register: RegisterAddress,
        value: T,
    ) -> Result<PduResponse<T>, Error>
    where
        T: PduData,
    {
        self.write_service(
            Command::Fpwr {
                address,
                register: register.into(),
            },
            value,
        )
        .await
    }

    /// Logical write.
    pub async fn lwr<T>(&self, address: u32, value: T) -> Result<PduResponse<T>, Error>
    where
        T: PduData,
    {
        self.write_service(Command::Lwr { address }, value).await
    }

    /// Logical read/write.
    pub async fn lrw<T>(&self, address: u32, value: T) -> Result<PduResponse<T>, Error>
    where
        T: PduData,
    {
        self.write_service(Command::Lrw { address }, value).await
    }

    /// Logical read/write, but direct from/to a mutable slice.
    // TODO: Chunked sends if buffer is too long for MAX_PDU_DATA
    pub async fn lrw_buf<'buf>(
        &self,
        address: u32,
        value: &'buf mut [u8],
    ) -> Result<PduResponse<&'buf mut [u8]>, Error> {
        let (data, working_counter) = self
            .pdu_loop
            .pdu_tx(Command::Lrw { address }, value, value.len() as u16)
            .await?;

        if data.len() != value.len() {
            return Err(Error::Pdu(PduError::Decode));
        }

        value.copy_from_slice(&data);

        Ok((value, working_counter))
    }
}
