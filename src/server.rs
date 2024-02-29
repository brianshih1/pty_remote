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
    io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader},
    net::{TcpListener, TcpStream},
};

pub async fn run_server() {
    let res = unsafe { forkpty(None, None) }.unwrap();
    let master_fd = res.master.as_raw_fd();
    match res.fork_result {
        nix::unistd::ForkResult::Parent { child } => {
            let mut listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
            println!("Inside parent!");

            // loop {
            let (mut socket, _) = listener.accept().await.unwrap();
            println!("Listener accepted");

            fcntl(socket.as_raw_fd(), FcntlArg::F_SETFL(OFlag::O_NONBLOCK)).unwrap();

            let (mut socket_reader, mut socket_writer) = socket.split();

            fcntl(master_fd, FcntlArg::F_SETFL(OFlag::O_NONBLOCK)).unwrap();

            loop {
                println!("Inside second loop");
                let mut buf = vec![0; 1024];
                let n = socket_reader
                    .read(&mut buf)
                    .await
                    .expect("failed to read data from socket");

                if n == 0 {
                    break;
                } else {
                    println!("Read from socket, writing into master: {}", n);
                }
                println!("Buffer: {:?}", &buf[..n]);
                // write to master fd
                write(res.master.as_raw_fd(), &buf[..n]).unwrap();
            }
            // loop {
            //     println!("Big loop");
            //     // Read from master and forward to socket.
            //     // What's read from master is the stdout from the shell program.
            //     loop {
            //         let mut buf = vec![0; 1024];

            //         match read(master_fd, &mut buf) {
            //             Ok(n) => {
            //                 if n == 0 {
            //                     println!("Breaking from loop");
            //                     break;
            //                 } else {
            //                     println!("Read N: {}", n);
            //                     socket_writer.write(&buf[..n]).await.unwrap();
            //                 }
            //             }
            //             Err(e) => {
            //                 if e != Errno::EAGAIN {
            //                     panic!("Unexpected Error reading: {}", e);
            //                 }
            //             }
            //         };
            //     }

            //     // In a loop, read data from the socket and write the data back.
            //     // What's written to the socket will be routed to stdin of the shell
            //     // program.
            //     loop {
            //         println!("Inside second loop");
            //         let mut buf = vec![0; 1024];
            //         let n = socket_reader
            //             .read(&mut buf)
            //             .await
            //             .expect("failed to read data from socket");

            //         if n == 0 {
            //             break;
            //         } else {
            //             println!("Read from socket, writing into master: {}", n);
            //         }
            //         println!("Buffer: {:?}", &buf[..n]);
            //         // write to master fd
            //         write(res.master.as_raw_fd(), &buf[..n]).unwrap();
            //     }
            // }
            // }
        }
        nix::unistd::ForkResult::Child => {
            let cstr = CString::new("/bin/bash").unwrap();
            execvp(&cstr, &[&cstr]).unwrap();
            std::process::exit(1);
        }
    }
}
