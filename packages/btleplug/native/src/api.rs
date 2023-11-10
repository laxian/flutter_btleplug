use crate::ble::{self, BleDevice, BleEvent};
use crate::logger::{self, LogEntry};
use anyhow::Result;
use flutter_rust_bridge::StreamSink;

pub fn init() -> Result<()> {
    ble::init()
}

/// Scan for Bluetooth Low Energy devices and send the results through the given sink.
/// In Dart/Flutter one can listen() to bleScan(). The scan is automatically stopped when the subscription is cancelled.
///
/// # Parameters
///
/// * `sink` - A stream sink that new discovered devices will be passed through.
/// * `filter` - A list of filter strings to apply to the scan result.
///
/// # Return
///
/// Returns a `Result<()>` indicating if the scan operation has successfully started.
///
/// # Dart/Flutter Example
/// ```dart
/// final scan = bleScan().await;
/// scan.listen((devices) {
/// ...
/// })
/// ```
pub fn scan(sink: StreamSink<Vec<BleDevice>>, filter: Vec<String>) -> Result<()> {
    ble::scan(sink, filter)
}

pub fn events(sink: StreamSink<BleEvent>) -> Result<()> {
    ble::events(sink)
}

pub fn connect(id: String) -> Result<()> {
    ble::connect(id)
}

pub fn disconnect(id: String) -> Result<()> {
    ble::disconnect(id)
}

pub fn create_log_stream(s: StreamSink<LogEntry>) {
    logger::create_log_stream(s);
}
