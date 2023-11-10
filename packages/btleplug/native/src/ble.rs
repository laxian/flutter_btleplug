use std::collections::HashMap;
use std::fmt::Display;

use anyhow::Result;
use btleplug::api::bleuuid::BleUuid;
use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral as _, ScanFilter};
use btleplug::platform::{Adapter, Manager, PeripheralId};
use flutter_rust_bridge::StreamSink;
use futures::stream::StreamExt;
use once_cell::sync::{Lazy, OnceCell};
use tokio::sync::{self, mpsc, Mutex};
use uuid::Uuid;

mod setup;
pub use setup::*;
mod error;
pub use error::Error;
use tokio::time;

use crate::logger::log;

enum Command {
    Scan {
        sink: StreamSink<Vec<BleDevice>>,
        filter: Vec<String>,
    },
    Event {
        sink: StreamSink<BleEvent>,
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

/// Wrapper struct around btleplug::platform::Peripheral that adds the last_seen variable.
///
#[derive(Debug, Clone)]
struct Peripheral {
    peripheral: btleplug::platform::Peripheral,
    last_seen: time::Instant,
    is_connected: bool,
}

impl Peripheral {
    fn new(peripheral: btleplug::platform::Peripheral) -> Self {
        Self {
            peripheral,
            last_seen: time::Instant::now(),
            is_connected: false,
        }
    }

    fn id(&self) -> PeripheralId {
        self.peripheral.id()
    }

    async fn name(&self) -> Option<String> {
        if let Ok(Some(properties)) = self.peripheral.properties().await {
            properties.local_name
        } else {
            None
        }
    }

    async fn connect(&self) -> Result<()> {
        self.peripheral.connect().await?;
        Ok(())
    }

    async fn disconnect(&self) -> Result<()> {
        self.peripheral.disconnect().await?;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.is_connected
    }
}

static DEVICES: Lazy<Mutex<HashMap<String, Peripheral>>> = Lazy::new(|| Mutex::new(HashMap::new()));

static TX: OnceCell<mpsc::UnboundedSender<Command>> = OnceCell::new();

/// Internal send function to send [Command]s into the message loop.
fn send(command: Command) -> Result<()> {
    let tx = TX.get().ok_or(Error::TxNotInitialized)?;
    tx.send(command)?;
    Ok(())
}

/// The init() function must be called before anything else.
/// At the moment the developer has to make sure it is only called once.
pub fn init() -> Result<()> {
    create_runtime()?;
    let runtime = RUNTIME.get().ok_or(Error::RuntimeNotInitialized)?;

    let (tx, mut rx) = mpsc::unbounded_channel::<Command>();
    TX.set(tx).map_err(|_| Error::TxAlreadySet)?;

    runtime.spawn(async move {
        while let Some(msg) = rx.recv().await {
            match msg {
                Command::Scan { sink, filter } => {
                    tokio::spawn(async { inner_scan(sink, filter).await.unwrap() });
                }
                Command::Event { sink } => {
                    tokio::spawn(async { inner_events(sink).await.unwrap() });
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
    let mut devices = DEVICES.lock().await;
    devices.retain(|_, d| d.is_connected() || d.last_seen.elapsed().as_secs() < timeout);
}

/// This is the BleDevice intended to show in Dart/Flutter
#[derive(Debug, Clone)]
pub struct BleDevice {
    pub id: String,
    pub name: String,
}

impl BleDevice {
    async fn from_peripheral(peripheral: &Peripheral) -> Self {
        Self {
            id: peripheral.id().to_string(),
            name: peripheral.name().await.unwrap_or_default(),
        }
    }
}

static CENTRAL: sync::OnceCell<Adapter> = sync::OnceCell::const_new();

async fn init_adapter() -> Adapter {
    let manager = Manager::new().await.expect("Init manager failed!");
    let adapters = manager.adapters().await.expect("Get adapters failed!");
    adapters.into_iter().next().expect("cannot fail")
}

pub struct MapData {
    pub key: String,
    pub data: Vec<u8>,
}

impl MapData {
    fn from_data<T: Display>(data: HashMap<T, Vec<u8>>) -> Vec<Self> {
        data.into_iter()
            .map(|(k, v)| MapData {
                key: k.to_string(),
                data: v,
            })
            .collect()
    }
}

pub enum BleEvent {
    DeviceDiscovered {
        id: String,
    },
    DeviceUpdated {
        id: String,
    },
    DeviceConnected {
        id: String,
    },
    DeviceDisconnected {
        id: String,
    },
    ManufacturerDataAdvertisement {
        id: String,
        manufacturer_data: Vec<MapData>,
    },
    ServiceDataAdvertisement {
        id: String,
        service_data: Vec<MapData>,
    },
    ServicesAdvertisement {
        id: String,
        services: Vec<String>,
    },
}

impl BleEvent {
    fn from_central_event(event: CentralEvent) -> Self {
        match event {
            CentralEvent::DeviceDiscovered(id) => BleEvent::DeviceDiscovered { id: id.to_string() },
            CentralEvent::DeviceUpdated(id) => BleEvent::DeviceUpdated { id: id.to_string() },
            CentralEvent::DeviceConnected(id) => BleEvent::DeviceConnected { id: id.to_string() },
            CentralEvent::DeviceDisconnected(id) => {
                BleEvent::DeviceDisconnected { id: id.to_string() }
            }
            CentralEvent::ManufacturerDataAdvertisement {
                id,
                manufacturer_data,
            } => BleEvent::ManufacturerDataAdvertisement {
                id: id.to_string(),
                manufacturer_data: MapData::from_data(manufacturer_data),
            },
            CentralEvent::ServiceDataAdvertisement { id, service_data } => {
                BleEvent::ServiceDataAdvertisement {
                    id: id.to_string(),
                    service_data: MapData::from_data(service_data),
                }
            }
            CentralEvent::ServicesAdvertisement { id, services } => {
                BleEvent::ServicesAdvertisement {
                    id: id.to_string(),
                    services: services.iter().map(|id| id.to_string()).collect(),
                }
            }
        }
    }
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
async fn send_devices(sink: &StreamSink<Vec<BleDevice>>) -> bool {
    let devices = DEVICES.lock().await;
    let mut d = vec![];
    for device in devices.values() {
        let dev = BleDevice::from_peripheral(&device).await;
        d.push(dev.clone())
    }
    sink.add(d)
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

pub fn events(sink: StreamSink<BleEvent>) -> Result<()> {
    send(Command::Event { sink: sink })
}

async fn inner_scan(sink: StreamSink<Vec<BleDevice>>, _filter: Vec<String>) -> Result<()> {
    let central = CENTRAL.get_or_init(init_adapter).await;
    let mut events = central.events().await?;

    // start scanning for devices
    log(format!(
        "start scanning on {}",
        central.adapter_info().await?
    ));
    central
        .start_scan(ScanFilter {
            services: _filter
                .iter()
                .map(|s| Uuid::parse_str(s).unwrap())
                .collect(),
        })
        .await?;

    let mut device_send_interval = time::interval(time::Duration::from_secs(1));
    loop {
        tokio::select! {
            _ = device_send_interval.tick() => {
                remove_stale_devices(3).await;
                if send_devices(&sink).await == false {
                    break;
                }
            }
            Some(event) = events.next() => {
                // log(format!("{:?}", event));
                match event {
                    CentralEvent::DeviceDiscovered(id) => {
                        log(format!("DeviceDiscovered: {:?}", &id));
                        let peripheral = central.peripheral(&id).await?;
                        let peripheral = Peripheral::new(peripheral);
                        let mut devices = DEVICES.lock().await;
                        devices.insert(id.to_string(), peripheral);
                    }
                    CentralEvent::DeviceUpdated(id) => {
                        let mut devices = DEVICES.lock().await;
                        if let Some(device) = devices.get_mut(&id.to_string()) {
                            device.last_seen = time::Instant::now();
                        }
                    }
                    CentralEvent::DeviceConnected(id) => {
                        log(format!("DeviceConnected: {:?}", id));
                        let mut devices = DEVICES.lock().await;
                        if let Some(device) = devices.get_mut(&id.to_string()) {
                            device.is_connected = true;
                        }
                    }
                    CentralEvent::DeviceDisconnected(id) => {
                        log(format!("DeviceDisconnected: {:?}", id));
                        let mut devices = DEVICES.lock().await;
                        if let Some(device) = devices.get_mut(&id.to_string()) {
                            device.is_connected = false;
                        }
                    }
                    CentralEvent::ManufacturerDataAdvertisement {
                        id,
                        manufacturer_data,
                    } => {
                        // log(format!(
                        //     "ManufacturerDataAdvertisement: {:?}, {:?}",
                        //     id, manufacturer_data
                        // ));
                    }
                    CentralEvent::ServiceDataAdvertisement { id, service_data } => {
                        // log(format!(
                        //     "ServiceDataAdvertisement: {:?}, {:?}",
                        //     id, service_data
                        // ));
                    }
                    CentralEvent::ServicesAdvertisement { id, services } => {
                        let services: Vec<String> =
                            services.into_iter().map(|s| s.to_short_string()).collect();
                        // log(format!("ServicesAdvertisement: {:?}, {:?}", id, services));
                    }
                }
            }
        }
    }
    central.stop_scan().await?;
    log("Scan finished");
    Ok(())
}

async fn inner_events(sink: StreamSink<BleEvent>) -> Result<()> {
    let central = CENTRAL.get_or_init(init_adapter).await;
    let mut events = central.events().await?;
    loop {
        if let Some(event) = events.next().await {
            if !sink.add(BleEvent::from_central_event(event)) {
                break;
            }
        }
    }
    Ok(())
}

pub fn connect(id: String) -> Result<()> {
    log(format!("Try to connect to: {id}"));
    send(Command::Connect { id })
}

async fn inner_connect(id: String) -> Result<()> {
    log(format!("Try to connect to: {id}"));
    let devices = DEVICES.lock().await;
    let device = devices.get(&id).ok_or(Error::UnknownPeripheral(id))?;
    device.connect().await
}

pub fn disconnect(id: String) -> Result<()> {
    send(Command::Disconnect { id })
}

async fn inner_disconnect(id: String) -> Result<()> {
    log(format!("Try to disconnect from: {id}"));
    let devices = DEVICES.lock().await;
    let device = devices.get(&id).ok_or(Error::UnknownPeripheral(id))?;
    device.disconnect().await
}
