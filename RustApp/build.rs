use std::{env, io};

fn set_env(var_name: &str) {
    println!("cargo:rerun-if-env-changed={var_name}");

    if let Ok(var) = env::var(var_name) {
        println!("cargo:rustc-cfg={var_name}=\"{var}\"");
    }
}

fn main() -> io::Result<()> {
    if env::var_os("CARGO_CFG_WINDOWS").is_some() && std::env::var("PROFILE").unwrap() == "release"
    {
        // https://github.com/mxre/winres/
        winres::WindowsResource::new()
            .set_icon("res/windows/app_icon.ico")
            .compile()?;
    }

    set_env("ANDROID_MIC_FORMAT");

    Ok(())
}
