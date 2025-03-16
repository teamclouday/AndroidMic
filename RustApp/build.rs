use std::{env, io};

fn set_env(var_name: &str) {
    println!("cargo:rerun-if-env-changed={var_name}");

    if let Ok(var) = env::var(var_name) {
        println!("cargo:rustc-cfg={var_name}=\"{var}\"");
    }
}

fn main() -> io::Result<()> {
    #[cfg(target_os = "windows")]
    if env::var_os("CARGO_CFG_WINDOWS").is_some() && env::var("PROFILE").unwrap() == "release" {
        // https://github.com/mxre/winres/
        winres::WindowsResource::new()
            .set_icon("res/windows/app_icon.ico")
            .compile()?;
    }

    set_env("ANDROID_MIC_FORMAT");

    // build protobuf
    prost_build::Config::new()
        .out_dir("src/streamer")
        .compile_protos(&["src/proto/message.proto"], &["src/proto"])
        .expect("Failed to compile protobuf files");

    Ok(())
}
