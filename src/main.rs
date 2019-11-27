use std::io::{BufRead, BufReader, BufWriter, Write};
use std::os::unix::net::{UnixStream,UnixListener};
use std::thread;
use std::ops::Add;

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

fn main() {
    let server = UnixListener::bind("/tmp/app-uds.sock").unwrap();

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
