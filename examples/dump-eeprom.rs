//! Dump the EEPROM of a given sub device to stdout.
//!
//! Requires the unstable `__internals` feature to be enabled.

use std::io::Write;

use embedded_io_async::Read;
use env_logger::Env;
use ethercrab::{
    internals::{ChunkReader, DeviceEeprom},
    std::{ethercat_now, tx_rx_task},
    Client, ClientConfig, PduStorage, Timeouts,
};

/// Maximum number of slaves that can be stored. This must be a power of 2 greater than 1.
const MAX_SLAVES: usize = 16;
/// Maximum PDU data payload size - set this to the max PDI size or higher.
const MAX_PDU_DATA: usize = PduStorage::element_size(1100);
/// Maximum number of EtherCAT frames that can be in flight at any one time.
const MAX_FRAMES: usize = 16;
/// Maximum total PDI length.
const PDI_LEN: usize = 64;

static PDU_STORAGE: PduStorage<MAX_FRAMES, MAX_PDU_DATA> = PduStorage::new();

#[tokio::main]
async fn main() -> Result<(), std::io::Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let exe_name: String = match std::env::current_exe()
        .as_deref()
        .map(std::path::Path::file_name)
    {
        Ok(Some(name)) => name.to_string_lossy().into_owned(),
        _ => "dump-eeprom".into(),
    };
    let usage = format!("Usage: {exe_name} NETWORK_INTERFACE SLAVE_INDEX");

    let interface = match std::env::args().nth(1) {
        Some(interface) => interface,
        None => {
            eprintln!("{usage}");
            return Ok(());
        }
    };

    let index: u16 = match std::env::args().nth(2).as_deref().map(str::parse) {
        Some(Ok(index)) => index,
        _ => {
            eprintln!("{usage}");
            return Ok(());
        }
    };

    let (tx, rx, pdu_loop) = PDU_STORAGE.try_split().expect("can only split once");

    let client = Client::new(
        pdu_loop,
        Timeouts::default(),
        ClientConfig {
            dc_static_sync_iterations: 0,
            ..ClientConfig::default()
        },
    );

    tokio::spawn(tx_rx_task(&interface, tx, rx)?);

    let group = client
        .init_single_group::<MAX_SLAVES, PDI_LEN>(ethercat_now)
        .await
        .map_err(|err| std::io::Error::new(std::io::ErrorKind::Other, err))?;

    if group.len() <= index.into() {
        return Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "index was {index}, but there are only {} slaves",
                group.len()
            ),
        ));
    }

    let slave = group
        .slave(&client, usize::from(index))
        .expect("Could not find device for given index");

    log::info!(
        "Dumping EEPROM for device index {}: {:#06x} {} {}",
        index,
        slave.configured_address(),
        slave.name(),
        slave.identity()
    );

    let base_address = 0x1000;

    let mut len_buf = [0u8; 2];

    // ETG2020 page 7: 0x003e is the EEPROM address size register in kilobit minus 1 (u16).
    ChunkReader::new(DeviceEeprom::new(&client, base_address + index), 0x003e, 2)
        .read_exact(&mut len_buf)
        .await
        .expect("Could not read EEPROM len");

    // Kilobits to bits to bytes, and undoing the offset
    let len = ((u16::from_le_bytes(len_buf) + 1) * 1024) / 8;

    let mut provider = ChunkReader::new(DeviceEeprom::new(&client, base_address + index), 0, len);

    let mut buf = vec![0u8; usize::from(len)];

    provider.read_exact(&mut buf).await.expect("Read");

    std::io::stdout().write_all(&buf[..]).expect("Stdout write");

    Ok(())
}
