use std::{env, path::PathBuf};

fn main() {
    println!("cargo:rerun-if-changed=submodule/Websocket/websocket/include/websocket/api/websocket_c_api.h");
    println!("cargo:rerun-if-changed=build.rs");

    let bindings = bindgen::Builder::default()
        .header("submodule/Websocket/websocket/include/websocket/api/websocket_c_api.h")
        .derive_default(true)
        .clang_arg("-xc++")
        .clang_arg("-std=c++17")
        .clang_arg("-DWEBSOCKET_C_API")
        .clang_arg("-DWEBSOCKET_API=")
        .clang_arg("-Isubmodule/Websocket/websocket/include")
        .generate()
        .expect("Unable to generate bindings");

    let out_path = PathBuf::from(env::var("OUT_DIR").unwrap());
    bindings
        .write_to_file(out_path.join("bindings.rs"))
        .expect("Couldn't write bindings!");
}
