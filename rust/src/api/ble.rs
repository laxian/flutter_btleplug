use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use anyhow::Result;
use btleplug::api::{bleuuid::BleUuid, Central, CentralEvent, Manager as _, ScanFilter};
pub use btleplug::platform::Manager;
use futures::stream::StreamExt;
use tokio::{
    sync::{mpsc, Mutex},
    time,
};

pub mod setup;
pub use setup::*;

pub mod device;
pub mod peripheral;

use crate::frb_generated::StreamSink;

pub use device::*;
pub use peripheral::*;

enum Command {
    Scan {
        sink: StreamSink<Vec<BleDevice>>,
        filter: Vec<String>,
    },
    Connect {
        id: String,
    },
    Disconnect {
        id: String,
    },
}

impl std::fmt::Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Scan").finish()
    }
}

static DEVICES: OnceLock<Arc<Mutex<HashMap<String, Peripheral>>>> = OnceLock::new();

static TX: OnceLock<mpsc::UnboundedSender<Command>> = OnceLock::new();

/// Internal send function to send [Command]s into the message loop.
fn send(command: Command) -> Result<()> {
    let tx = TX.get().ok_or(anyhow::anyhow!("TxNotInitialized"))?;
    tx.send(command)?;
    Ok(())
}

/// The init() function must be called before anything else.
/// At the moment the developer has to make sure it is only called once.
pub fn init() -> Result<()> {
    create_runtime()?;
    let runtime = RUNTIME
        .get()
        .ok_or(anyhow::anyhow!("RuntimeNotInitialized"))?;

    let _ = DEVICES.set(Arc::new(Mutex::new(HashMap::new())));

    let (tx, mut rx) = mpsc::unbounded_channel::<Command>();
    TX.set(tx).map_err(|_| anyhow::anyhow!("TxAlreadySet"))?;

    runtime.spawn(async move {
        while let Some(msg) = rx.recv().await {
            match msg {
                Command::Scan { sink, filter } => {
                    tokio::spawn(async { inner_scan(sink, filter).await.unwrap() });
                }
                Command::Connect { id } => inner_connect(id).await.unwrap(),
                Command::Disconnect { id } => inner_disconnect(id).await.unwrap(),
            }
        }
    });
    Ok(())
}

/// Removes all devices that haven't been seen for longer than [timeout] seconds, from the [DEVICES] list.
///
/// # Parameters
/// * timeout - A `u64` representing the maximum age of a device (in seconds)
///   that should be allowed to remain in the list.
///
/// # Example
///
/// Removing all devices that were last seen more than ten seconds ago:
///
/// ```rust
/// # use async_fn;
/// async_fn::remove_stale_devices(10);
/// ```
async fn remove_stale_devices(timeout: u64) {
    let mut devices = DEVICES.get().unwrap().lock().await;
    devices.retain(|_, d| d.is_connected() || d.last_seen.elapsed().as_secs() < timeout);
}

/// Helper function to send all [BleDevice]s to Dart/Flutter.
///
/// # Arguments
///
/// sink: StreamSink<Vec<BleDevice>>
///     The StreamSink to which the Vec<BleDevice> should be sent
///
/// # Return
///
/// Returns false if the stream is closed.
async fn send_devices(sink: &StreamSink<Vec<BleDevice>>) -> Result<()> {
    let devices = DEVICES.get().unwrap().lock().await;
    let mut d = vec![];
    for device in devices.values() {
        let dev = BleDevice::from_peripheral(device).await;
        d.push(dev.clone())
    }
    sink.add(d).map_err(|_| anyhow::anyhow!("No listeners"))?;
    Ok(())
}

/// This function is used to scan for BLE devices and returns the results via the given stream sink.
///
/// Parameters
///
/// sink: StreamSink<Vec<BleDevice>> - A stream sink to which the results are send
///
/// filter: Vec<String> - A vector of strings to filter the results with
pub fn scan(sink: StreamSink<Vec<BleDevice>>, filter: Vec<String>) -> Result<()> {
    send(Command::Scan { sink, filter })
}

