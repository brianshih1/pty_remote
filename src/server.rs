use std::{
    ffi::CString,
    os::fd::{AsRawFd, FromRawFd, OwnedFd},
    sync::mpsc,
};

use nix::{
    pty::forkpty,
    unistd::{execve, execvp},
};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    net::{tcp::ReadHalf, TcpListener, TcpStream},
    sync::mpsc::Sender,
};

struct Socket {
    stream: TcpStream,
    sender: Sender<Vec<u8>>,
}

impl Socket {
    pub async fn listen(&mut self) {
        let mut reader = self.stream.split().0;
        let mut buf = vec![0; 1024];

        loop {
            if let Ok(n) = reader.read(&mut buf).await {
                if n > 0 {
                    self.sender.send(Vec::from(&buf[..n])).await.unwrap();
                }
            }
        }
    }
}

pub async fn run_server() {
    dbg!("foo");

    let res = unsafe { forkpty(None, None) }.unwrap();

    match res.fork_result {
        nix::unistd::ForkResult::Parent { child: _ } => loop {
            let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();

            println!("New TCP Connection Established");

            let mut master_reader = File::from(std::fs::File::from(res.master));
            let mut master_writer = master_reader.try_clone().await.unwrap();

            // Senders are the sockets
            // Reciever takes data from sockets (stdin from clients) and writes it to the PTY master
            // to propagate that to the PTY slave (bash program)
            let (sender, mut receiver) = tokio::sync::mpsc::channel::<Vec<u8>>(1048);

            let mut sockets: Vec<Sender<Vec<u8>>> = Vec::new();
            let mut buf = vec![0; 1024];
            let mut buf2 = vec![0; 1024];
            loop {
                println!("Looping");
                tokio::select! {
                    Ok((mut stream, _)) =  listener.accept() => {
                        println!("Received socket");
                        let sender = sender.clone();
                        let (bash_stdout_sender, mut bash_stdout_rx) = tokio::sync::mpsc::channel::<Vec<u8>>(100);
                        sockets.push(bash_stdout_sender);
                        tokio::spawn(async move {
                            let (mut stream_reader, mut stream_writer) = stream.split();
                            let mut buf = vec![0; 1024];
                            loop {
                                tokio::select! {
                                    Ok(n) = stream_reader.read(&mut buf) => {
                                        let s = String::from_utf8_lossy(&buf[..n]);
                                        println!("Read from TCP socket! Sending: {}", s);
                                        sender.send(Vec::from(&buf[..n])).await.unwrap();
                                    }
                                    Some(data) = bash_stdout_rx.recv() => {
                                        println!("Writing to socket!");
                                        stream_writer.write(&data).await.unwrap();
                                    }
                                }
                            }
                        });
                    }
                    Some(foo) = receiver.recv() => {
                        let s = String::from_utf8_lossy(&foo);
                        println!("Writing into PTY master -> bash: {}", s);
                        master_writer.write(&foo).await.unwrap();
                        master_writer.flush().await.unwrap();
                    }
                    Ok(n) =  master_reader.read(&mut buf2) => {
                        println!("Received stdout from bash, forwarding to sockets");
                        for sender in sockets.iter() {
                            println!("send bash stdout to socket");
                            sender.send(Vec::from(&buf2[..n])).await.unwrap();
                        }
                    }
                }
            }
        },
        nix::unistd::ForkResult::Child => {
            let echo = CString::new("echo").unwrap();
            let param = CString::new("\"PS1='MyCustomPrompt> '\"").unwrap();

            let command = CString::new("--rcfile <(echo \"PS1='MyCustomPrompt> '\")").unwrap();
            let rc_file = CString::new("--rcfile").unwrap();
            let lol = CString::new("<(echo \"PS1='MyCustomPrompt> '\")").unwrap();

            let cstr = CString::new("/bin/bash").unwrap();
            execvp(&cstr, &[&cstr]).unwrap();
            println!("Child process - Finished bash");
            std::process::exit(1);
        }
    }
}
