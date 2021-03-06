extern crate byteorder;

use std::str;
use std::thread;
use std::net::{TcpListener, TcpStream};
use std::io::{Read,Write,BufWriter,Cursor,Seek,SeekFrom};
use std::fs::{File, OpenOptions};
use byteorder::{BigEndian, ReadBytesExt};

const LOG_FILE_NAME:&'static str = "main.log";

struct LogEntry<'a> {
    size: &'a [u8],
    data: &'a [u8],
}

/// Takes a 4-byte array and returns a u32.
/// Assumes the bytes are in big-endian form.
fn read_u32(buf: [u8; 4]) -> u32 {
    let mut rdr = Cursor::new(buf);
    rdr.read_u32::<BigEndian>().unwrap()
}

/// Writes an instance of LogEntry to the given log file.
/// First writes the size in bytes of the payload, then the
/// payload itself. Uses a buffered writer to make the write atomic.
fn write_log(f: &File, entry: LogEntry) {
    let mut writer = BufWriter::new(f);
    writer.write_all(entry.size).unwrap();
    writer.write_all(entry.data).unwrap();
    writer.flush().unwrap();
}

/// Reads `n` entries starting at `offset` from the given `log` file,
/// returns a vector of strings that represent the given payloads.
fn read_entries(log: &mut File, offset: u32, n: u32)  -> Vec<String> {
    let mut entries: Vec<String> = Vec::with_capacity(n as usize);
    log.seek(SeekFrom::Start(0)).unwrap();
    for i in 0..(offset+n) {
        let mut entry_size_buf = [0; 4];
        let entry_size;
        let mut data_buf = Vec::new();
        match log.read_exact(&mut entry_size_buf) {
            Ok(_) => {
                entry_size = read_u32(entry_size_buf);
                log.take(entry_size as u64).read_to_end(&mut data_buf).unwrap();
                if i >= offset {
                    entries.push(String::from_utf8(data_buf).unwrap());
                }
            }
            _ => break
        }
    }
    entries
}

/// Reads incoming entries from `stream` and builds LogEntries
/// to write to the given `log` file. Writes back onto `stream`
/// the number of bytes written.
/// Incoming entries are in the format of first the size in bytes
/// of the expected payload followed by the payload itself.
fn handle_writer(stream: &mut TcpStream, log: &File) {
    let mut topic_exp_buf;
    let mut topic_buf = vec![];
    let mut data_exp_buf;
    let mut data_buf = vec![];
    loop {
        topic_exp_buf = [0; 4];
        data_exp_buf = [0; 4];
        topic_buf.truncate(0);
        data_buf.truncate(0);
        match stream.read_exact(&mut topic_exp_buf) {
            Ok(_) => {
                // Read in topic
                let topic_exp = read_u32(topic_exp_buf);

                stream.take(topic_exp as u64).read_to_end(&mut topic_buf).unwrap();
                println!("topic {:?}", topic_buf);

                // Read in number of expected data bytes
                stream.read_exact(&mut data_exp_buf).unwrap();
                let data_expected = read_u32(data_exp_buf);

                let n = stream.take(data_expected as u64) .read_to_end(&mut data_buf).unwrap();
                match str::from_utf8(&data_buf) {
                    Ok(data) => {
                        write_log(log, LogEntry{size: &data_exp_buf, data: &*data_buf});
                        println!("{:?}", data_exp_buf);
                        println!("{}", data);
                    }
                    _ => panic!("Couldn't convert data to string.")
                }
                stream.write(n.to_string().as_bytes()).unwrap();
            }
            _ => break
        }
    }
}

/// Takes incoming read requests from `stream`, fetches
/// the requested data from `log`, and writes them back
/// on `stream`.
fn handle_reader(stream: &mut TcpStream, log: &mut File) {
    let mut offset_buf;
    let mut num_entries_buf;
    loop {
        offset_buf = [0; 4];
        num_entries_buf = [0; 4];
        match stream.read_exact(&mut offset_buf) {
            Ok(_) => {
                let _ = stream.read_exact(&mut num_entries_buf);
                let offset = read_u32(offset_buf);
                let num_entries = read_u32(num_entries_buf);
                println!("Reading {} entries at offset {}", num_entries, offset);
                let entries = read_entries(log, offset, num_entries);
                let _ = stream.write(entries.join("").as_bytes());
            }
            _ => break
        }
    }
}

/// Create the log file if it does not exist yet.
fn init_log() -> File {
    let f = OpenOptions::new()
        .append(true)
        .open(LOG_FILE_NAME);
    match f {
        Ok(f) => f,
        Err(_) => panic!("Error: Can't create the log file: {}!", LOG_FILE_NAME),
    } 
}

/// Grab a file handle that allows appending to the log file.
fn get_log_writer() -> File {
    let f = OpenOptions::new()
        .append(true)
        .open(LOG_FILE_NAME);
    match f {
        Ok(f) => f,
        Err(_) => panic!("Error: Can't open the log file: {}!", LOG_FILE_NAME),
    } 
}

/// Grab a file handle that allows reading from the log file.
fn get_log_reader() -> File {
    File::open(LOG_FILE_NAME).unwrap()
}

fn main(){
    let _ = init_log();

    let writers_thread = thread::spawn(|| {
        println!("Listening for writers on on :4242");
        let writers = TcpListener::bind("0.0.0.0:4242").unwrap();
        for s in writers.incoming() {
            match s {
                Ok(mut stream) => {
                    thread::spawn(move || {
                        let f = get_log_writer();
                        handle_writer(&mut stream, &f)
                    });
                }
                Err(e) => println!("ERROR! {:?}", e)
            }
        }
    });

    let readers_thread = thread::spawn(|| {
        println!("Listening for readers on on :2424");
        let readers = TcpListener::bind("0.0.0.0:2424").unwrap();
        for s in readers.incoming() {
            match s {
                Ok(mut stream) => {
                    thread::spawn(move || {
                        let mut f = get_log_reader();
                        handle_reader(&mut stream, &mut f)
                    });
                }
                Err(e) => println!("ERROR! {:?}", e)
            }
        }
    });
    let _ = writers_thread.join();
    let _ = readers_thread.join();
}
