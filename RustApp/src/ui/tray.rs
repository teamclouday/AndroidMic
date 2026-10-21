use cosmic::iced::{futures::Stream, stream};
use futures::SinkExt;
use std::sync::Arc;
use tokio::sync::{Mutex, mpsc};

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
mod tray_icon_backend {
    use futures::SinkExt;
    use std::sync::Arc;
    use tokio::sync::{Mutex, mpsc};
    use tray_icon::{
        TrayIcon, TrayIconBuilder, TrayIconEvent,
        menu::{Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    };

    use crate::{fl, tray_icon};

    use super::{SystemTrayMsg, SystemTrayStream};

    pub struct SystemTray {
        tray_icon: TrayIcon,
        item_connect: MenuItem,
        item_disconnect: MenuItem,
    }

    impl SystemTray {
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
            TrayIconEvent::set_event_handler(Some(move |event: TrayIconEvent| {
                if let TrayIconEvent::DoubleClick { .. } = event {
                    let _ = tray_sender.send(SystemTrayMsg::Show);
                }
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
    }
}

#[cfg(not(target_os = "linux"))]
pub use tray_icon_backend::SystemTray;

#[cfg(target_os = "linux")]
mod ksni_backend {
    use std::sync::Arc;
    use tokio::sync::{Mutex, mpsc};
    use ksni::TrayMethods;
    use ksni::menu::{MenuItem as MenuEntry, StandardItem};

    use crate::{fl, utils::APP_ID};

    use super::{SystemTrayMsg, SystemTrayStream};

    enum TrayCommand {
        MenuState { disconnected: bool, status: String },
    }

    struct KsniTray {
        event_sender: mpsc::UnboundedSender<SystemTrayMsg>,
        connected: bool,
        description: String,
    }

    impl KsniTray {
        fn notify(&self, msg: SystemTrayMsg) {
            let _ = self.event_sender.send(msg);
        }

        fn apply(&mut self, cmd: TrayCommand) {
            match cmd {
                TrayCommand::MenuState { disconnected, status } => {
                    self.connected = !disconnected;
                    self.description = format!("AndroidMic - {status}");
                }
            }
        }

        fn icon_pixmap() -> Vec<ksni::Icon> {
            let svg = include_bytes!("../../res/icons/icon.svg");

            [32, 64]
                .into_iter()
                .filter_map(|size| {
                    let opt = resvg::usvg::Options::default();
                    let tree = resvg::usvg::Tree::from_data(svg, &opt).ok()?;
                    let viewbox = tree.size();

                    let mut pixmap = resvg::tiny_skia::Pixmap::new(size, size)?;
                    resvg::render(
                        &tree,
                        resvg::tiny_skia::Transform::from_scale(
                            size as f32 / viewbox.width(),
                            size as f32 / viewbox.height(),
                        ),
                        &mut pixmap.as_mut(),
                    );

                    let mut data = pixmap.data().to_vec();
                    // RGBA to ARGB32, network byte order
                    for pixel in data.chunks_exact_mut(4) {
                        pixel.rotate_right(1);
                    }

                    Some(ksni::Icon {
                        width: size as i32,
                        height: size as i32,
                        data,
                    })
                })
                .collect()
        }
    }

    impl ksni::Tray for KsniTray {
        fn id(&self) -> String {
            APP_ID.into()
        }

        fn title(&self) -> String {
            "AndroidMic".into()
        }

        fn status(&self) -> ksni::Status {
            ksni::Status::Active
        }

        fn icon_pixmap(&self) -> Vec<ksni::Icon> {
            Self::icon_pixmap()
        }

        fn tool_tip(&self) -> ksni::ToolTip {
            ksni::ToolTip {
                icon_name: String::new(),
                icon_pixmap: Self::icon_pixmap(),
                title: "AndroidMic".into(),
                description: self.description.clone(),
            }
        }

        fn activate(&mut self, _x: i32, _y: i32) {
            self.notify(SystemTrayMsg::Show);
        }

        fn menu(&self) -> Vec<MenuEntry<Self>> {
            vec![
                StandardItem {
                    label: fl!("tray_show_window"),
                    activate: Box::new(|this: &mut Self| this.notify(SystemTrayMsg::Show)),
                    ..Default::default()
                }
                .into(),
                StandardItem {
                    label: fl!("tray_connect"),
                    enabled: self.connected,
                    activate: Box::new(|this: &mut Self| this.notify(SystemTrayMsg::Connect)),
                    ..Default::default()
                }
                .into(),
                StandardItem {
                    label: fl!("tray_disconnect"),
                    enabled: !self.connected,
                    activate: Box::new(|this: &mut Self| this.notify(SystemTrayMsg::Disconnect)),
                    ..Default::default()
                }
                .into(),
                MenuEntry::Separator,
                StandardItem {
                    label: fl!("tray_exit"),
                    activate: Box::new(|this: &mut Self| this.notify(SystemTrayMsg::Exit)),
                    ..Default::default()
                }
                .into(),
            ]
        }
    }

    pub struct SystemTray {
        state_sender: mpsc::UnboundedSender<TrayCommand>,
    }

    impl SystemTray {
        pub fn new() -> anyhow::Result<(Self, SystemTrayStream)> {
            let (event_sender, event_receiver) = mpsc::unbounded_channel();
            let (state_sender, mut state_receiver) = mpsc::unbounded_channel();

            // run the tray service on a dedicated thread with its own runtime
            // so dbus calls never block the UI event loop
            std::thread::Builder::new()
                .name("tray".into())
                .spawn(move || {
                    let rt = match tokio::runtime::Builder::new_current_thread()
                        .enable_all()
                        .build()
                    {
                        Ok(rt) => rt,
                        Err(e) => {
                            error!("failed to create tray runtime: {e}");
                            return;
                        }
                    };

                    rt.block_on(async move {
                        let tray = KsniTray {
                            event_sender,
                            connected: true,
                            description: "AndroidMic".into(),
                        };

                        let handle = match tray.spawn().await {
                            Ok(handle) => handle,
                            Err(e) => {
                                error!("failed to create system tray: {e}");
                                return;
                            }
                        };

                        while let Some(cmd) = state_receiver.recv().await {
                            handle.update(|tray: &mut KsniTray| tray.apply(cmd)).await;
                        }

                        handle.shutdown().await;
                    });
                })
                .map_err(|e| anyhow::anyhow!("failed to spawn tray thread: {e}"))?;

            Ok((
                Self { state_sender },
                SystemTrayStream {
                    receiver: Arc::new(Mutex::new(event_receiver)),
                },
            ))
        }

        pub fn update_menu_state(&mut self, disconnected: bool, status: &str) {
            let _ = self.state_sender.send(TrayCommand::MenuState {
                disconnected,
                status: status.to_string(),
            });
        }
    }
}

#[cfg(target_os = "linux")]
pub use ksni_backend::SystemTray;

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
