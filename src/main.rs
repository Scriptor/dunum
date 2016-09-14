extern crate byteorder;

use std::fmt;
use std::net::{TcpListener, TcpStream};
use std::io::{Cursor,Read,Write};
use std::fs::{File, OpenOptions};
use std::thread;

use byteorder::{BigEndian, ReadBytesExt};

const LOG_FILE_NAME:&'static str = "main.log";

#[derive(Debug)]
struct Event{
    time: i32,
    data: i32,
}

impl fmt::Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Event with data: {} happened at {}",
               self.data, self.time)
    }
}

fn handle_client(stream: &mut TcpStream, log: &mut File) {
    let mut buf = [0; 4];
    loop {
        let n = stream.read(&mut buf).unwrap();
        if n == 0 { break; }

        let mut cursor = Cursor::new(&buf[..]);
        let num_expected = cursor.read_u32::<BigEndian>().unwrap() as usize;
        let _ = log.write(num_expected.to_string().as_bytes());
        println!("Writing {}", num_expected);
        let mut data = vec![0; num_expected];
        let _ = stream.read_to_end(&mut data);
        data.truncate(num_expected);

        let _ = match log.write(data.as_slice()) {
            Ok(n) => stream.write(n.to_string().as_bytes()),
            Err(_) => stream.write(b"err")
        };
    }
}

fn init_log() -> File {
    let f = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .open(LOG_FILE_NAME);
    match f {
        Ok(f) => f,
        Err(_) => panic!("Error: Can't create the log file: {}!", LOG_FILE_NAME),
    }
}

fn get_log() -> File {
    let f = OpenOptions::new()
            .read(true)
            .append(true)
            .open(LOG_FILE_NAME);
    match f {
        Ok(f) => f,
        Err(_) => panic!("Error: Can't open the log file: {}!", LOG_FILE_NAME),
    } 
}

fn main(){
    // println!("{}", DeepPair(Pair(1,2), Pair(3,4)));
    let e = Event{time : 1, data : 2};
    println!("{}", e);

    let _ = init_log();
    let listener = TcpListener::bind("0.0.0.0:4242").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut s) => {
                thread::spawn(move || {
                    let mut f = get_log();
                    handle_client(&mut s, &mut f)
                });
            }
            Err(e) => println!("ERROR! {:?}", e)
           
        }
    }

    drop(listener)
}
