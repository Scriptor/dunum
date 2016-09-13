use std::fmt;
use std::net::{TcpListener, TcpStream};
use std::io::{Read,Write};
use std::fs::{File, OpenOptions};
use std::thread;

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

fn handle_client(stream: &mut TcpStream, f: &mut File) {
    let mut buf = [0; 10];
    loop {
        match stream.read(&mut buf) {
            Ok(n) => {
                if n == 0 { break; }
                let _ = f.write(&buf);
            }
            Err(_) => break
        }
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
            .write(true)
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
