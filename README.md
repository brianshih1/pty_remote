## Building a Remote Terminal in Rust

The goal of this project is to build a simple client that can control a remote terminal in a server (similar to running `ssh` but without the encryption aspects).

Here is a quick demo:

https://github.com/brianshih1/pty_remote/assets/47339399/4dd83c7a-5235-4f1c-9cb0-85e871b7a64f

Here is what the Server and Client does under the hood:

**Remote Terminal Server**

- invoke `fork_pty` to create a new process operating in a pseudoterminal.
- execute `bash` in the child process. The `stdin`, `stdout`, and `stderr` of the child process is set to the `PTY slave`’s `fd` as a result of `fork_pty`(or `login_tty` under the hood).
- the parent process creates a `TCPListener` and accepts clients to connect
- after connecting to a client:
  - if data comes in via the TCP connection, copy it to the `PTY master`. `PTY master` will then forward to `PTY slave` which is also the `stdin` of the `bash` program.
  - since `stdout` of the `bash` program is connected to the `PTY slave`, the parent process reads data from `PTY master` and forwards the data to the client via `TCP`.

**Client**

- set the terminal to `raw mode` (to disable the `line discipline`)
- establish a TCP connection with the server
- copy the `stdin` to the connection
- copy data from the TCP connection to `stdout`.

**Running the project:**

After running `cargo build`,

to run the server: `./target/debug/pty_remote --server`

to run the client: `./target/debug/pty_remote`

**TODOs:**

The current implementation is quite hacky and I plan to improve it in the near term.

- Error handle better
- Deal with `SIGINT` & `exit` 
- Deal with client disconnecting
