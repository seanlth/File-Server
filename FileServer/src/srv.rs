

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

fn handle_touch(mut stream: TcpStream, srv: &mut FileServer, file: String, path: String) {

	if let Ok(_) = env::set_current_dir(&srv.path) {
		if path == "" {
			if let Ok(_) = File::create(&file) {
				let _ = stream.write( "OK\r\n".to_string().as_bytes() );
			}
		}
		else {
			if let Ok(_) = fs::create_dir_all(&path) {
				if let Ok(_) = env::set_current_dir(&path) {
					if let Ok(_) = File::create(file) {
						let _ = stream.write( "OK\r\n".to_string().as_bytes() );
					}
				}
			}
		}
	}
}

fn handle_write(mut stream: TcpStream, srv: &mut FileServer, file_path: String, content: String) {

	if let Ok(_) = env::set_current_dir(&srv.path) {

		if let Ok(ref mut f) = OpenOptions::new().write(true).append(true).open(file_path) {
			let _ = f.write_all( content.as_bytes() );
			let _ = stream.write( "OK\r\n".to_string().as_bytes() );
		}
	}
}

fn handle_read(mut stream: TcpStream, srv: &mut FileServer, file_path: String) {

	if let Ok(_) = env::set_current_dir(&srv.path) {

		if let Ok(ref mut f) = OpenOptions::new().write(false).open(file_path) {
			let mut buf = String::new();
			let _  = f.read_to_string(&mut buf);
			let _ = stream.write( format!("{}\r\n", buf).as_bytes() );
		}
	}
}

fn handle_list(mut stream: TcpStream, srv: &mut FileServer, path: String) {

	if let Ok(_) = env::set_current_dir(&srv.path) {

		let paths = fs::read_dir(&*path).unwrap();

		let mut message = String::new();

	    for p in paths {
			message = message + p.unwrap().path().file_name().unwrap().to_str().unwrap() + "\t";
	    }

		let _ = stream.write( format!("{}\r\n", message).as_bytes() );
	}

}




fn handle_connection(mut stream: TcpStream, srv: Arc<Mutex<FileServer>>)  {

	let mut buf = [0; 1024];
	if let Ok(size) = stream.read(&mut buf) {
		let message: &str  = &*(String::from_utf8_lossy( &buf[0..size] ));

		println!("request {}", message);

		let touch = Regex::new(r"^TOUCH (((?:\w+/)*)(\w+))\r\n$").unwrap();
		let write = Regex::new(r"^WRITE (.+) \| ((?:(?:\w+/)*)(?:\w+))\r\n$").unwrap();
		let read = Regex::new(r"^READ (((?:\w+/)*)(\w+))\r\n$").unwrap();
		let list = Regex::new(r"^LIST ((((\w+)|(\.))/)((?:\w+/)*))\r\n$").unwrap();
		let ping = Regex::new(r"^PING\r\n$").unwrap();

		if let Some(cap) = touch.captures(message) {
			let path = cap.at(2).unwrap();
			let file = cap.at(3).unwrap();

			println!("touching {}/{}", path, file);

			let mut s = srv.lock().unwrap();
			handle_touch(stream, &mut s, file.to_string(), path.to_string());
		}
		else if let Some(cap) = write.captures(message) {
			let content = cap.at(1).unwrap();
			let path = cap.at(2).unwrap();

			println!("writing {} to {}", content, path);

			let mut s = srv.lock().unwrap();
			handle_write(stream, &mut s, path.to_string(), content.to_string());
		}
		else if let Some(cap) = read.captures(message) {
			let path = cap.at(1).unwrap();

			println!("reading from {}", path);

			let mut s = srv.lock().unwrap();
			handle_read(stream, &mut s, path.to_string());
		}
		else if let Some(cap) = list.captures(message) {
			let path = cap.at(1).unwrap();

			println!("listing {}", path);

			let mut s = srv.lock().unwrap();
			handle_list(stream, &mut s, path.to_string());
		}
		else if ping.is_match(message) {
			println!("ponging");
			let _ = stream.write( "PONG\r\n".to_string().as_bytes() );
		}

	}

}

#[derive(Clone)]
pub struct FileServer  {
	pub id: u32,
	pub port: u32,
	pub path: String,
	pub directory_server: (String, u32)
}

impl FileServer {
	pub fn new(port: u32, directory_server: (String, u32)) -> FileServer {
		FileServer {
			id : 0,
			port : port,
			path: "/Users/Seanlth/.dfs".to_string(),
			directory_server: directory_server
		}
	}

	pub fn read_directory_contents(&self, path: String, mut files: &mut Vec<String>) {

		if let Ok(paths) = fs::read_dir(&path) {
		    for p in paths {
				let file = p.unwrap().path().into_os_string().as_os_str().to_str().unwrap().to_string();
				&self.read_directory_contents(file.clone(), &mut files);

				let f = String::from( &file[2..file.len()] );
				files.push(f);
			}
		}
	}


	pub fn touch_file(&self, file: String) {
		let (address, port) = self.directory_server.clone();
		let addr = format!("{}:{}", address, port);
		let mut s = TcpStream::connect( &*addr );
		if let Ok(ref mut stream) = s {
			let message = format!("TOUCH {} {}\r\n", file, self.id);

			let _ = stream.write( message.to_string().as_bytes() );
		}
	}

	pub fn notify_directory_server(&mut self) -> bool {
		let (address, port) = self.directory_server.clone();
		let addr = format!("{}:{}", address, port);
		let mut s = TcpStream::connect( &*addr );

		if let Ok(ref mut stream) = s {
			let parse = Regex::new("^(.+):(.+)$").unwrap();
			let local = format!("{}", stream.local_addr().unwrap());
			let ip = parse.captures(&*local).unwrap().at(1).unwrap();
			let message = format!("ADD {}:{}\r\n", ip, self.port );

			let _ = stream.write( message.to_string().as_bytes() );
			let mut buf = [0; 1024];
			let r = stream.read(&mut buf);
			if let Ok(result) = r {
				let parse = Regex::new(r"^(\d{1,})\r\n$").unwrap();
				let id = String::from_utf8_lossy(&buf[0..result]);
				if let Some(cap) = parse.captures(&*id) {

					self.id = cap.at(1).unwrap().parse().ok().unwrap();
					let mut files = Vec::<String>::new();
					if let Ok(_) = env::set_current_dir(&self.path) {
						self.read_directory_contents("./".to_string(), &mut files);
						for file in files {
							self.touch_file(file);
						}
					}
					return true;
				}
			}
		}
		false
	}

	pub fn run(self) {
        let listener = TcpListener::bind( &*("0.0.0.0:".to_string() + &*self.port.to_string()) ).unwrap();
        let pool = ThreadPool::new(4);
		let lock = Arc::new(Mutex::new(self));

        loop {
            match listener.accept() {
                Ok((stream, _)) => {
					//let k = lock.lock().unwrap();

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
