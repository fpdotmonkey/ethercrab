//! Print SDO Info as YAML about the devices on the network.
//!
//! Without options, this will state the counts of objects for each device
//! and list the index of all available objects for each device.
//!
//! With arguments `device (all|<number>)`, this will give detailed
//! information about each object on either all or a specific device.
//!
//! Run with e.g.
//!
//! Linux
//!
//! ```bash
//! cargo build --release --example sdo-info
//! # avoid sudo with `sudo setcap cap_net_raw=pe /path/to/sdo-info`
//! RUST_LOG=debug sudo -E ./target/release/sdo-info eth0
//! ```
//!
//! Windows
//!
//! ```ps
//! $env:RUST_LOG="debug" ; cargo run --example sdo-info --release -- '\Device\NPF_{FF0ACEE6-E8CD-48D5-A399-619CD2340465}'
//! ```

use std::str::FromStr;

use env_logger::Env;
use ethercrab::{
    error::Error,
    sdo_info::{ObjectDescription, ObjectDescriptionListQuery, ObjectDescriptionListQueryCounts},
    std::ethercat_now,
    MainDevice, MainDeviceConfig, PduStorage, SubDevice, SubDeviceRef, Timeouts,
};
use std::{sync::Arc, time::Duration};

/// Maximum number of SubDevices that can be stored. This must be a power of 2 greater than 1.
const MAX_SUBDEVICES: usize = 16;
/// Maximum PDU data payload size - set this to the max PDI size or higher.
const MAX_PDU_DATA: usize = PduStorage::element_size(1100);
/// Maximum number of EtherCAT frames that can be in flight at any one time.
const MAX_FRAMES: usize = 16;
/// Maximum total PDI length.
const PDI_LEN: usize = 64;

static PDU_STORAGE: PduStorage<MAX_FRAMES, MAX_PDU_DATA> = PduStorage::new();

#[tokio::main]
async fn main() -> Result<(), Error> {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();

    let interface = std::env::args()
        .nth(1)
        .expect("Provide network interface as first argument.");

    let device_selection = if let Some(command) = std::env::args().nth(2)
        && command == "device"
    {
        Some(
            DeviceSelection::from_str(
                &std::env::args()
                    .nth(3)
                    .expect("`all` or a device index number"),
            )
            .expect("`all` or a device index number"),
        )
    } else {
        None
    };

    let (tx, rx, pdu_loop) = PDU_STORAGE.try_split().expect("can only split once");

    let maindevice = Arc::new(MainDevice::new(
        pdu_loop,
        Timeouts {
            wait_loop_delay: Duration::from_millis(2),
            mailbox_response: Duration::from_millis(1000),
            ..Default::default()
        },
        MainDeviceConfig::default(),
    ));

    #[cfg(target_os = "windows")]
    std::thread::spawn(move || {
        ethercrab::std::tx_rx_task_blocking(
            &interface,
            tx,
            rx,
            ethercrab::std::TxRxTaskConfig { spinloop: false },
        )
        .expect("TX/RX task")
    });
    #[cfg(not(target_os = "windows"))]
    tokio::spawn(ethercrab::std::tx_rx_task(&interface, tx, rx).expect("spawn TX/RX task"));

    let group = maindevice
        .init_single_group::<MAX_SUBDEVICES, PDI_LEN>(ethercat_now)
        .await
        .expect("Init");

    for (i, subdevice) in group.iter(&maindevice).enumerate() {
        if let Some(device_selection) = device_selection {
            match device_selection {
                DeviceSelection::All => print_device_object_information(subdevice).await?,
                DeviceSelection::Index(idx) if idx == i => {
                    print_device_object_information(subdevice).await?;
                    break;
                }
                _ => continue,
            }
        } else {
            print_quantities_and_addresses(subdevice).await?;
        }
    }

    let _group = group.into_init(&maindevice).await.expect("PRE-OP -> INIT");

    log::info!("PRE-OP -> INIT, shutdown complete");

    Ok(())
}

async fn print_quantities_and_addresses(
    subdevice: SubDeviceRef<'_, &SubDevice>,
) -> Result<(), Error> {
    println!("{}:", subdevice.name());
    let object_quantities =
        subdevice
            .sdo_info_object_quantities()
            .await?
            .unwrap_or(ObjectDescriptionListQueryCounts {
                all: 0,
                rx_pdo_mappable: 0,
                tx_pdo_mappable: 0,
                stored_for_device_replacement: 0,
                startup_parameters: 0,
            });
    println!(
        r#"  object-quantities:
    all: {}
    rx-pdo-mappable: {}
    tx-pdo-mappable: {}
    stored-for-device-replacement: {}
    startup-parameters: {}"#,
        object_quantities.all,
        object_quantities.rx_pdo_mappable,
        object_quantities.tx_pdo_mappable,
        object_quantities.stored_for_device_replacement,
        object_quantities.startup_parameters,
    );

    let addresses = subdevice
        .sdo_info_object_description_list(ObjectDescriptionListQuery::All)
        .await?
        .unwrap_or_default();
    println!("  addresses: {}", address_list(addresses));
    Ok(())
}

async fn print_device_object_information(
    subdevice: SubDeviceRef<'_, &SubDevice>,
) -> Result<(), Error> {
    println!("{}:", subdevice.name());
    let addresses = subdevice
        .sdo_info_object_description_list(ObjectDescriptionListQuery::All)
        .await?
        .unwrap_or_default();
    for address in addresses {
        println!("    {:#06x}:", address);
        let ObjectDescription {
            data_type,
            max_sub_index,
            object_code,
            name,
        } = match subdevice.sdo_info_object_description(address).await {
            Ok(Some(object_description)) => object_description,
            Ok(None) => continue,
            Err(err) => {
                log::error!(
                    "SDO Info Get Object Description device {:#06x} index {:#06x}: {}",
                    subdevice.configured_address(),
                    address,
                    err
                );
                println!("        error: \"{}\"", err);
                continue;
            }
        };
        println!(
            r#"        data-type: "{}"
        max-sub-index: {}
        object-code: "{}"
        name: "{}""#,
            data_type, max_sub_index, object_code, name
        );
    }
    Ok(())
}

// the string can be as big as 0x1_0000 addresses * 8 char/address.
fn address_list(addresses: heapless::Vec<u16, 0x1_0000>) -> heapless::String<0x8_0000> {
    let mut out = heapless::String::<0x8_0000>::new();
    if addresses.is_empty() {
        out.push_str("[]").unwrap();
        return out;
    }
    out.push('[').expect("longer buffer");
    for address in addresses[..addresses.len() - 1].iter() {
        let _ = out.push_str(&format!("{address:#06x}, "));
    }
    out.push_str(&format!("{:#06x}]", addresses.last().unwrap()))
        .expect("longer buffer");
    out
}
#[derive(Copy, Clone)]
enum DeviceSelection {
    All,
    Index(usize),
}

impl FromStr for DeviceSelection {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "all" {
            return Ok(Self::All);
        }
        Ok(Self::Index(usize::from_str(s).map_err(|_| ())?))
    }
}
