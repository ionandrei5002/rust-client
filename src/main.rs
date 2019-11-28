use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::{UnixStream,UnixListener};
use std::thread;
use std::ops::Add;
use std::process::Command;

use serde::{Serialize, Deserialize};
use std::thread::sleep;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug)]
enum MsgTypes {
    register,
    ok,
    command,
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    header: MsgTypes,
    value: String,
}

fn handle_client(stream: UnixStream) {
    let read = BufReader::new(&stream);
    let mut write = BufWriter::new(&stream);

    for res in read.lines() {
        match res {
            Ok(mut line) => {
                if line.len() == 0 {
                    break;
                }
                println!("{}", line);
                let msg = serde_json::from_str(&line.as_str());
                let msg: Message = match msg {
                    Ok(msg) => {
                        msg
                    },
                    _ => {
                        println!("{}", "Bad Message!");
                        return;
                    }
                };

                match msg.header {
                    MsgTypes::register => {},
                    _ => {
                        let parts = msg.value.split_whitespace().collect::<Vec<_>>();

                        let mut app = Command::new(parts[0]);
                        for part in 1..parts.len() {
                            app.arg(parts[part]);
                        }
                        let output = app.output().unwrap();

                        println!("{}", String::from_utf8_lossy(output.stdout.as_slice()));
                        write.write(output.stdout.as_slice());
                        write.write("\n".as_bytes());
                        write.flush();
                    }
                }
            },
            _ => {}
        }
    }
}

fn remove_socket(path: &String) {
    std::fs::remove_file(path);
}

fn register_on_broker(path: &String, msg: &Message) {
    loop {
        let client = match UnixStream::connect(path) {
            Ok(client) => {
                let mut write_client = BufWriter::new(&client);

                let mut msg = serde_json::to_string(msg).unwrap();
                msg = msg.add("\n");
                println!("{}", &msg);
                write_client.write_all(msg.as_bytes());
                break;
            },
            _ => {
                println!("Waiting for broker!");
                sleep(Duration::from_secs(1));
            },
        };
    }
}

fn main() {
    let socket = String::from("/tmp/app-uds.sock");
    let broker = String::from("/tmp/rust-uds.sock");

    remove_socket(&socket);
    let server = UnixListener::bind(&socket).unwrap();

    let msg = Message {header: MsgTypes::register, value: socket.clone()};
    register_on_broker(&broker, &msg);

    for stream in server.incoming() {
        match stream {
            Ok(stream) => {
                thread::spawn(|| handle_client(stream));
            }
            Err(err) => {
                println!("Error: {}", err);
                break;
            }
        }
    }
}
