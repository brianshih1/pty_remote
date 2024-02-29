use clap::{arg, command, Parser};

use crate::client::run_client;
use crate::server::run_server;

mod client;
mod server;

// Server: ./target/debug/pty_remote -s
// Client: ./target/debug/pty_remote

#[tokio::main]
async fn main() {
    let matches = command!()
        .arg(arg!(
            -s --server ... "Runs as server"
        ))
        .get_matches();
    if let Some(count) = matches.get_one::<u8>("server") {
        if *count == 0 {
            println!("Running client");
            run_client().await;
        } else {
            println!("Running server");
            run_server().await;
        }
    }
    println!("End of main!");
}
