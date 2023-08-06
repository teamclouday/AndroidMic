use std::net::UdpSocket;

fn main() {
    // Replace this with the port you want to bind to.
    let bind_port = 55555;

    // Create a UDP socket and bind it to the specified port
    let socket = UdpSocket::bind(("0.0.0.0", bind_port)).expect("Failed to bind to socket");

    // Buffer to store received data
    let mut buf = [0u8; 1024];

    println!("Waiting for data...");

    loop {
        // Receive data into the buffer
        match socket.recv_from(&mut buf) {
            Ok((size, src_addr)) => {
                let data = &buf[..size];
                let src_addr = src_addr.to_string();
                println!("Received {} bytes from {}: {:?}", size, src_addr, data);
            }
            Err(e) => {
                eprintln!("Error while receiving data: {:?}", e);
                break;
            }
        }
    }
}