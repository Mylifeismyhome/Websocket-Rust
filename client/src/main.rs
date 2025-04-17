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
use std::sync::OnceLock;

mod binding;
use binding::*;

#[cfg(target_os = "windows")]
const LIB_NAME: &str = "Websocket.dll";
#[cfg(target_os = "linux")]
const LIB_NAME: &str = "Websocket.so";
#[cfg(target_os = "macos")]
const LIB_NAME: &str = "Websocket.dylib";

// ── runtime‑loaded helpers for frame creation/emission ───────────────────────
static FRAME_CREATE: OnceLock<unsafe extern "C" fn(e_ws_frame_opcode) -> *mut c_void> =
    OnceLock::new();
static FRAME_PUSH: OnceLock<unsafe extern "C" fn(*mut c_void, *const c_uchar, usize) -> bool> =
    OnceLock::new();
static FRAME_EMIT: OnceLock<unsafe extern "C" fn(*mut c_void, c_int, *mut c_void) -> bool> =
    OnceLock::new();
static FRAME_DESTROY: OnceLock<unsafe extern "C" fn(*mut c_void)> = OnceLock::new();

// ── utility ──────────────────────────────────────────────────────────────────
fn default_ws_settings() -> ws_settings_t {
    let mut s: ws_settings_t = unsafe { core::mem::zeroed() };
    s.endpoint = e_ws_endpoint_type_endpoint_client;
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
            libc::free(ptr.cast());
        }
    }
    *s = core::mem::zeroed();
}

// ── event names (C‑string literals) ──────────────────────────────────────────
const EVT_OPEN: &[u8] = b"open\0";
const EVT_CLOSE: &[u8] = b"close\0";
const EVT_FRAME: &[u8] = b"frame\0";
const EVT_ERROR: &[u8] = b"error\0";

// ── callback implementations ────────────────────────────────────────────────
unsafe extern "C" fn on_open(ctx: *mut c_void, fd: c_int, addr: *const c_char) {
    let peer = if addr.is_null() {
        "<null>"
    } else {
        CStr::from_ptr(addr).to_str().unwrap_or("<utf8 err>")
    };
    println!("[open] fd={fd} addr={peer}");

    // send "hello world!" immediately after connection
    let payload = b"hello world!";
    let frame = FRAME_CREATE.get().unwrap()(e_ws_frame_opcode_opcode_text);
    FRAME_PUSH.get().unwrap()(frame, payload.as_ptr(), payload.len());
    FRAME_EMIT.get().unwrap()(ctx, fd, frame);
    FRAME_DESTROY.get().unwrap()(frame);
}

unsafe extern "C" fn on_close(_ctx: *mut c_void, fd: c_int, status: e_ws_closure_status) {
    println!("[close] fd={fd} status={status}");
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
    let m = if msg.is_null() {
        "<null>"
    } else {
        CStr::from_ptr(msg).to_str().unwrap_or("<utf8 err>")
    };
    eprintln!("[error] {m}");
}

type ResultE<T> = Result<T, Box<dyn std::error::Error>>;

fn main() -> ResultE<()> {
    let lib = unsafe { Library::new(LIB_NAME) }?;

    unsafe {
        // required symbols
        let websocket_create: Symbol<unsafe extern "C" fn() -> *mut c_void> =
            lib.get(b"websocket_create\0")?;
        let websocket_destroy: Symbol<unsafe extern "C" fn(*mut c_void)> =
            lib.get(b"websocket_destroy\0")?;
        let websocket_setup: Symbol<
            unsafe extern "C" fn(*mut c_void, *const ws_settings_t) -> e_ws_status,
        > = lib.get(b"websocket_setup\0")?;
        let websocket_operate: Symbol<unsafe extern "C" fn(*mut c_void) -> bool> =
            lib.get(b"websocket_operate\0")?;
        let websocket_open: Symbol<
            unsafe extern "C" fn(
                *mut c_void,
                *const c_char,
                *const c_char,
                *mut c_int,
            ) -> e_ws_status,
        > = lib.get(b"websocket_open\0")?;
        let websocket_on: Symbol<
            unsafe extern "C" fn(*mut c_void, *const c_char, *mut c_void) -> e_ws_status,
        > = lib.get(b"websocket_on\0")?;

        // frame helpers
        FRAME_CREATE
            .set(*lib.get(b"websocket_frame_create\0")?)
            .ok();
        FRAME_PUSH.set(*lib.get(b"websocket_frame_push\0")?).ok();
        FRAME_EMIT.set(*lib.get(b"websocket_frame_emit\0")?).ok();
        FRAME_DESTROY
            .set(*lib.get(b"websocket_frame_destroy\0")?)
            .ok();

        // settings
        let mut settings = default_ws_settings();
        settings.host = CString::new("localhost:4433")?.into_raw();

        // context
        let ctx = websocket_create();
        if ctx.is_null() {
            eprintln!("websocket_create failed");
            destroy_ws_settings(&mut settings);
            return Ok(());
        }

        // register callbacks
        for (event, callback) in [
            (EVT_OPEN, on_open as *mut c_void),
            (EVT_CLOSE, on_close as *mut c_void),
            (EVT_FRAME, on_frame as *mut c_void),
            (EVT_ERROR, on_error as *mut c_void),
        ] {
            if websocket_on(ctx, event.as_ptr().cast(), callback) == e_ws_status_status_error {
                eprintln!("failed to register event {event:?}");
            }
        }

        // configure
        if websocket_setup(ctx, &settings) == e_ws_status_status_error {
            eprintln!("setup failed");
            websocket_destroy(ctx);
            destroy_ws_settings(&mut settings);
            return Ok(());
        }

        // connect to server
        let ret = websocket_open(
            ctx,
            b"localhost\0".as_ptr().cast(),
            b"4433\0".as_ptr().cast(),
            core::ptr::null_mut(),
        );
        if ret == e_ws_status_status_error {
            eprintln!("open failed");
            websocket_destroy(ctx);
            destroy_ws_settings(&mut settings);
            return Ok(());
        }

        println!("WebSocket client running (Rust)…  Ctrl+C to stop");
        while websocket_operate(ctx) {}

        websocket_destroy(ctx);
        destroy_ws_settings(&mut settings);
    }

    Ok(())
}
