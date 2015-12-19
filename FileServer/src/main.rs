
extern crate FileSrv;
extern crate regex;
use std::fs::File;

use FileSrv::srv::*;
use std::io::prelude::*;
use regex::Regex;

fn main() {
    let fs = FileServer::new(3, 8889, ("127.0.0.1".to_string(), 8888));

    let parse = Regex::new(r"^((?:\w+/)*)(\w+)$").unwrap();


    // println!("{}",  String::from_utf8_lossy(&buf) );

    //println!("{}", parse.captures("asd2").unwrap().at(1).unwrap() );
    println!("{}", fs.notify_directory_server());
    fs.run();
}
