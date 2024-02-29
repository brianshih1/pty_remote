use std::{
    convert::Infallible,
    ffi::CString,
    io::{self, Read, Write},
    os::fd::AsRawFd,
    ptr::NonNull,
    sync::Arc,
    thread::spawn,
};

use nix::{
    errno::Errno,
    fcntl::{self, fcntl, FcntlArg, OFlag},
    pty::{forkpty, openpty},
    unistd::{execvp, read, write},
};
use std::ptr;
use tokio::{
    fs::File,
    io::{unix::AsyncFd, AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader, Interest},
    net::{TcpListener, TcpStream},
};

pub async fn run_server() {
    let res = unsafe { forkpty(None, None) }.unwrap();
    let master_fd = res.master.as_raw_fd();
    match res.fork_result {
        nix::unistd::ForkResult::Parent { child } => {
            let mut listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
            println!("Inside parent!");
            let (mut socket, _) = listener.accept().await.unwrap();
            println!("Listener accepted");

            // fcntl(socket.as_raw_fd(), FcntlArg::F_SETFL(OFlag::O_NONBLOCK)).unwrap();
            let mut master_reader = File::from(std::fs::File::from(res.master));
            let mut master_writer = master_reader.try_clone().await.unwrap();
            println!("kobe");

            let (mut socket_reader, mut socket_writer) = socket.split();

            socket_writer.write("foo".as_bytes()).await.unwrap();
            let mut buf = vec![0; 1024];
            let mut buf2 = vec![0; 1024];
            println!("YO?");
            loop {
                println!("Looping");

                tokio::select! {
                    Ok(n) = socket_reader.read(&mut buf) => {
                        println!("Reading");
                        if n == 0 {
                            println!("BREAK");
                            continue;
                        } else {
                            println!("Read from socket, writing into master: {}", n);
                            let s = String::from_utf8_lossy(&buf[..n]);
                            println!("Writing to master: {}", s);
                            master_writer.write(&mut buf[..n]).await.unwrap();
                        }
                    }
                    Ok(n) =  master_reader.read(&mut buf2) => {
                        println!("Writing");
                        if n == 0 {
                            println!("Breaking from loop");
                            continue;
                        } else {
                            println!("Writing N into socket: {}", n);
                            let s = String::from_utf8_lossy(&buf2[..n]);
                            println!("Writing to socket: {}", s);
                            socket_writer.write(&buf2[..n]).await.unwrap();
                        }
                    }
                }
            }
        }
        nix::unistd::ForkResult::Child => {
            let cstr = CString::new("/bin/bash").unwrap();
            execvp(&cstr, &[&cstr]).unwrap();
            println!("Child process - Finished bash");
            std::process::exit(1);
        }
    }
}
