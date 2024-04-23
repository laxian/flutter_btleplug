use std::collections::HashMap;
use std::sync::{Arc, OnceLock};

use anyhow::Result;
use btleplug::api::{Central, CentralEvent, Manager as _, ScanFilter};
pub use btleplug::platform::Manager;
use flutter_rust_bridge::frb;
use futures::stream::StreamExt;
use jni::objects::{JClass, JObject, JString, JValue};
use jni::sys::{jint, jobjectArray};
use jni::JNIEnv;
use tokio::runtime::Runtime;
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

use uuid::Uuid;

enum Command {
    Scan {
        sink: StreamSink<Vec<BleDevice>>,
        filter: Vec<String>,
    },
    StopScan,
    Connect {
        id: String,
    },
    Disconnect {
        id: String,
    },
}

impl std::fmt::Debug for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Scan { .. } => f.debug_struct("Scan").finish(),
            Command::Connect { .. } => f.debug_struct("Connect").finish(),
            Command::Disconnect { .. } => f.debug_struct("Disconnect").finish(),
            Command::StopScan => f.debug_struct("StopScan").finish(),
        }
    }
}

static DEVICES: OnceLock<Arc<Mutex<HashMap<String, Peripheral>>>> = OnceLock::new();

static TX: OnceLock<mpsc::UnboundedSender<Command>> = OnceLock::new();

use std::sync::atomic::{AtomicBool, Ordering};

static IS_SCANNING: AtomicBool = AtomicBool::new(false);

fn set_is_scanning(value: bool) {
    IS_SCANNING.store(value, Ordering::SeqCst);
}

