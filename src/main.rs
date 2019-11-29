use std::io::{BufReader, BufWriter, Write, Read};
use std::os::unix::net::{UnixStream,UnixListener};
use std::thread;
use std::process::Command;

use serde::{Serialize, Deserialize};
use std::thread::sleep;
use std::time::Duration;
use byteorder::{WriteBytesExt, BigEndian, ReadBytesExt};

#[derive(Serialize, Deserialize, Debug)]
enum MsgTypes {
    Register,
    Ok,
    Command,
    Close,
}

#[derive(Serialize, Deserialize, Debug)]
struct Message {
    header: MsgTypes,
    value: String,
}

fn read_message(stream: &UnixStream) -> String {
    let mut read = BufReader::new(stream);
    let mut msg_size = read.read_u64::<BigEndian>().unwrap() as usize;
    let mut line = String::from("");
    line.reserve(msg_size);

    while msg_size > 0 {
        let mut buffer = [0u8; 1024];
        let size = read.read(&mut buffer).unwrap();
        for ch in 0..size {
            line.push(char::from(buffer[ch]));
        }
        msg_size = msg_size - size;
    }
    println!("reading");
    return line;
}

fn write_message(stream: &UnixStream, message: &String) -> usize {
    let mut write = BufWriter::new(stream);
    let msg_size = message.as_bytes().len();
    if write.write_u64::<BigEndian>(msg_size as u64).is_err() {
        println!("Can't write to broker");
        return 0;
    }
    if write.write_all(message.as_bytes()).is_err() {
        println!("Can't write to broker");
        return 0;
    }
    if write.flush().is_err() {
        println!("Can't flush");
        return 0;
    }
    println!("writing");
    return msg_size;
}

fn handle_client(stream: UnixStream) {
    let line = read_message(&stream);

    if line.len() == 0 {
        return;
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
        MsgTypes::Register => {},
        MsgTypes::Close => {
            let close = Message { header: MsgTypes::Close, value: String::from("") };
            let msg = serde_json::to_string(&close).unwrap();
            write_message(&stream, &msg);
        }
        _ => {
            let parts = msg.value.split_whitespace().collect::<Vec<_>>();

            let mut app = Command::new(parts[0]);
            for part in 1..parts.len() {
                app.arg(parts[part]);
            }
            let output = app.output().unwrap();

            let msg = String::from(String::from_utf8_lossy(output.stdout.as_slice()).as_ref());
            println!("{}", msg);
            write_message(&stream, &msg);
        }
    }
}

fn remove_socket(path: &String) {
    if std::fs::remove_file(path).is_err() {
        panic!("Can't remove socket {}", path);
    }
}

fn register_on_broker(path: &String, msg: &Message) {
    loop {
        match UnixStream::connect(path) {
            Ok(client) => {
                {
                    let msg = serde_json::to_string(msg).unwrap();
                    write_message(&client, &msg);
                    println!("{}", msg);
                }
                {
                    let line = read_message(&client);
                    println!("{}", line);

//                    let msg = serde_json::from_str(&line.as_str());
//                    let msg: Message = match msg {
//                        Ok(msg) => {
//                            msg
//                        },
//                        _ => {
//                            println!("{} {}", line, "Bad Message!");
//                            return;
//                        }
//                    };
                }

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

    let msg = Message {header: MsgTypes::Register, value: socket.clone()};
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
