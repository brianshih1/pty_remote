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
// use raw_tty::IntoRawMode;
use std::ptr;
use termion::raw::IntoRawMode as OtherIntoRawMode;

// We need to route the data from TcpStream to stdout
// We need to route stdin to the TcpStream
pub async fn run_client() {
    println!("I'm the client");
    // let mut stdout = std::io::stdout().into_raw_mode().unwrap();
    let mut stdout = std::io::stdout().into_raw_mode().unwrap();
    // let mut stdin = std::io::stdin().into_raw_mode().unwrap();
    // let mut stdin = std::io::stdin();

    let mut stream = std::net::TcpStream::connect("127.0.0.1:8080").unwrap();

    let mut stream2 = stream.try_clone().unwrap();
    tokio::spawn(async move {
        loop {
            let mut buf = vec![0; 1024];

            let n = stream2.read(&mut buf).unwrap();

            if n > 0 {
                // let s = String::from_utf8_lossy(&buf[..n]);

                // println!("Read data from socket: {}", s);
                stdout.write(&buf[..n]).unwrap();
                stdout.flush().unwrap();
            }
        }
    });

    io::copy(
        std::io::stdin().by_ref(),
        std::io::Write::by_ref(&mut stream),
    )
    .unwrap();

    println!("Ending Client");
}
