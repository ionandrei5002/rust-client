use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::{UnixStream,UnixListener};
use std::thread;
use std::ops::Add;

use serde::{Serialize, Deserialize};

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
                line = line.add("\n\n");
                let bytes_wrote: usize = write.write(line.as_bytes()).unwrap();
                println!("{}", bytes_wrote);
                write.flush();
            },
            _ => {}
        }
    }
}

fn remove_socket(path: &String) {
    std::fs::remove_file(path);
}

#[derive(Serialize, Deserialize, Debug)]
enum MsgTypes {
    register,
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    header: MsgTypes,
    value: String,
}

fn register_on_broker(path: &String, msg: &Message) {
    let client = UnixStream::connect(path).unwrap();
    let mut write_client = BufWriter::new(&client);

    let mut msg = serde_json::to_string(msg).unwrap();
    msg = msg.add("\n\n");
    println!("{}", &msg);
    write_client.write_all(msg.as_bytes());
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
