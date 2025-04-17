#![allow(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    dead_code,
    unsafe_op_in_unsafe_fn
)]

use libloading::{Library, Symbol};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uchar, c_void};

mod binding;
use binding::*;

#[cfg(target_os = "windows")]
const LIB_NAME: &str = "Websocket.dll";
#[cfg(target_os = "linux")]
const LIB_NAME: &str = "Websocket.so";
#[cfg(target_os = "macos")]
const LIB_NAME: &str = "Websocket.dylib";

fn default_ws_settings() -> ws_settings_t {
    let mut s: ws_settings_t = unsafe { core::mem::zeroed() };
    s.endpoint = e_ws_endpoint_type_endpoint_server;
    s.mode = e_ws_mode_mode_unsecured;
    s.ping_interval = 60_000;
    s.ping_timeout = 30_000;
    s.message_limit = 4 * 1024 * 1024;
    s.auto_mask_frame = true;
    s.extensions.permessage_deflate.enabled = false;
    s.extensions.permessage_deflate.window_bits = 15;
    s
}

unsafe fn destroy_ws_settings(s: &mut ws_settings_t) {
    for ptr in [
        s.ssl_seed,
        s.ssl_ca_cert,
        s.ssl_own_cert,
        s.ssl_private_key,
        s.host,
        s.allowed_origin,
    ] {
        if !ptr.is_null() {
            unsafe { libc::free(ptr.cast()) };
        }
    }
    unsafe { *s = core::mem::zeroed() };
}

const EVT_OPEN: &[u8] = b"open\0";
const EVT_CLOSE: &[u8] = b"close\0";
const EVT_FRAME: &[u8] = b"frame\0";
const EVT_ERROR: &[u8] = b"error\0";

unsafe extern "C" fn on_open(_ctx: *mut c_void, fd: c_int, addr: *const c_char) {
    let s = if addr.is_null() {
        "<null>"
    } else {
        unsafe { CStr::from_ptr(addr) }
            .to_str()
            .unwrap_or("<utf8 err>")
    };
    println!("[open] fd={fd} addr={s}");
}

unsafe extern "C" fn on_close(_ctx: *mut c_void, fd: c_int, st: e_ws_closure_status) {
    println!("[close] fd={fd} status={st}");
}

unsafe extern "C" fn on_frame(
    _ctx: *mut c_void,
    fd: c_int,
    opcode: e_ws_frame_opcode,
    data: *const c_uchar,
    len: usize,
) {
    let slice = core::slice::from_raw_parts(data, len);
    match opcode {
        // text frame → try UTF‑8
        e_ws_frame_opcode_opcode_text => match core::str::from_utf8(slice) {
            Ok(txt) => println!("[frame] fd={fd} text: {txt}"),
            Err(_) => println!("[frame] fd={fd} invalid UTF‑8 ({} bytes)", len),
        },
        // anything else → show hex
        _ => {
            let hex: String = slice.iter().map(|b| format!("{:02X} ", b)).collect();
            println!("[frame] fd={fd} opcode={opcode:?} {len} bytes: {hex}");
        }
    }
}

unsafe extern "C" fn on_error(_ctx: *mut c_void, msg: *const c_char) {
    let s = if msg.is_null() {
        "<null>"
    } else {
        unsafe { CStr::from_ptr(msg) }
            .to_str()
            .unwrap_or("<utf8 err>")
    };
    eprintln!("[error] {s}");
}

type WsResult<T> = Result<T, Box<dyn std::error::Error>>;

fn main() -> WsResult<()> {
    let lib = unsafe { Library::new(LIB_NAME) }?;
    unsafe {
        let websocket_create: Symbol<unsafe extern "C" fn() -> *mut c_void> =
            lib.get(b"websocket_create\0")?;
        let websocket_destroy: Symbol<unsafe extern "C" fn(*mut c_void)> =
            lib.get(b"websocket_destroy\0")?;
        let websocket_setup: Symbol<
            unsafe extern "C" fn(*mut c_void, *const ws_settings_t) -> e_ws_status,
        > = lib.get(b"websocket_setup\0")?;
        let websocket_operate: Symbol<unsafe extern "C" fn(*mut c_void) -> bool> =
            lib.get(b"websocket_operate\0")?;
        let websocket_bind: Symbol<
            unsafe extern "C" fn(
                *mut c_void,
                *const c_char,
                *const c_char,
                *mut c_int,
            ) -> e_ws_status,
        > = lib.get(b"websocket_bind\0")?;
        let websocket_on: Symbol<
            unsafe extern "C" fn(*mut c_void, *const c_char, *mut c_void) -> e_ws_status,
        > = lib.get(b"websocket_on\0")?;

        let mut settings = default_ws_settings();
        let host = CString::new("localhost:4433").unwrap();
        settings.host = host.into_raw();

        let ctx = websocket_create();
        if ctx.is_null() {
            eprintln!("websocket_create failed");
            destroy_ws_settings(&mut settings);
            return Ok(());
        }

        for (ev, cb) in [
            (EVT_OPEN, on_open as *mut c_void),
            (EVT_CLOSE, on_close as *mut c_void),
            (EVT_FRAME, on_frame as *mut c_void),
            (EVT_ERROR, on_error as *mut c_void),
        ] {
            if websocket_on(ctx, ev.as_ptr().cast(), cb) == e_ws_status_status_error {
                eprintln!("failed to register {ev:?}");
            }
        }

        if websocket_setup(ctx, &settings) == e_ws_status_status_error {
            eprintln!("setup failed");
            websocket_destroy(ctx);
            destroy_ws_settings(&mut settings);
            return Ok(());
        }

        if websocket_bind(
            ctx,
            b"localhost\0".as_ptr().cast(),
            b"4433\0".as_ptr().cast(),
            core::ptr::null_mut(),
        ) == e_ws_status_status_error
        {
            eprintln!("bind failed");
            websocket_destroy(ctx);
            destroy_ws_settings(&mut settings);
            return Ok(());
        }

        println!("WebSocket server running (Rust)…  Ctrl+C to stop");
        while websocket_operate(ctx) {}

        websocket_destroy(ctx);
        destroy_ws_settings(&mut settings);
    }
    Ok(())
}
