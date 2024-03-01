// Client opens a TCP connection to server
// sends the stdin to TCP connection
// write data from the connection to stdout

use std::io::{self, Read, Write};

use termion::raw::IntoRawMode as OtherIntoRawMode;

// We need to route the data from TcpStream to stdout
// We need to route stdin to the TcpStream
pub async fn run_client() {
    let mut stdout = std::io::stdout().into_raw_mode().unwrap();

    let mut stream = std::net::TcpStream::connect("127.0.0.1:8080").unwrap();

    let mut stream2 = stream.try_clone().unwrap();
    tokio::spawn(async move {
        loop {
            let mut buf = vec![0; 1024];

            let n = stream2.read(&mut buf).unwrap();

            if n > 0 {
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