fn get_is_scanning() -> bool {
    IS_SCANNING.load(Ordering::SeqCst)
}

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
                    tokio::spawn(async { inner_scan(filter, move |devices| {
                        sink.add(devices);
                    }).await.unwrap() });
                }
                Command::Connect { id } => inner_connect(id).await.unwrap(),
                Command::Disconnect { id } => inner_disconnect(id).await.unwrap(),
                Command::StopScan => {inner_stop_scan().await.unwrap()}
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
async fn inner_scan<F>(_filter: Vec<String>, sink_callback: F) -> Result<()>
where
    F: Fn(Vec<BleDevice>) + Send + 'static,
{
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let central = adapters.into_iter().next().expect("cannot fail");
    let mut events = central.events().await?;

    // start scanning for devices
    tracing::debug!(
        "{}",
        format!("start scanning on {}", central.adapter_info().await?)
    );
    set_is_scanning(true);
    central.start_scan(ScanFilter {
        services: _filter.iter().map(|s| Uuid::parse_str(s).unwrap()).collect(),
    })
    .await?;
    trace!("ZWX start_scan {:?}", _filter);
    let mut cnt = 0;
    let mut device_send_interval = time::interval(time::Duration::from_secs(1));
    while get_is_scanning() {
        tokio::select! {
            _ = device_send_interval.tick() => {
                // 超30s没更新的设备，从设备列表中移除
                remove_stale_devices(30).await;
                cnt += 1;
                if cnt > 10 {
                    trace!("ZWX stop scan");
                    set_is_scanning(false);
                }
            }
            Some(event) = events.next() => {
                match event {
                    CentralEvent::DeviceDiscovered(id) => {
                        tracing::debug!("{}",format!("DeviceDiscovered: {:?}", &id));
                        let peripheral = central.peripheral(&id).await?;
                        let peripheral = Peripheral::new(peripheral);
                        let mut devices = DEVICES.get().unwrap().lock().await;
                        devices.insert(id.to_string(), peripheral);
                        // 释放锁
                        drop(devices);
                        // 发现设备立即调用闭包将设备添加到给定的 StreamSink
                        let devices = DEVICES.get().unwrap().lock().await;
                        let devices = devices.values().map(|d| BleDevice::from_peripheral(d)).collect::<Vec<_>>();
                        let devices = futures::future::join_all(devices).await;
                        sink_callback(devices);
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
                    } => {}
                    CentralEvent::ServiceDataAdvertisement { id:_, service_data:_ } => {}
                    CentralEvent::ServicesAdvertisement { id:_, services } => {}
                }
            }
        }
    }
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

/// Stops the ongoing BLE device scan.
pub fn stop_scan() -> Result<()> {
    send(Command::StopScan)
}

async fn inner_stop_scan() -> Result<()> {
    set_is_scanning(false);
    let manager = Manager::new().await?;
    let adapters = manager.adapters().await?;
    let central = adapters.into_iter().next().expect("cannot fail");
    central.stop_scan().await?;
    Ok(())
}


// 包装异步 Rust 函数，使其能够从 C++ 中调用
#[frb(ignore)]
#[no_mangle]
pub extern "C" fn scan_wrapper<F>(filter: Vec<String>, sink_callback: F) 
where
    F: Fn(Vec<BleDevice>) + Send + 'static,
{
    // 将回调函数转换为 Rust 的闭包
    let cb = move |devices: Vec<BleDevice>| {
        // 将 devices 转换为 C++ 中的字符串数组并调用回调函数
        sink_callback(devices);
    };

    // 调用异步 Rust 函数
    inner_scan(filter, cb);
}

// 声明 init_wrapper 函数
#[frb(ignore)]
#[no_mangle]
pub extern "C" fn init_wrapper() -> i32 {
    match init() {
        Ok(_) => 0,
        Err(_) => 1,
    }
}

#[no_mangle]
pub extern "C" fn rust_add(n1: i32, n2: i32) -> i32 {
    n1 + n2
}

use log::{trace, LevelFilter};
use android_logger::Config;

fn native_activity_create() {
    android_logger::init_once(
        Config::default().with_max_level(LevelFilter::Trace),
    );
}

#[frb(ignore)]
#[no_mangle]
pub extern "C" fn Java_com_example_bleandroid_RustBridge_initWrapper(env: JNIEnv, _class: JClass) -> i32 {
    trace!("ZWX init ----------------------------------------------------------------");
    init_wrapper()
}

#[frb(ignore)]
#[no_mangle]
pub extern "C" fn Java_com_example_bleandroid_RustBridge_add(env: JNIEnv, _class: JClass, n1: jint, n2: jint) -> jint {
    native_activity_create();
    trace!("ZWX add ----------------------------------------------------------------");
    rust_add(n1, n2) * 2
}

#[frb(ignore)]
#[no_mangle]
pub extern "system" fn Java_com_example_bleandroid_RustBridge_scanWrapper(
    env: JNIEnv,
    _: JObject, // jobject thiz
    filter: jobjectArray,
    callback: JObject,
) {
    trace!("ZWX scan ----------------------------------------------------------------");
    let callback = env.new_global_ref(callback).unwrap();

    // Convert jobjectArray to Vec<JObject>
    let filter_vec: Vec<String> = (0..env.get_array_length(filter).unwrap()).map(|i| {
        let item: Result<JString<'_>, jni::errors::Error> = env.get_object_array_element(filter, i as i32).map_or_else(|e| Err(e), |obj| Ok(JString::from(obj)));
        let item_str = match env.get_string(item.unwrap()) {
        Ok(s) => s.into(),
        Err(e) => {
            // Handle the error
            panic!("Failed to convert jstring to String: {}", e);
        }
        };
        item_str
    }).collect();

    let jvm = env.get_java_vm().unwrap();

    // 将回调函数转换为 Rust 的闭包
    let cb = move |devices: Vec<BleDevice>| {
        trace!("ZWX cb ----------------------------------------------------------------");
        let env = jvm.attach_current_thread().unwrap();
        
        // Convert Vec<BleDevice> to Array<BleDevice>
        let array_class = env.find_class("com/example/bleandroid/BleDevice").unwrap();
        let array = env.new_object_array(devices.len() as i32, array_class, JObject::null()).unwrap();
        for (i, device) in devices.iter().enumerate() {
            let java_device = env.new_object(
                "com/example/bleandroid/BleDevice",
                "(Ljava/lang/String;Ljava/lang/String;)V",
                &[JValue::Object(env.new_string(&device.id).unwrap().into()), JValue::Object(env.new_string(&device.name).unwrap().into())],
            ).unwrap();
            env.set_object_array_element(array, i as i32, java_device).unwrap();
        }
        
        // Call the Kotlin callback method
        // let method_name_d = "demo";
        // let method_signature_d = "()V";
        // env.call_method(&callback, method_name_d, method_signature_d, &[]).expect("call demo error");
        // trace!("ZWX call kotlin demo ----------------------------------------------------------------");
        // Create a Rust string
        
        // let msg = "hello rust";
        // // Convert Rust string to JNI string
        // let jni_msg = env.new_string(msg).expect("Couldn't create JNI string from Rust string");
        // let method_name_d = "demo1";
        // let method_signature_d = "(Ljava/lang/String;)V";
        // env.call_method(&callback, method_name_d, method_signature_d, &[jni_msg.into()]).expect("call demo error");
        
        // 创建一个包含字符串的数组
        // let strings: Vec<&str> = vec!["String 1", "String 2", "String 3"];
        // // 创建 JNI 字符串数组并将 Rust 字符串转换为 JNI 字符串并添加到数组中
        // let array = env.new_object_array(strings.len() as i32, "java/lang/String", JObject::null()).unwrap();
        // for (index, &string) in strings.iter().enumerate() {
        //     let jni_string = env.new_string(string).unwrap();
        //     env.set_object_array_element(array, index as i32, jni_string).unwrap();
        // }
    
        // // 调用 Kotlin 方法 demo2
        // let method_name_d = "demo2";
        // let method_signature_d = "([Ljava/lang/String;)V";
        // env.call_method(&callback, method_name_d, method_signature_d, &[JValue::Object(array.into())]).expect("call demo error");
        

        // 调用 Kotlin 方法 demo2
        let method_name_d = "onDeviceFound";
        let method_signature_d = "([Lcom/example/bleandroid/BleDevice;)V";
        env.call_method(&callback, method_name_d, method_signature_d, &[JValue::Object(array.into())]).expect("call demo error");
    

        trace!("ZWX call kotlin cb end ----------------------------------------------------------------");
    };
    let future = async { 
            trace!("ZWX spawn ----------------------------------------------------------------");
            inner_scan(filter_vec, move |devices| {
                trace!("{}",format!("DeviceDiscovered: {:?}", devices));
                cb(devices);
            }).await.unwrap();
        };
        let runtime = Runtime::new().unwrap();
        runtime.block_on(future);
        trace!("ZWX return");
}
