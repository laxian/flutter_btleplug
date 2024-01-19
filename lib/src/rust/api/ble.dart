// This file is automatically generated, so please do not edit it.
// Generated by `flutter_rust_bridge`@ 2.0.0-dev.20.

// ignore_for_file: invalid_use_of_internal_member, unused_import, unnecessary_import

import '../frb_generated.dart';
import 'ble/device.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge_for_generated.dart';

/// The init() function must be called before anything else.
/// At the moment the developer has to make sure it is only called once.
Future<void> init({dynamic hint}) => RustLib.instance.api.init(hint: hint);

/// This function is used to scan for BLE devices and returns the results via the given stream sink.
///
/// Parameters
///
/// sink: StreamSink<Vec<BleDevice>> - A stream sink to which the results are send
///
/// filter: Vec<String> - A vector of strings to filter the results with
Stream<List<BleDevice>> scan({required List<String> filter, dynamic hint}) =>
    RustLib.instance.api.scan(filter: filter, hint: hint);

Future<void> connect({required String id, dynamic hint}) =>
    RustLib.instance.api.connect(id: id, hint: hint);

Future<void> disconnect({required String id, dynamic hint}) =>
    RustLib.instance.api.disconnect(id: id, hint: hint);
