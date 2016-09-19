extern crate byteorder;

use std::str;
use std::thread;
use std::net::{TcpListener, TcpStream};
use std::io::{Read,Write,BufWriter,Cursor};
use std::fs::{File, OpenOptions};
use byteorder::{BigEndian, ReadBytesExt};

const LOG_FILE_NAME:&'static str = "main.log";

struct LogEntry<'a> {
    size: &'a [u8],
    data: &'a [u8],
}

fn write_log(f: &File, entry: LogEntry) {
    let mut writer = BufWriter::new(f);
    writer.write_all(entry.size).unwrap();
    writer.write_all(entry.data).unwrap();
    writer.flush().unwrap();
}

fn handle_client(stream: &mut TcpStream, log: &File) {
    let mut num_exp_buf;
    let mut data_buf = vec![];
    loop {
        num_exp_buf = [0; 4];
        data_buf.truncate(0);
        match stream.read_exact(&mut num_exp_buf) {
            Ok(_) => {
                let mut rdr = Cursor::new(num_exp_buf);
                let num_expected = rdr.read_u32::<BigEndian>().unwrap();

                let n = stream.take(num_expected as u64)
                    .read_to_end(&mut data_buf).unwrap();
                match str::from_utf8(&data_buf) {
                    Ok(data) => {
                        write_log(log, LogEntry{size: &num_exp_buf, data: &*data_buf});
                        println!("{:?}", num_exp_buf);
                        println!("{}", data);
                    }
                    _ => panic!("Couldn't convert data to string.")
                }
                let _ = stream.write(n.to_string().as_bytes());
            }
            _ => break
        }
    }
}

fn init_log() -> File {
    let f = OpenOptions::new()
            .append(true)
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
    let _ = init_log();
    let listener = TcpListener::bind("0.0.0.0:4242").unwrap();

    for s in listener.incoming() {
        match s {
            Ok(mut stream) => {
                thread::spawn(move || {
                    let f = get_log();
                    handle_client(&mut stream, &f)
                });
            }
            Err(e) => println!("ERROR! {:?}", e)
        }
    }

    drop(listener)
}
