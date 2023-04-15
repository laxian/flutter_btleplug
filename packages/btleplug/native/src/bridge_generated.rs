#![allow(
    non_camel_case_types,
    unused,
    clippy::redundant_closure,
    clippy::useless_conversion,
    clippy::unit_arg,
    clippy::double_parens,
    non_snake_case,
    clippy::too_many_arguments
)]
// AUTO GENERATED FILE, DO NOT EDIT.
// Generated by `flutter_rust_bridge`@ 1.72.2.

use crate::api::*;
use core::panic::UnwindSafe;
use flutter_rust_bridge::*;
use std::ffi::c_void;
use std::sync::Arc;

// Section: imports

use crate::ble::BleDevice;
use crate::logger::LogEntry;

// Section: wire functions

fn wire_init_impl(port_: MessagePort) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "init",
            port: Some(port_),
            mode: FfiCallMode::Normal,
        },
        move || move |task_callback| init(),
    )
}
fn wire_scan_impl(port_: MessagePort, filter: impl Wire2Api<Vec<String>> + UnwindSafe) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "scan",
            port: Some(port_),
            mode: FfiCallMode::Stream,
        },
        move || {
            let api_filter = filter.wire2api();
            move |task_callback| scan(task_callback.stream_sink(), api_filter)
        },
    )
}
fn wire_connect_impl(port_: MessagePort, id: impl Wire2Api<String> + UnwindSafe) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "connect",
            port: Some(port_),
            mode: FfiCallMode::Normal,
        },
        move || {
            let api_id = id.wire2api();
            move |task_callback| connect(api_id)
        },
    )
}
fn wire_disconnect_impl(port_: MessagePort, id: impl Wire2Api<String> + UnwindSafe) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "disconnect",
            port: Some(port_),
            mode: FfiCallMode::Normal,
        },
        move || {
            let api_id = id.wire2api();
            move |task_callback| disconnect(api_id)
        },
    )
}
fn wire_create_log_stream_impl(port_: MessagePort) {
    FLUTTER_RUST_BRIDGE_HANDLER.wrap(
        WrapInfo {
            debug_name: "create_log_stream",
            port: Some(port_),
            mode: FfiCallMode::Stream,
        },
        move || move |task_callback| Ok(create_log_stream(task_callback.stream_sink())),
    )
}
// Section: wrapper structs

// Section: static checks

// Section: allocate functions

// Section: related functions

// Section: impl Wire2Api

pub trait Wire2Api<T> {
    fn wire2api(self) -> T;
}

impl<T, S> Wire2Api<Option<T>> for *mut S
where
    *mut S: Wire2Api<T>,
{
    fn wire2api(self) -> Option<T> {
        (!self.is_null()).then(|| self.wire2api())
    }
}

impl Wire2Api<u8> for u8 {
    fn wire2api(self) -> u8 {
        self
    }
}

// Section: impl IntoDart

impl support::IntoDart for BleDevice {
    fn into_dart(self) -> support::DartAbi {
        vec![self.id.into_dart(), self.name.into_dart()].into_dart()
    }
}
impl support::IntoDartExceptPrimitive for BleDevice {}

impl support::IntoDart for LogEntry {
    fn into_dart(self) -> support::DartAbi {
        vec![self.time_millis.into_dart(), self.msg.into_dart()].into_dart()
    }
}
impl support::IntoDartExceptPrimitive for LogEntry {}

// Section: executor

support::lazy_static! {
    pub static ref FLUTTER_RUST_BRIDGE_HANDLER: support::DefaultHandler = Default::default();
}

/// cbindgen:ignore
#[cfg(target_family = "wasm")]
mod web {
    use super::*;
    // Section: wire functions

    #[wasm_bindgen]
    pub fn wire_init(port_: MessagePort) {
        wire_init_impl(port_)
    }

    #[wasm_bindgen]
    pub fn wire_scan(port_: MessagePort, filter: JsValue) {
        wire_scan_impl(port_, filter)
    }

    #[wasm_bindgen]
    pub fn wire_connect(port_: MessagePort, id: String) {
        wire_connect_impl(port_, id)
    }

    #[wasm_bindgen]
    pub fn wire_disconnect(port_: MessagePort, id: String) {
        wire_disconnect_impl(port_, id)
    }

    #[wasm_bindgen]
    pub fn wire_create_log_stream(port_: MessagePort) {
        wire_create_log_stream_impl(port_)
    }

    // Section: allocate functions

    // Section: related functions

    // Section: impl Wire2Api

