// Client opens a TCP connection to server
// sends the stdin to TCP connection
// write data from the connection to stdout

use libc::sleep;
use std::{
    fs::read,
    io::{self, Read, Write},
    os::fd::{AsFd, AsRawFd},
    thread,
};
use tokio::signal;

use nix::{libc::dup2, pty::forkpty, unistd::close};
use std::ptr;
use termion::raw::IntoRawMode;

// We need to route the data from TcpStream to stdout
// We need to route stdin to the TcpStream
pub async fn run_client() {
    println!("I'm the client");
    let mut stream = std::net::TcpStream::connect("127.0.0.1:8080").unwrap();

    // stream.write("ls\n".as_bytes()).unwrap();
    // // let mut master_fd = -1;

    // // let mut buffer = String::new();
    // let mut stream = std::net::TcpStream::connect("127.0.0.1:8080").unwrap();
    let mut stream2 = stream.try_clone().unwrap();

    tokio::spawn(async move {
        // let mut stdout = std::io::stdout().into_raw_mode().unwrap();
        // io::copy(std::io::Read::by_ref(&mut stream2), stdout.by_ref()).unwrap();
        loop {
            let mut buf = vec![0; 1024];

            let n = stream2.read(&mut buf).unwrap();
            // println!("Read from socket: {:?} - {:?}", n, &buf);
            if n > 0 {
                // println!("Writing to client stdout");
                std::io::stdout().write(&buf[..n]).unwrap();
            }
        }
    });
    // let mut stdout = std::io::stdout().into_raw_mode().unwrap();

    io::copy(io::stdin().by_ref(), std::io::Write::by_ref(&mut stream)).unwrap();

    // let res = unsafe { forkpty(None, None) }.unwrap();
    // let master_fd = res.master.as_raw_fd();
    // match res.fork_result {
    //     nix::unistd::ForkResult::Parent { child } => {}
    //     nix::unistd::ForkResult::Child => {}
    // }
    println!("Ending Client");
}
