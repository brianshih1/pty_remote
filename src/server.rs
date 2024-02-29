use std::ffi::CString;

use nix::{
    pty::forkpty,
    unistd::{execve, execvp},
};
use tokio::{
    fs::File,
    io::{AsyncReadExt, AsyncWriteExt},
    net::TcpListener,
};

pub async fn run_server() {
    let res = unsafe { forkpty(None, None) }.unwrap();
    match res.fork_result {
        nix::unistd::ForkResult::Parent { child: _ } => loop {
            let listener = TcpListener::bind("127.0.0.1:8080").await.unwrap();
            let (mut socket, _) = listener.accept().await.unwrap();
            println!("New TCP Connection Established");

            let mut master_reader = File::from(std::fs::File::from(res.master));
            let mut master_writer = master_reader.try_clone().await.unwrap();

            let (mut socket_reader, mut socket_writer) = socket.split();

            let mut buf = vec![0; 1024];
            let mut buf2 = vec![0; 1024];
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
                            let s = String::from_utf8_lossy(&buf2[..n]);
                            println!("Writing to socket: {}\n", s);
                            socket_writer.write(&buf2[..n]).await.unwrap();
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
