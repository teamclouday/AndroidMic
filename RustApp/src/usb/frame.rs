use nusb::transfer::{Queue, RequestBuffer};
use std::io;
use std::pin::Pin;
use std::task::{ready, Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub struct UsbStream {
    read_queue: Queue<RequestBuffer>,
    write_queue: Queue<Vec<u8>>,
}

const MAX_PACKET_SIZE: usize = 512;

impl UsbStream {
    pub fn new(read_queue: Queue<RequestBuffer>, write_queue: Queue<Vec<u8>>) -> Self {
        UsbStream {
            read_queue,
            write_queue,
        }
    }
}

impl AsyncRead for UsbStream {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let pin = self.get_mut();
        // make sure there's pending request
        if pin.read_queue.pending() == 0 {
            pin.read_queue.submit(RequestBuffer::new(MAX_PACKET_SIZE));
        }

        // try to read from the buffer
        let mut res = ready!(pin.read_queue.poll_next(cx));
        // copy into the buffer
        match res.status {
            Ok(_) => {
                buf.put_slice(res.data.as_mut_slice());
                // submit new request
                pin.read_queue
                    .submit(RequestBuffer::reuse(res.data, MAX_PACKET_SIZE));
                Poll::Ready(Ok(()))
            }
            Err(e) => Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, e))),
        }
    }
}

impl AsyncWrite for UsbStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        let pin = self.get_mut();

        // send data by chunks
        for chunk in buf.chunks(MAX_PACKET_SIZE) {
            // create a vector of MAX_PACKET_SIZE size and fill the rest with 0
            let mut data = vec![0; MAX_PACKET_SIZE];
            data.extend_from_slice(chunk);
            pin.write_queue.submit(data);
        }

        let res = ready!(pin.write_queue.poll_next(cx));

        match res.status {
            Ok(_) => Poll::Ready(Ok(res.data.actual_length())),
            Err(e) => Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, e))),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        let pin = self.get_mut();

        pin.write_queue.cancel_all();
        pin.read_queue.cancel_all();

        Poll::Ready(Ok(()))
    }
}
