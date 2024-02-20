use std::io::{self, Write};

fn main() {
    let foo = std::net::TcpListener::bind("127.0.0.1:8080").unwrap();
    if let Ok((mut stream, addr)) = foo.accept() {
        io::copy(std::io::Read::by_ref(&mut stream), io::stdout().by_ref()).unwrap();
    }
}
