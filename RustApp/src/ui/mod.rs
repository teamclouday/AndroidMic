pub mod app;
mod icon;
mod message;

#[cfg(not(target_os = "linux"))]
mod tray;
mod view;
mod wave;
