use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;

use regex::Regex;

pub struct Client {
	directory_server: (String, u32),

}

impl Client {
	pub fn new(directory_server: (String, u32)) -> Client {
		Client {
			directory_server: directory_server
		}
	}

	pub fn ping_directory_server(&self) -> bool {
		let (address, port) = self.directory_server.clone();
		let addr = format!("{}:{}", address, port);

		let mut s = TcpStream::connect( &*addr );

		if let Ok(ref mut stream) = s {
			let _ = stream.write( "PING\r\n".to_string().as_bytes() );
			let mut buf = [0; 1024];
			let r = stream.read(&mut buf);
			if let Ok(result) = r {
				let parse = Regex::new(r"^PONG\r\n$").unwrap();
				let pong = String::from_utf8_lossy(&buf[0..result]);
				if parse.is_match(&*pong) == true {
					return true;
				}
			}
		}
		false
	}

	pub fn run() {

	}
}
