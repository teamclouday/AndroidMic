use byteorder::WriteBytesExt;
use futures::stream::Stream;
use interprocess::local_socket::tokio::Stream as InterprocessTokioStream;
use interprocess::local_socket::traits::Stream as InterprocessStreamTrait;
use interprocess::local_socket::traits::tokio::Listener as TokioListener;
use interprocess::local_socket::{GenericNamespaced, ListenerOptions, ToNsName};
use interprocess::local_socket::{Name, Stream as InterprocessStream};

use tokio::io::AsyncReadExt;

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

pub async fn create_stream() -> anyhow::Result<InterprocessTokioStream> {
    let name = get_name()?;

    let opts = ListenerOptions::new().name(name);

    let listener = opts.create_tokio()?;

    let stream = listener.accept().await?;
    Ok(stream)
}

pub fn parse_stream(stream: InterprocessTokioStream) -> impl Stream<Item = IpcEvent> {
    futures::stream::unfold(stream, |mut stream| async {
        match stream.read_u8().await {
            Ok(value) => match value.try_into() {
                Ok(event) => Some((event, stream)),
                Err(()) => {
                    error!("can't parse ipc event");
                    None
                }
            },
            Err(e) => {
                error!("{e}");
                None
            }
        }
    })
}

pub fn send_event(event: IpcEvent) -> anyhow::Result<()> {
    let name = get_name()?;

    let mut stream = InterprocessStream::connect(name)?;

    stream.write_u8(event.into())?;

    Ok(())
}
