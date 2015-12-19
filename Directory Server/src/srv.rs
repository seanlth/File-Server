

use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use regex::Regex;
use std::sync::{Arc, Mutex};


use threadpool::ThreadPool;

use std::collections::HashMap;


fn handle_search(mut stream: TcpStream, srv: &DirectoryServer, file: String) {

	if let Some((_, id)) = srv.map.iter().find(|&(f, _)| { *f == file }) {
		println!("1");
		println!("{}", id);
		println!("{}", file);
		println!("{}", srv.servers.len());
		if let Some((_, fs)) = srv.servers.iter().find(|&(i, _)| { *i == *id }) {
			let (ip, port) = fs.clone();

			println!("2");

			let reply = &*format!("{}:{}:{}\r\n", id, ip, port);
			let _ = stream.write( reply.to_string().as_bytes() );
		}
	}
}

fn handle_touch(mut stream: TcpStream, srv: &mut DirectoryServer, file: String) {
	if let None = srv.map.iter().find(|&(f, _)| { *f == file }) {
		srv.map.insert(file, srv.current_server);
		srv.current_server = ((srv.current_server + 1) as usize % srv.servers.len() as usize) as u32;

		let _ = stream.write( "OK\r\n".to_string().as_bytes() );
	}
}

fn handle_add(mut stream: TcpStream, srv: &mut DirectoryServer, ip: String, port: u32) {
	srv.servers.insert(srv.number_of_servers, (ip, port));

	let message = format!("{}\r\n", srv.number_of_servers.to_string());

	let _ = stream.write( message.as_bytes() );

	srv.number_of_servers += 1;
	println!("asd");
}

fn handle_connection(mut stream: TcpStream, srv: Arc<Mutex<DirectoryServer>>)  {

	let mut buf = [0; 1024];
	if let Ok(size) = stream.read(&mut buf) {
		let message: &str  = &*(String::from_utf8_lossy( &buf[0..size] ));

		let search = Regex::new(r"^SEARCH (.+)\r\n$").unwrap();
		let touch = Regex::new(r"^TOUCH (.+)\r\n$").unwrap();
		let add = Regex::new(r"^ADD ((?:(?:[1-9]\d\d)|(?:[1-9]\d)|(?:\d)).(?:(?:[1-9]\d\d)|(?:[1-9]\d)|(?:\d)).(?:(?:[1-9]\d\d)|(?:[1-9]\d)|(?:\d)).(?:(?:[1-9]\d\d)|(?:[1-9]\d)|(?:\d))):(\d{1,5})\r\n$").unwrap();
		let ping = Regex::new(r"^PING\r\n$").unwrap();

		if let Some(cap) = search.captures(message) {
			let file = cap.at(1).unwrap();
			let s = srv.lock().unwrap();

			handle_search(stream, &s, file.to_string());
		}
		else if let Some(cap) = touch.captures(message) {
			let file = cap.at(1).unwrap();
			let mut s = srv.lock().unwrap();

			handle_touch(stream, &mut s, file.to_string());
		}
		else if let Some(cap) = add.captures(message) {
			let ip = cap.at(1).unwrap();
			let port = cap.at(2).unwrap();

			let mut s = srv.lock().unwrap();

			println!("{}", ip);
			println!("{}", port);

			handle_add(stream, &mut s, ip.to_string(), port.parse().ok().unwrap() );
		}
		else if ping.is_match(message) {
			let _ = stream.write( "PONG".to_string().as_bytes() );
		}
	}

}

#[derive(Clone)]
pub struct DirectoryServer  {
	pub map: HashMap<String, u32>, //id
	pub servers: HashMap<u32, (String, u32)>, // ip, port
	port: u32,
	current_server: u32,
	number_of_servers: u32
}


impl DirectoryServer {
	pub fn new(port: u32, number_of_servers: u32) -> DirectoryServer {
		DirectoryServer {
			map : HashMap::new(),
			servers : HashMap::new(),
			port : port,
			current_server : 0,
			number_of_servers: number_of_servers
		}
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
