
use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use regex::Regex;
use std::sync::{Arc, Mutex};

use std::fs;
use std::fs::{File, OpenOptions};
use std::env;
use std::path::Path;

use threadpool::ThreadPool;

use std::collections::HashMap;
use std::process::Command;

fn handle_lock(mut stream: TcpStream, srv: &mut LockServer, file_path: String) {

 	if srv.locks.iter().find(|&f| { *f == file_path }) == None {
		srv.locks.push(file_path);
		let _ = stream.write( format!("OK\r\n").as_bytes() );
	}
    else {
        let _ = stream.write( format!("ERR\r\n").as_bytes() );
    }
}

fn handle_unlock(mut stream: TcpStream, srv: &mut LockServer, file_path: String) {

	let index = srv.locks.iter().position(|f| *f == file_path);
	if let Some(i) = index {
		srv.locks.remove(i);
		let _ = stream.write( format!("OK\r\n").as_bytes() );
	}
}

fn handle_query(mut stream: TcpStream, srv: &mut LockServer, file_path: String) {

	let index = srv.locks.iter().position(|f| *f == file_path);
	if let Some(i) = index {
		let _ = stream.write( format!("LOCKED\r\n").as_bytes() );
	}
    else {
        let _ = stream.write( format!("UNLOCKED\r\n").as_bytes() );
    }
}

fn handle_connection(mut stream: TcpStream, srv: Arc<Mutex<LockServer>>)  {

	let mut buf = [0; 1024];
	if let Ok(size) = stream.read(&mut buf) {
		let message: &str  = &*(String::from_utf8_lossy( &buf[0..size] ));

		println!("request {}", message);

		let lock = Regex::new(r"^LOCK (((?:\w+/)*)(\w+))\r\n$").unwrap();
		let unlock = Regex::new(r"^UNLOCK (((?:\w+/)*)(\w+))\r\n$").unwrap();
        let query = Regex::new(r"^QUERY (((?:\w+/)*)(\w+))\r\n$").unwrap();

		let ping = Regex::new(r"^PING\r\n$").unwrap();

		if let Some(cap) = lock.captures(message) {
			let path = cap.at(1).unwrap();

			println!("locking {}", path);

			let mut s = srv.lock().unwrap();
			handle_lock(stream, &mut s, path.to_string());
		}
		else if let Some(cap) = unlock.captures(message) {
			let path = cap.at(1).unwrap();

			println!("unlocking {}", path);

			let mut s = srv.lock().unwrap();
			handle_unlock(stream, &mut s, path.to_string());
		}
        else if let Some(cap) = query.captures(message) {
            let path = cap.at(1).unwrap();

            println!("query {}", path);

            let mut s = srv.lock().unwrap();
            handle_query(stream, &mut s, path.to_string());
        }
		else if ping.is_match(message) {
			println!("ponging");
			let _ = stream.write( "Ok\r\n".to_string().as_bytes() );
		}

	}
}


#[derive(Clone)]
pub struct LockServer  {
	pub port: u32,
	pub locks: Vec<String>,
	pub directory_server: (String, u32)
}

impl LockServer {
	pub fn new(port: u32, directory_server: (String, u32)) -> LockServer {
		LockServer {
			port: port,
			locks: Vec::new(),
			directory_server: directory_server
		}
	}

	pub fn run(self) {
		let listener = TcpListener::bind( &*("0.0.0.0:".to_string() + &*self.port.to_string()) ).unwrap();
		let pool = ThreadPool::new(4);
		let lock = Arc::new(Mutex::new(self));

		loop {
			match listener.accept() {
				Ok((stream, _)) => {

					let this = lock.clone();
					pool.execute(move || {
						handle_connection(stream, this);
					});
				},
				Err(_) => { println!("An error occured " ); }
			}
		}
	}

}