async fn inner_scan(sink: StreamSink<Vec<BleDevice>>, _filter: Vec<String>) -> Result<()> {
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let central = adapters.into_iter().next().expect("cannot fail");
    let mut events = central.events().await?;

    // start scanning for devices
    tracing::debug!(
        "{}",
        format!("start scanning on {}", central.adapter_info().await?)
    );
    central.start_scan(ScanFilter::default()).await?;

    let mut device_send_interval = time::interval(time::Duration::from_secs(1));
    loop {
        tokio::select! {
            _ = device_send_interval.tick() => {
                remove_stale_devices(3).await;
                if send_devices(&sink).await.is_err() {
                    break;
                }
            }
            Some(event) = events.next() => {
                // tracing::debug!(format!("{:?}", event));
                match event {
                    CentralEvent::DeviceDiscovered(id) => {
                        tracing::debug!("{}",format!("DeviceDiscovered: {:?}", &id));
                        let peripheral = central.peripheral(&id).await?;
                        let peripheral = Peripheral::new(peripheral);
                        let mut devices = DEVICES.get().unwrap().lock().await;
                        devices.insert(id.to_string(), peripheral);
                    }
                    CentralEvent::DeviceUpdated(id) => {
                        let mut devices = DEVICES.get().unwrap().lock().await;
                        if let Some(device) = devices.get_mut(&id.to_string()) {
                            device.last_seen = time::Instant::now();
                        }
                    }
                    CentralEvent::DeviceConnected(id) => {
                        tracing::debug!("{}",format!("DeviceConnected: {:?}", id));
                        let mut devices = DEVICES.get().unwrap().lock().await;
                        if let Some(device) = devices.get_mut(&id.to_string()) {
                            device.is_connected = true;
                        }
                    }
                    CentralEvent::DeviceDisconnected(id) => {
                        tracing::debug!("{}",format!("DeviceDisconnected: {:?}", id));
                        let mut devices = DEVICES.get().unwrap().lock().await;
                        if let Some(device) = devices.get_mut(&id.to_string()) {
                            device.is_connected = false;
                        }
                    }
                    CentralEvent::ManufacturerDataAdvertisement {
                        id:_,
                        manufacturer_data:_,
                    } => {
                        // tracing::debug!("{}",format!(
                        //     "ManufacturerDataAdvertisement: {:?}, {:?}",
                        //     id, manufacturer_data
                        // ));
                    }
                    CentralEvent::ServiceDataAdvertisement { id:_, service_data:_ } => {
                        // tracing::debug!("{}",format!(
                        //     "ServiceDataAdvertisement: {:?}, {:?}",
                        //     id, service_data
                        // ));
                    }
                    CentralEvent::ServicesAdvertisement { id:_, services } => {
                        let _services: Vec<String> =
                            services.into_iter().map(|s| s.to_short_string()).collect();
                        // tracing::debug!("{}",format!("ServicesAdvertisement: {:?}, {:?}", id, services));
                    }
                }
            }
        }
    }
    // tracing::debug!("Scan finished");
    Ok(())
}

pub fn connect(id: String) -> Result<()> {
    tracing::debug!("{}", format!("Try to connect to: {id}"));
    send(Command::Connect { id })
}

async fn inner_connect(id: String) -> Result<()> {
    tracing::debug!("{}", format!("Try to connect to: {id}"));
    let devices = DEVICES.get().unwrap().lock().await;
    let device = devices
        .get(&id)
        .ok_or(anyhow::anyhow!("UnknownPeripheral(id)"))?;
    device.connect().await
}

pub fn disconnect(id: String) -> Result<()> {
    send(Command::Disconnect { id })
}

async fn inner_disconnect(id: String) -> Result<()> {
    tracing::debug!("{}", format!("Try to disconnect from: {id}"));
    let devices = DEVICES.get().unwrap().lock().await;
    let device = devices
        .get(&id)
        .ok_or(anyhow::anyhow!("UnknownPeripheral(id)"))?;
    device.disconnect().await
}
