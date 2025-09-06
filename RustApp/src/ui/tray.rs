use cosmic::iced::{futures::Stream, stream};
use futures::SinkExt;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

#[cfg(not(target_os = "linux"))]
use tray_icon::{
    TrayIcon, TrayIconBuilder, TrayIconEvent,
    menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
};

use crate::{fl, tray_icon};

#[derive(Debug, Clone)]
pub enum SystemTrayMsg {
    Show,
    Connect,
    Disconnect,
    Exit,
}

#[derive(Clone)]
pub struct SystemTrayStream {
    receiver: Arc<Mutex<mpsc::UnboundedReceiver<SystemTrayMsg>>>,
}

#[cfg(not(target_os = "linux"))]
pub struct SystemTray {
    tray_icon: TrayIcon,
    item_connect: MenuItem,
    item_disconnect: MenuItem,
}

#[cfg(target_os = "linux")]
pub struct SystemTray;

impl SystemTray {
    #[cfg(not(target_os = "linux"))]
    pub fn new() -> anyhow::Result<(Self, SystemTrayStream)> {
        let item_show = MenuItem::new(fl!("tray_show_window"), true, None);
        let item_connect = MenuItem::new(fl!("tray_connect"), true, None);
        let item_disconnect = MenuItem::new(fl!("tray_disconnect"), true, None);
        let item_exit = MenuItem::new(fl!("tray_exit"), true, None);

        let item_show_id = item_show.id().clone();
        let item_connect_id = item_connect.id().clone();
        let item_disconnect_id = item_disconnect.id().clone();
        let item_exit_id = item_exit.id().clone();

        let menu = Menu::with_items(&[
            &item_show,
            &item_connect,
            &item_disconnect,
            &PredefinedMenuItem::separator(),
            &item_exit,
        ])?;

        let tray_icon = TrayIconBuilder::new()
            .with_menu(Box::new(menu))
            .with_tooltip("AndroidMic")
            .with_icon(tray_icon!("icon")?)
            .build()?;

        // set up event channel
        let (sender, receiver) = mpsc::unbounded_channel();

        let menu_sender = sender.clone();
        MenuEvent::set_event_handler(Some(move |event: MenuEvent| {
            let _ = match event.id {
                id if id == item_show_id => menu_sender.send(SystemTrayMsg::Show),
                id if id == item_connect_id => menu_sender.send(SystemTrayMsg::Connect),
                id if id == item_disconnect_id => menu_sender.send(SystemTrayMsg::Disconnect),
                id if id == item_exit_id => menu_sender.send(SystemTrayMsg::Exit),
                _ => return,
            };
        }));

        let tray_sender = sender.clone();
        TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| match event {
            TrayIconEvent::DoubleClick { .. } => {
                let _ = tray_sender.send(SystemTrayMsg::Show);
            }
            _ => {}
        }));

        Ok((
            Self {
                tray_icon,
                item_connect,
                item_disconnect,
            },
            SystemTrayStream {
                receiver: Arc::new(Mutex::new(receiver)),
            },
        ))
    }

    #[cfg(target_os = "linux")]
    pub fn new() -> anyhow::Result<(Self, SystemTrayStream)> {
        // placeholder here
        let (_sender, receiver) = mpsc::unbounded_channel();
        Ok((
            Self {},
            SystemTrayStream {
                receiver: Arc::new(Mutex::new(receiver)),
            },
        ))
    }

    #[cfg(not(target_os = "linux"))]
    pub fn update_menu_state(&mut self, disconnected: bool, status: &str) {
        // update menu item states
        self.item_connect.set_enabled(disconnected);
        self.item_disconnect.set_enabled(!disconnected);
        let _ = self
            .tray_icon
            .set_tooltip(Some(format!("AndroidMic - {}", status)))
            .map_err(|e| {
                error!("failed to set tray icon tooltip: {e}");
            });
    }

    #[cfg(target_os = "linux")]
    pub fn update_menu_state(&mut self, _disconnected: bool, _status: &str) {
        // placeholder here
    }
}

impl SystemTrayStream {
    pub fn sub(self) -> impl Stream<Item = SystemTrayMsg> {
        let receiver_arc = self.receiver.clone();

        stream::channel(1, |mut sender| async move {
            loop {
                let mut receiver = receiver_arc.lock().await;
                if let Some(msg) = receiver.recv().await {
                    if sender.send(msg).await.is_err() {
                        break;
                    }
                } else {
                    break;
                }
            }
        })
    }
}
