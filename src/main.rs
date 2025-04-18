#![allow(
    non_upper_case_globals,
    non_camel_case_types,
    non_snake_case,
    dead_code,
    unsafe_op_in_unsafe_fn
)]

// =============================================================
// Single source supporting **two roles** at compile‑time:
//   cargo run --features client   # WebSocket client
//   cargo run --features server   # WebSocket server
// =============================================================

#[cfg(all(not(feature = "client"), not(feature = "server")))]
compile_error!("Enable either the `client` or `server` feature (exactly one).");
#[cfg(all(feature = "client", feature = "server"))]
compile_error!("Features `client` and `server` are mutually exclusive.");

use std::env;
use std::path::PathBuf;
use libloading::{Library, Symbol};
use std::ffi::{CStr, CString};
use std::os::raw::{c_char, c_int, c_uchar, c_void};
use std::sync::OnceLock;

include!(concat!(env!("OUT_DIR"), "/bindings.rs"));

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

// ── defaults helper ───────────────────────────────────────────
fn default_ws_settings() -> ws_settings_t {
    let mut s: ws_settings_t = unsafe { core::mem::zeroed() };
    s.endpoint = if cfg!(feature = "client") {
        e_ws_endpoint_type_endpoint_client
    } else {
        e_ws_endpoint_type_endpoint_server
    };
    s.mode = e_ws_mode_mode_unsecured;
    s.ping_interval = 60_000;
    s.ping_timeout = 30_000;
    s.message_limit = 4 * 1024 * 1024;
    s.auto_mask_frame = cfg!(feature = "client");
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

// ── event C‑strings ───────────────────────────────────────────
const EVT_OPEN: &[u8] = b"open\0";
const EVT_CLOSE: &[u8] = b"close\0";
const EVT_FRAME: &[u8] = b"frame\0";
const EVT_ERROR: &[u8] = b"error\0";

// ── callbacks ─────────────────────────────────────────────────
unsafe extern "C" fn on_open(ctx: *mut c_void, fd: c_int, addr: *const c_char) {
    let peer = if addr.is_null() {
        "<null>"
    } else {
        CStr::from_ptr(addr).to_str().unwrap_or("<utf8 err>")
    };
    println!("[open] fd={fd} addr={peer}");

    if cfg!(feature = "client") {
        let payload = b"hello world!";
        let frame = FRAME_CREATE.get().unwrap()(e_ws_frame_opcode_opcode_text);
        FRAME_PUSH.get().unwrap()(frame, payload.as_ptr(), payload.len());
        FRAME_EMIT.get().unwrap()(ctx, fd, frame);
        FRAME_DESTROY.get().unwrap()(frame);
    }
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
    if opcode == e_ws_frame_opcode_opcode_text {
        match core::str::from_utf8(slice) {
            Ok(txt) => println!("[frame] fd={fd} text: {txt}"),
            Err(_) => println!("[frame] fd={fd} invalid UTF‑8 ({} bytes)", len),
        }
    } else {
        let hex: String = slice.iter().map(|b| format!("{:02X} ", b)).collect();
        println!("[frame] fd={fd} opcode={opcode:?} {len} bytes: {hex}");
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

// ── entry point ───────────────────────────────────────────────

type ResultE<T> = Result<T, Box<dyn std::error::Error>>;

fn get_library_path(lib_name: &str) -> Result<PathBuf, Box<dyn std::error::Error>> {
    let exe_path = env::current_exe()?;                  
    let exe_dir = exe_path.parent().ok_or("No exe dir")?;
    Ok(exe_dir.join(lib_name))                     
}

fn main() -> ResultE<()> {
    let lib_path = get_library_path(LIB_NAME)?;
	let lib = unsafe { Library::new(lib_path) }?;

    unsafe {
        // core API
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

        // settings & context
        let mut settings = default_ws_settings();
        settings.host = CString::new("localhost:4433")?.into_raw();

        let ctx = websocket_create();
        if ctx.is_null() {
            eprintln!("websocket_create failed");
            destroy_ws_settings(&mut settings);
            return Ok(());
        }

        // register callbacks
        for (event, cb) in [
            (EVT_OPEN, on_open as *mut c_void),
            (EVT_CLOSE, on_close as *mut c_void),
            (EVT_FRAME, on_frame as *mut c_void),
            (EVT_ERROR, on_error as *mut c_void),
        ] {
            if websocket_on(ctx, event.as_ptr().cast(), cb) == e_ws_status_status_error {
                eprintln!("failed to register {event:?}");
            }
        }

        // apply settings
        if websocket_setup(ctx, &settings) == e_ws_status_status_error {
            eprintln!("setup failed");
            websocket_destroy(ctx);
            destroy_ws_settings(&mut settings);
            return Ok(());
        }

        // bind (server) or open (client)
        let rc = if cfg!(feature = "client") {
            websocket_open(
                ctx,
                b"localhost\0".as_ptr() as *const c_char,
                b"4433\0".as_ptr() as *const c_char,
                core::ptr::null_mut(),
            )
        } else {
            websocket_bind(
                ctx,
                b"localhost\0".as_ptr() as *const c_char,
                b"4433\0".as_ptr() as *const c_char,
                core::ptr::null_mut(),
            )
        };

        if rc == e_ws_status_status_error {
            eprintln!(
                "{}",
                if cfg!(feature = "client") {
                    "open failed"
                } else {
                    "bind failed"
                }
            );
            websocket_destroy(ctx);
            destroy_ws_settings(&mut settings);
            return Ok(());
        }

        println!(
            "WebSocket {} running (Rust)…  Ctrl+C to stop",
            if cfg!(feature = "client") {
                "client"
            } else {
                "server"
            }
        );

        while websocket_operate(ctx) {}

        websocket_destroy(ctx);
        destroy_ws_settings(&mut settings);
    }

    Ok(())
}
