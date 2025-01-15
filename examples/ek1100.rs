//! Demonstrate setting outputs using a Beckhoff EK1100/EK1501 and modules.
//!
//! Run with e.g.
//!
//! Linux
//!
//! ```bash
//! RUST_LOG=debug cargo run --example ek1100 --release -- eth0
//! ```
//!
//! Windows
//!
//! ```ps
//! $env:RUST_LOG="debug" ; cargo run --example ek1100 --release -- '\Device\NPF_{FF0ACEE6-E8CD-48D5-A399-619CD2340465}'
//! ```

use env_logger::Env;
use ethercrab::{
    error::Error,
    std::{ethercat_now, tx_rx_task},
    MainDevice, MainDeviceConfig, PduStorage, Timeouts,
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

    let Some(interface) = std::env::args().nth(1) else {
        println!("Provide network interface as first argument.");
        std::process::exit(1);
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

    match tx_rx_task(&interface, tx, rx) {
	Ok(task) => tokio::spawn(task),
	Err(err) => {
	    println!("{err}");
	    std::process::exit(1);
	}
    };

    let Ok(group) = maindevice
        .init_single_group::<MAX_SUBDEVICES, PDI_LEN>(ethercat_now)
        .await
    else {
        println!("failed to init; EtherCAT bus could be on a different interface, disconnected, or timing out");
        std::process::exit(1);
    };

    let group = group.into_op(&maindevice).await.expect("PRE-OP -> OP");

    for subdevice in group.iter(&maindevice) {
        let io = subdevice.io_raw();

        println!(
            "{:#06x} {} inputs: {} B, outputs: {} B",
            subdevice.configured_address(),
            subdevice.name(),
            io.inputs().len(),
            io.outputs().len()
        );
    }

    let group = group
        .into_safe_op(&maindevice)
        .await
        .expect("OP -> SAFE-OP");

    let group = group
        .into_pre_op(&maindevice)
        .await
        .expect("SAFE-OP -> PRE-OP");

    let _group = group.into_init(&maindevice).await.expect("PRE-OP -> INIT");

    Ok(())
}
