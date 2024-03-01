use std::ffi::CString;

use nix::{pty::forkpty, unistd::execvp};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::mpsc::{Receiver, Sender},
};

async fn socket_listen(
    mut stream: TcpStream,
    mut bash_stdout_rx: Receiver<Vec<u8>>,
    sender: Sender<Vec<u8>>,
) {
    let (mut stream_reader, mut stream_writer) = stream.split();
    let mut buf = vec![0; 100];

    loop {
        tokio::select! {
            Ok(n) = stream_reader.read(&mut buf) => {
                let s = String::from_utf8_lossy(&buf[..n]);
                log::debug!("Read from TCP socket! Sending: {}", s);
                log::debug!("SLen: {}", s.len());
                log::debug!("Buf: {:?}", &buf[..n]);
                sender.send(Vec::from(&buf[..n])).await.unwrap();
            }
            Some(data) = bash_stdout_rx.recv() => {
                log::debug!("Writing to socket!");
                stream_writer.write(&data).await.unwrap();
                stream_writer.flush().await.unwrap();
            }
        }
    }
}

pub async fn run_server() {
    let res = unsafe { forkpty(None, None) }.unwrap();

    match res.fork_result {
        nix::unistd::ForkResult::Parent { child: _ } => loop {
            let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

            let mut master_reader = File::from(std::fs::File::from(res.master));
            let mut master_writer = master_reader.try_clone().await.unwrap();

            // Senders are the sockets
            // Reciever takes data from sockets (stdin from clients) and writes it to the PTY master
            // to propagate that to the PTY slave (bash program)
            let (sender, mut receiver) = tokio::sync::mpsc::channel::<Vec<u8>>(1048);

            let mut sockets: Vec<Sender<Vec<u8>>> = Vec::new();
            let mut buf = vec![0; 1024];

            let mut cache: Vec<u8> = vec![];

            // TODO: Deal with TCP connection disconnecting - need to remove it from sockets vec
            loop {
                let cache_ref = cache.clone();
                tokio::select! {
                    Ok((mut stream, _)) =  listener.accept() => {
                        log::debug!("Established new TCP Connection");
                        let sender = sender.clone();
                        let (_, mut stream_writer) = stream.split();
                        if cache_ref.len() > 0 {
                            stream_writer.write(&cache_ref.clone()).await.unwrap();
                        }
                        let (bash_stdout_sender, bash_stdout_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(100);
                        sockets.push(bash_stdout_sender);

                        tokio::spawn(async move {
                            socket_listen(stream, bash_stdout_rx, sender.clone()).await;
                        });
                    }
                    Some(data) = receiver.recv() => {
                        log::debug!("Writing into PTY master -> bash: {}", String::from_utf8_lossy(&data));
                        master_writer.write(&data).await.unwrap();
                        master_writer.flush().await.unwrap();
                    }
                    Ok(n) = master_reader.read(&mut buf) => {
                        log::debug!("Received stdout from bash, forwarding to sockets: {}", String::from_utf8_lossy(&buf[..n]));
                        println!("BUffer: {:?}", &buf[..n]);
                        println!("b: {:?}", n);


                        for sender in sockets.iter() {
                            log::debug!("send bash stdout to socket");
                            match sender.send(Vec::from(&buf[..n])).await {
                                Ok(_) => {},
                                Err(e) => {
                                    log::debug!("Error with sending: {}", e);
                                }
                            }
                        }

                        // This detects if it's an "enter". We want to preserve
                        // anything user types before they hit enter so that
                        // newly joined clients will receive the characters since
                        // the last "enter"
                        if n >= 2 && buf[0] == 13 && buf[1] == 10 {
                            cache = vec![];
                        } else {
                            cache.extend_from_slice(&buf[..n]);
                        }

                    }
                }
            }
        },
        nix::unistd::ForkResult::Child => {
            let cstr = CString::new("/bin/bash").unwrap();
            // TODO: Generate Bash Prompt, e.g.
            // /bin/bash --rcfile <(echo "PS1='MyCustomPrompt> '")
            execvp(&cstr, &[&cstr]).unwrap();
            std::process::exit(1);
        }
    }
}
