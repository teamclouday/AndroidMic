#[cfg(target_os = "windows")]
pub use windows::start_at_login;

#[cfg(target_os = "linux")]
pub use linux::start_at_login;

#[cfg(target_os = "macos")]
pub use macos::start_at_login;

#[cfg(target_os = "windows")]
mod windows {
    use std::{env, fs, path::Path};

    use directories::BaseDirs;
    use zconf::ConfigManager;

    use crate::config::Config;

    fn create_shortcut(lnk: &Path) -> anyhow::Result<()> {
        let target = env::current_exe()?;

        let mut shell_link = mslnk::ShellLink::new(target)?;
        shell_link.set_name(Some(String::from("AndroidMic")));
        shell_link.set_arguments(Some(String::from("--launched-automatically")));
        shell_link.create_lnk(lnk)?;

        Ok(())
    }

    fn remove_shortcut(lnk: &Path) -> anyhow::Result<()> {
        fs::remove_file(lnk)?;
        Ok(())
    }

    pub fn start_at_login(start_at_login: bool, config: &mut ConfigManager<Config>) {
        let dirs = BaseDirs::new().unwrap();
        let file_path = dirs
            .data_dir()
            .join("Microsoft/Windows/Start Menu/Programs/Startup/AndroidMic.lnk");

        if start_at_login {
            if let Err(e) = create_shortcut(&file_path) {
                error!("can't create shortcut: {e}");
                config.update(|s| s.start_at_login = false);
            } else {
                config.update(|s| s.start_at_login = true);
            }
        } else {
            if let Err(e) = remove_shortcut(&file_path) {
                error!("can't remove shortcut: {e}");
            }
            config.update(|s| s.start_at_login = false);
        }
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use crate::config::Config;
    use zconf::ConfigManager;

    pub fn start_at_login(_start_at_login: bool, _config: &mut ConfigManager<Config>) {
        todo!()
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use crate::config::Config;
    use zconf::ConfigManager;

    pub fn start_at_login(_start_at_login: bool, _config: &mut ConfigManager<Config>) {
        todo!()
    }
}
