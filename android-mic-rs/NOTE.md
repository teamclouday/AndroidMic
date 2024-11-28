.\adb.exe reverse tcp:6000 tcp:6000

.\adb.exe reverse --remove-all


streamer_sub:

let mut streamer = None;
loop {

    if let Some(command) = rx.recv() {

        match command {

            Connect(mode, buff) => {

                match mode {

                    Adb => {
                        streamer = adb::new(buff, sender.clone());
                    }
                }

            }

            Stop => {
                streamer = None;
            }

            Buff(buff) => {
                streamer.set_buff(buff)
            }
        }
        
    }

}


tcp:

new(buff, sender) {

    bind_port;

    let (tx, rx) = mpsc::chanel();

    task::spawn(
        listen(listener, sender.clone(), tx)
    )

    task::spawn(
        process(buff, sender.clone(), rx)
    )

    TcpStreamer {
        ip,
        port: addr.port(),
        tx
    }

}


fn listen(listener, buff, sender.clone(), rx) {

    let stream = listen.accept;


}


tcp_streamer:

new(buff,)