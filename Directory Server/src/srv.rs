

use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use regex::Regex;
use std::sync::{Arc, Mutex};

use threadpool::ThreadPool;

use std::collections::HashMap;

fn matches_path(path: &String, file: &String) -> Option<String> {
	println!("{}", file);
	if file.len() > path.len() {
		let top = Regex::new(r"(\w+)(/(.+))?").unwrap();
		if path == "./" {
			if let Some(cap) = top.captures(file) {
				let file = cap.at(1).unwrap();
				return Some(file.to_string());
			}
		}
		else {
			let (p, f) = file.split_at(path.len());
			if p.to_string() == *path {
				if let Some(cap) = top.captures(f) {
					let file = cap.at(1).unwrap();
					return Some(file.to_string());
				}
			}
		}
	}
	return None;
}

fn handle_search(mut stream: TcpStream, srv: &DirectoryServer, file: String) {

	if srv.number_of_servers == 0 {
		let reply = &*format!("ERR\r\n");
		let _ = stream.write( reply.to_string().as_bytes() );
	}
	else if let Some((_, id)) = srv.map.iter().find(|&(f, _)| { *f == file }) {
		println!("1");
		println!("{}", id);
		println!("{}", file);
		println!("{}", srv.servers.len());
		if let Some((_, fs)) = srv.servers.iter().find(|&(i, _)| { *i == *id }) {
			let (ip, port) = fs.clone();

			println!("2");

			let reply = &*format!("{}:{}\r\n", ip, port);
			let _ = stream.write( reply.to_string().as_bytes() );
		}
	}
}

fn handle_touch(mut stream: TcpStream, server: Option<u32>, srv: &mut DirectoryServer, file: String) {
	if srv.number_of_servers == 0 {
		let reply = &*format!("ERR\r\n");
		let _ = stream.write( reply.to_string().as_bytes() );
	}

	if let None = srv.map.iter().find(|&(f, _)| { *f == file }) {

		match server {
			Some(id) => {
				srv.map.insert(file, id);

				let (ip, port) = srv.servers.get(&id).unwrap().clone();

				let message = format!("{}:{}\r\n", ip, port);
				let _ = stream.write( message.to_string().as_bytes() );
			}
			None => {
				srv.map.insert(file, srv.current_server);

				let (ip, port) = srv.servers.get(&srv.current_server).unwrap().clone();

				srv.current_server = ((srv.current_server + 1) as usize % srv.servers.len() as usize) as u32;

				let message = format!("{}:{}\r\n", ip, port);
				let _ = stream.write( message.to_string().as_bytes() );
			}
		}

	}
}

fn handle_list(mut stream: TcpStream, srv: &DirectoryServer, path: String) {

	if srv.number_of_servers == 0 {
		let reply = &*format!("ERR\r\n");
		let _ = stream.write( reply.to_string().as_bytes() );
	}
	else {
		let mut message = String::new();
		let mut files = Vec::<String>::new();
		for (f, _) in &srv.map {
			if let Some(file) = matches_path(&path, &f) {
				let t = file.clone();
				if files.iter().find(|&x| { *x == t } ).is_none() {
					files.push(t);
					message = message + &*file + "\t";
				}
			}
		}
		let _ = stream.write( format!("{}\r\n", message).as_bytes() );
	}
}

fn handle_add(mut stream: TcpStream, srv: &mut DirectoryServer, ip: String, port: u32) {
	srv.servers.insert(srv.number_of_servers, (ip, port));

	let message = format!("{}\r\n", srv.number_of_servers.to_string());

	let _ = stream.write( message.as_bytes() );

	srv.number_of_servers += 1;
}


fn handle_connection(mut stream: TcpStream, srv: Arc<Mutex<DirectoryServer>>)  {

	let mut buf = [0; 1024];
	if let Ok(size) = stream.read(&mut buf) {
		let message: &str  = &*(String::from_utf8_lossy( &buf[0..size] ));

		println!("request {}", message);

		let search = Regex::new(r"^SEARCH ((?:(?:\w+/)*)(?:\w+))\r\n$").unwrap();
		let touch = Regex::new(r"^TOUCH ((?:(?:\w+/)*)(?:\w+))(?: (\d+))?\r\n$").unwrap();
		let list = Regex::new(r"^LIST ((((\w+)|(\.))/)((?:\w+/)*))\r\n$").unwrap();
		let add = Regex::new(r"^ADD ((?:(?:[1-9]\d\d)|(?:[1-9]\d)|(?:\d)).(?:(?:[1-9]\d\d)|(?:[1-9]\d)|(?:\d)).(?:(?:[1-9]\d\d)|(?:[1-9]\d)|(?:\d)).(?:(?:[1-9]\d\d)|(?:[1-9]\d)|(?:\d))):(\d{1,5})\r\n$").unwrap();
		let ping = Regex::new(r"^PING\r\n$").unwrap();

		if let Some(cap) = search.captures(message) {
			let file = cap.at(1).unwrap();

			println!("searching for {}", file);

			let s = srv.lock().unwrap();
			handle_search(stream, &s, file.to_string());
		}
		else if let Some(cap) = touch.captures(message) {
			let file = cap.at(1).unwrap();
			let server_id = match cap.at(2) {
				Some(id) => Some(id.parse().ok().unwrap()),
				None => None
			};

			println!("touching {}", file);

			let mut s = srv.lock().unwrap();
			handle_touch(stream, server_id, &mut s, file.to_string());
		}
		else if let Some(cap) = list.captures(message) {
			let path = cap.at(1).unwrap();

			println!("listing {}", path);

			let mut s = srv.lock().unwrap();
			handle_list(stream, &mut s, path.to_string());
		}
		else if let Some(cap) = add.captures(message) {
			let ip = cap.at(1).unwrap();
			let port = cap.at(2).unwrap();

			println!("adding server {}:{}", ip, port);

			let mut s = srv.lock().unwrap();
			handle_add(stream, &mut s, ip.to_string(), port.parse().ok().unwrap() );
		}
		else if ping.is_match(message) {
			println!("ponging");
			let _ = stream.write( "Ok".to_string().as_bytes() );
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
	pub fn new(port: u32) -> DirectoryServer {
		DirectoryServer {
			map : HashMap::new(),
			servers : HashMap::new(),
			port : port,
			current_server : 0,
			number_of_servers: 0
		}
	}



	pub fn run(self) {
        let listener = TcpListener::bind( &*("0.0.0.0:".to_string() + &*self.port.to_string()) ).unwrap();
        let pool = ThreadPool::new(4);
		let lock = Arc::new(Mutex::new(self));

        loop {
            match listener.accept() {
                Ok((stream, _)) => {
					println!("asd");

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