    impl Wire2Api<String> for String {
        fn wire2api(self) -> String {
            self
        }
    }
    impl Wire2Api<Vec<String>> for JsValue {
        fn wire2api(self) -> Vec<String> {
            self.dyn_into::<JsArray>()
                .unwrap()
                .iter()
                .map(Wire2Api::wire2api)
                .collect()
        }
    }

    impl Wire2Api<Vec<u8>> for Box<[u8]> {
        fn wire2api(self) -> Vec<u8> {
            self.into_vec()
        }
    }
    // Section: impl Wire2Api for JsValue

    impl Wire2Api<String> for JsValue {
        fn wire2api(self) -> String {
            self.as_string().expect("non-UTF-8 string, or not a string")
        }
    }
    impl Wire2Api<u8> for JsValue {
        fn wire2api(self) -> u8 {
            self.unchecked_into_f64() as _
        }
    }
    impl Wire2Api<Vec<u8>> for JsValue {
        fn wire2api(self) -> Vec<u8> {
            self.unchecked_into::<js_sys::Uint8Array>().to_vec().into()
        }
    }
}
#[cfg(target_family = "wasm")]
pub use web::*;

#[cfg(not(target_family = "wasm"))]
mod io {
    use super::*;
    // Section: wire functions

    #[no_mangle]
    pub extern "C" fn wire_init(port_: i64) {
        wire_init_impl(port_)
    }

    #[no_mangle]
    pub extern "C" fn wire_scan(port_: i64, filter: *mut wire_StringList) {
        wire_scan_impl(port_, filter)
    }

    #[no_mangle]
    pub extern "C" fn wire_connect(port_: i64, id: *mut wire_uint_8_list) {
        wire_connect_impl(port_, id)
    }

    #[no_mangle]
    pub extern "C" fn wire_disconnect(port_: i64, id: *mut wire_uint_8_list) {
        wire_disconnect_impl(port_, id)
    }

    #[no_mangle]
    pub extern "C" fn wire_create_log_stream(port_: i64) {
        wire_create_log_stream_impl(port_)
    }

    // Section: allocate functions

    #[no_mangle]
    pub extern "C" fn new_StringList_0(len: i32) -> *mut wire_StringList {
        let wrap = wire_StringList {
            ptr: support::new_leak_vec_ptr(<*mut wire_uint_8_list>::new_with_null_ptr(), len),
            len,
        };
        support::new_leak_box_ptr(wrap)
    }

    #[no_mangle]
    pub extern "C" fn new_uint_8_list_0(len: i32) -> *mut wire_uint_8_list {
        let ans = wire_uint_8_list {
            ptr: support::new_leak_vec_ptr(Default::default(), len),
            len,
        };
        support::new_leak_box_ptr(ans)
    }

    // Section: related functions

    // Section: impl Wire2Api

    impl Wire2Api<String> for *mut wire_uint_8_list {
        fn wire2api(self) -> String {
            let vec: Vec<u8> = self.wire2api();
            String::from_utf8_lossy(&vec).into_owned()
        }
    }
    impl Wire2Api<Vec<String>> for *mut wire_StringList {
        fn wire2api(self) -> Vec<String> {
            let vec = unsafe {
                let wrap = support::box_from_leak_ptr(self);
                support::vec_from_leak_ptr(wrap.ptr, wrap.len)
            };
            vec.into_iter().map(Wire2Api::wire2api).collect()
        }
    }

    impl Wire2Api<Vec<u8>> for *mut wire_uint_8_list {
        fn wire2api(self) -> Vec<u8> {
            unsafe {
                let wrap = support::box_from_leak_ptr(self);
                support::vec_from_leak_ptr(wrap.ptr, wrap.len)
            }
        }
    }
    // Section: wire structs

    #[repr(C)]
    #[derive(Clone)]
    pub struct wire_StringList {
        ptr: *mut *mut wire_uint_8_list,
        len: i32,
    }

    #[repr(C)]
    #[derive(Clone)]
    pub struct wire_uint_8_list {
        ptr: *mut u8,
        len: i32,
    }

    // Section: impl NewWithNullPtr

    pub trait NewWithNullPtr {
        fn new_with_null_ptr() -> Self;
    }

    impl<T> NewWithNullPtr for *mut T {
        fn new_with_null_ptr() -> Self {
            std::ptr::null_mut()
        }
    }

    // Section: sync execution mode utility

    #[no_mangle]
    pub extern "C" fn free_WireSyncReturn(ptr: support::WireSyncReturn) {
        unsafe {
            let _ = support::box_from_leak_ptr(ptr);
        };
    }
}
#[cfg(not(target_family = "wasm"))]
pub use io::*;
