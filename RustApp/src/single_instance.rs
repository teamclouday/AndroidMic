use byteorder::WriteBytesExt;
use futures::stream::Stream;
use interprocess::local_socket::traits::Stream as InterprocessStreamTrait;
use interprocess::local_socket::traits::tokio::Listener as TokioListener;
use interprocess::local_socket::{GenericNamespaced, ListenerOptions, ToNsName};
use interprocess::local_socket::{Name, Stream as InterprocessStream};

use tokio::io::AsyncReadExt;

use async_stream::stream;

#[derive(Debug, Clone)]
pub enum IpcEvent {
    Show,
}

impl TryFrom<u8> for IpcEvent {
    type Error = ();

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(IpcEvent::Show),
            _ => Err(()),
        }
    }
}

impl From<IpcEvent> for u8 {
    fn from(value: IpcEvent) -> Self {
        match value {
            IpcEvent::Show => 0,
        }
    }
}

fn get_name() -> anyhow::Result<Name<'static>> {
    let printname = "android-mic.sock";
    let name = printname.to_ns_name::<GenericNamespaced>()?;
    Ok(name)
}

pub fn stream() -> anyhow::Result<impl Stream<Item = IpcEvent>> {
    let name = get_name()?;
    let opts = ListenerOptions::new().name(name);
    let listener = opts.create_tokio()?;

    let stream = stream! {

        loop {
            match listener.accept().await {
                Ok(mut client) => {
                    loop {
                        match client.read_u8().await {
                            Ok(byte) => match byte.try_into() {
                                Ok(event) => yield event,
                                Err(_) => {
                                    error!("can't parse ipc event");
                                }
                            },
                            Err(e) => {
                                if e.kind() == std::io::ErrorKind::UnexpectedEof {
                                } else {
                                    error!("error reading client: {e}");
                                }
                                break;
                            }
                        }
                    }
                }
                Err(e) => {
                    error!("error accepting client: {e}");
                    tokio::time::sleep(std::time::Duration::from_millis(100)).await;
                }
            }
        }
    };

    Ok(stream)
}

pub fn send_event(event: IpcEvent) -> anyhow::Result<()> {
    let name = get_name()?;

    let mut stream = InterprocessStream::connect(name)?;

    stream.write_u8(event.into())?;

    Ok(())
}
