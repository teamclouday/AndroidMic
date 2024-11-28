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
pub fn resource_dir() -> PathBuf {
    if cfg!(ANDROID_MIC_FORMAT = "flatpak") {
        PathBuf::from(format!("/app/share/{APP_ID}/res"))
    } else {
        match resource_resolver::current_format() {
            Ok(format) => resource_resolver::resources_dir(format).unwrap(),
            Err(e) => {
                if matches!(e, resource_resolver::Error::UnkownPackageFormat) {
                    std::fs::canonicalize("res").unwrap()
                    
                } else {
                    error!("{e}");
                    panic!()
                }
            }
        }
    }
}
