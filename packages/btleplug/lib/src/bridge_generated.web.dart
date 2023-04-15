// AUTO GENERATED FILE, DO NOT EDIT.
// Generated by `flutter_rust_bridge`@ 1.72.2.
// ignore_for_file: non_constant_identifier_names, unused_element, duplicate_ignore, directives_ordering, curly_braces_in_flow_control_structures, unnecessary_lambdas, slash_for_doc_comments, prefer_const_literals_to_create_immutables, implicit_dynamic_list_literal, duplicate_import, unused_import, unnecessary_import, prefer_single_quotes, prefer_const_constructors, use_super_parameters, always_use_package_imports, annotate_overrides, invalid_use_of_protected_member, constant_identifier_names, invalid_use_of_internal_member, prefer_is_empty, unnecessary_const

import 'dart:convert';
import 'dart:async';
import 'package:meta/meta.dart';
import 'package:flutter_rust_bridge/flutter_rust_bridge.dart';
import 'package:uuid/uuid.dart';
import 'bridge_generated.dart';
export 'bridge_generated.dart';

class BtleplugPlatform extends FlutterRustBridgeBase<BtleplugWire>
    with FlutterRustBridgeSetupMixin {
  BtleplugPlatform(FutureOr<WasmModule> dylib) : super(BtleplugWire(dylib)) {
    setupMixinConstructor();
  }
  Future<void> setup() => inner.init;

// Section: api2wire

  @protected
  String api2wire_String(String raw) {
    return raw;
  }

  @protected
  List<String> api2wire_StringList(List<String> raw) {
    return raw;
  }

  @protected
  Uint8List api2wire_uint_8_list(Uint8List raw) {
    return raw;
  }
// Section: finalizer
}

// Section: WASM wire module

@JS('wasm_bindgen')
external BtleplugWasmModule get wasmModule;

@JS()
@anonymous
class BtleplugWasmModule implements WasmModule {
  external Object /* Promise */ call([String? moduleName]);
  external BtleplugWasmModule bind(dynamic thisArg, String moduleName);
  external dynamic /* void */ wire_init(NativePortType port_);

  external dynamic /* void */ wire_scan(
      NativePortType port_, List<String> filter);

  external dynamic /* void */ wire_connect(NativePortType port_, String id);

  external dynamic /* void */ wire_create_log_stream(NativePortType port_);
}

// Section: WASM wire connector

class BtleplugWire extends FlutterRustBridgeWasmWireBase<BtleplugWasmModule> {
  BtleplugWire(FutureOr<WasmModule> module)
      : super(WasmModule.cast<BtleplugWasmModule>(module));

  void wire_init(NativePortType port_) => wasmModule.wire_init(port_);

  void wire_scan(NativePortType port_, List<String> filter) =>
      wasmModule.wire_scan(port_, filter);

  void wire_connect(NativePortType port_, String id) =>
      wasmModule.wire_connect(port_, id);

  void wire_create_log_stream(NativePortType port_) =>
      wasmModule.wire_create_log_stream(port_);
}
