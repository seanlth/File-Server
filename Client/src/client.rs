use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use std::io::{self};

use regex::Regex;

pub struct Client {
	directory_server: (String, u32),
	lock_server: (String, u32),
}

impl Client {
	pub fn new(directory_server: (String, u32), lock_server: (String, u32)) -> Client {
		Client {
			directory_server: directory_server,
			lock_server: lock_server
		}
	}

	pub fn send_request(&self, server: (String, u32), request: String) -> Option<String> {
		let (address, port) = server;
		let addr = format!("{}:{}", address, port);

		let mut s = TcpStream::connect( &*addr );

		if let Ok(ref mut stream) = s {
			let mut buf = [0; 1024];

			let _ = stream.write( request.as_bytes() );
			let success = stream.read(&mut buf);

			if let Ok(s) = success {
				let result = String::from_utf8_lossy(&buf[0..s]);
				return Some( result.into_owned() );
			}
		}
		None
	}

	pub fn touch_file(&self, file_path: String) -> bool {
		let message = format!("TOUCH {}\r\n", file_path);

		if let Some(server) = self.send_request(self.directory_server.clone(), message.clone()) {
			let parse = Regex::new(r"^(.+):(.+)\r\n$").unwrap();
			if let Some(cap) = parse.captures(&*server) {
				let ip = cap.at(1)
							.unwrap()
							.to_string();

				let port = cap.at(2)
				              .unwrap()
							  .parse()
							  .ok()
							  .unwrap();

				self.send_request((ip, port), message.clone());
				return true
			}
		}

		false
	}

	pub fn read_file(&self, file_path: String) -> Option<String> {
		let message = format!("SEARCH {}\r\n", file_path);

		if let Some(server) = self.send_request(self.directory_server.clone(), message.clone()) {
			let parse = Regex::new(r"^(.+):(.+)\r\n$").unwrap();
			if let Some(cap) = parse.captures(&*server) {
				let ip = cap.at(1)
							.unwrap()
							.to_string();

				let port = cap.at(2)
				              .unwrap()
							  .parse()
							  .ok()
							  .unwrap();

				let message = format!("READ {}\r\n", file_path);

				return self.send_request((ip, port), message.clone());
			}
		}
		None
	}

	pub fn write_file(&self, string: String, file_path: String) -> Option<String> {
		let message = format!("LOCK {}\r\n", file_path);

		if let Some(lock) = self.send_request(self.lock_server.clone(), message.clone()) {
			if lock == "OK\r\n" {
				let message = format!("SEARCH {}\r\n", file_path);

				if let Some(server) = self.send_request(self.directory_server.clone(), message.clone()) {
					let parse = Regex::new(r"^(.+):(.+)\r\n$").unwrap();
					if let Some(cap) = parse.captures(&*server) {
						let ip = cap.at(1)
									.unwrap()
									.to_string();

						let port = cap.at(2)
									  .unwrap()
									  .parse()
									  .ok()
									  .unwrap();

						let message = format!("WRITE {} | {}\r\n", string, file_path);

						return self.send_request((ip, port), message.clone());
					}
				}
			}
		}
		None
	}

	pub fn list_files(&self, file_path: String) -> Option<String> {
		let message = format!("LIST {}\r\n", file_path);

		return self.send_request(self.directory_server.clone(), message.clone());
	}

	pub fn run(&self) {
		let touch = Regex::new(r"^touch (((?:\w+/)*)(\w+))$").unwrap();
		let cat = Regex::new(r"^cat (((?:\w+/)*)(\w+))$").unwrap();
		let write_string = "^\"(.+)\" > (((?:\\w+/)*)(\\w+))$";
		let write = Regex::new(write_string).unwrap();
		let list = Regex::new(r"ls ((((\w+)|(\.))/)((?:\w+/)*))").unwrap();

		println!("{}", write);

    	let stdin = io::stdin();
    	for line in stdin.lock().lines() {
			let s = line.unwrap();

			if let Some(cap) = touch.captures(&*s) {
				let path = cap.at(1).unwrap();
				self.touch_file(path.to_string());
			}
			else if let Some(cap) = cat.captures(&*s) {
				let path = cap.at(1).unwrap();
				let r = self.read_file(path.to_string());
				if let Some(result) = r {
					println!("{}", result );
				}
				else {
					println!("{}:  No such file or directory", path);
				}
			}
			else if let Some(cap) = write.captures(&*s) {
				let string = cap.at(1).unwrap();
				let path = cap.at(2).unwrap();
				let r = self.write_file(string.to_string(), path.to_string());
				if let Some(result) = r {
					if result == "OK\r\n" {
						println!("{}", result );
					}
					else {
						println!("{}:  No such file or directory", path);
					}
				}
			}
			else if let Some(cap) = list.captures(&*s) {
				let path = cap.at(1).unwrap();
				println!("{}", path);
				let r = self.list_files(path.to_string());
				if let Some(result) = r {
					if result == "ERR\r\n" {
						println!("{}:  No such file or directory", path);
					}
					else {
						println!("{}", result);
					}
				}
			}
		}
	}
}
