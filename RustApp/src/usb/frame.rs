use nusb::transfer::{Queue, RequestBuffer};
use std::pin::Pin;
use std::task::{ready, Context, Poll};
use std::{io, vec};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};

pub struct UsbStream {
    read_queue: Queue<RequestBuffer>,
    read_buffer: Vec<u8>,
    write_queue: Queue<Vec<u8>>,
    write_buffer: Vec<u8>,
}

const MAX_PACKET_SIZE: usize = 512;

impl UsbStream {
    pub fn new(read_queue: Queue<RequestBuffer>, write_queue: Queue<Vec<u8>>) -> Self {
        UsbStream {
            read_queue,
            read_buffer: Vec::with_capacity(MAX_PACKET_SIZE),
            write_queue,
            write_buffer: Vec::with_capacity(MAX_PACKET_SIZE),
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

        let (copy_from_buffer, remaining) = {
            let unfilled = buf.initialize_unfilled();

            // check if possible to read more
            if unfilled.is_empty() {
                return Poll::Pending;
            }

            // first copy from local buffer
            let copy_from_buffer = std::cmp::min(unfilled.len(), pin.read_buffer.len());

            if copy_from_buffer > 0 {
                unfilled[..copy_from_buffer].copy_from_slice(&pin.read_buffer[..copy_from_buffer]);
                pin.read_buffer.drain(..copy_from_buffer);
            }

            (copy_from_buffer, unfilled.len() - copy_from_buffer)
        };

        buf.advance(copy_from_buffer);

        // check if can request from remote
        if remaining > 0 {
            // make sure there's pending request
            if pin.read_queue.pending() == 0 {
                pin.read_queue.submit(RequestBuffer::new(MAX_PACKET_SIZE));
            }

            // try to read from the remote
            let res = ready!(pin.read_queue.poll_next(cx));

            // copy into the buffer
            match res.status {
                Ok(_) => {
                    // copy into poll buffer
                    let copy_from_buffer = {
                        let unfilled = buf.initialize_unfilled();

                        let copy_from_buffer = std::cmp::min(remaining, res.data.len());
                        unfilled[..copy_from_buffer].copy_from_slice(&res.data[..copy_from_buffer]);

                        copy_from_buffer
                    };
                    buf.advance(copy_from_buffer);
                    // copy the rest into local buffer
                    pin.read_buffer
                        .extend_from_slice(&res.data[copy_from_buffer..]);
                    // submit new request
                    pin.read_queue
                        .submit(RequestBuffer::reuse(res.data, MAX_PACKET_SIZE));
                    Poll::Ready(Ok(()))
                }
                Err(e) => Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, e))),
            }
        } else {
            Poll::Ready(Ok(()))
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

        // extend to local buffer
        pin.write_buffer.extend_from_slice(buf);
        let iter = pin.write_buffer.chunks_exact(MAX_PACKET_SIZE);
        let remainder = iter.remainder().to_vec();

        // send data by chunks
        let mut submitted = false;
        for chunk in iter {
            // if the chunk is full, submit to the queue
            pin.write_queue.submit(chunk.to_vec());
            submitted = true;
        }

        // fill the rest into the local buffer
        pin.write_buffer.clear();
        pin.write_buffer.extend_from_slice(&remainder);

        if !submitted {
            return Poll::Pending;
        }

        let res = ready!(pin.write_queue.poll_next(cx));

        match res.status {
            Ok(_) => Poll::Ready(Ok(res.data.actual_length())),
            Err(e) => Poll::Ready(Err(io::Error::new(io::ErrorKind::Other, e))),
        }
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        let pin = self.get_mut();

        let mut submitted = false;
        let iter = pin.write_buffer.chunks_exact(MAX_PACKET_SIZE);
        let remainder = iter.remainder();
        for chunk in iter {
            pin.write_queue.submit(chunk.to_vec());
            submitted = true;
        }

        // fill the rest into a whole packet padded with zeros
        if remainder.len() > 0 {
            let mut remainder_vec = vec![0; MAX_PACKET_SIZE];
            remainder_vec[..remainder.len()].copy_from_slice(remainder);
            pin.write_queue.submit(remainder_vec);
            submitted = true;
        }

        if submitted {
            let _ = ready!(pin.write_queue.poll_next(cx));
        }

        Poll::Ready(Ok(()))
    }

    fn poll_shutdown(self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Result<(), io::Error>> {
        let pin = self.get_mut();

        pin.write_queue.cancel_all();
        pin.read_queue.cancel_all();

        Poll::Ready(Ok(()))
    }
}
