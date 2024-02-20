// Client opens a TCP connection to server
// sends the stdin to TCP connection
// write data from the connection to stdout

use std::{
    fs::read,
    io::{self, Read, Write},
    thread,
};

fn main() {
    let mut buffer = String::new();
    let mut stream = std::net::TcpStream::connect("127.0.0.1:8080").unwrap();
    io::copy(io::stdin().by_ref(), std::io::Write::by_ref(&mut stream)).unwrap();
    // if let Ok(foo) = io::stdin().read_line(&mut buffer) {
    //     println!("Buffer: {}", buffer);
    // }
    // match std::net::TcpStream::connect("addr") {
    //     Ok(c) => {
    //         thread::spawn(move || {

    //         });
    //     }
    //     Err(e) => {
    //         println!("Failed to bind to Tcp: {:?}", e);
    //     }
    // }
}
