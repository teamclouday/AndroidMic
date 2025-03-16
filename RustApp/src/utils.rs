use std::path::PathBuf;

use cargo_packager_resource_resolver as resource_resolver;
use log::error;

use cached::proc_macro::cached;
use constcat::concat;

pub const APP_ID: &str = concat!(QUALIFIER, ".", ORG, ".", APP);

pub const QUALIFIER: &str = "io.github";
pub const ORG: &str = "teamclouday";
pub const APP: &str = "android-mic";

#[cached]
pub fn resource_dir(os_dependent: bool) -> PathBuf {
    if cfg!(ANDROID_MIC_FORMAT = "flatpak") {
        PathBuf::from(format!("/app/share/{APP_ID}/res"))
    } else {
        match resource_resolver::current_format() {
            Ok(format) => resource_resolver::resources_dir(format).unwrap(),
            Err(e) => {
                if matches!(e, resource_resolver::Error::UnkownPackageFormat) {
                    let path = std::fs::canonicalize("res").unwrap();
                    if os_dependent {
                        if cfg!(target_os = "linux") {
                            path.join("linux")
                        } else if cfg!(target_family = "windows") {
                            path.join("windows")
                        } else if cfg!(target_os = "macos") {
                            path.join("macos")
                        } else {
                            panic!("unsupported os")
                        }
                    } else {
                        path
                    }
                } else {
                    error!("{e}");
                    panic!()
                }
            }
        }
    }
}
