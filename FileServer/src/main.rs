
extern crate FileSrv;
extern crate regex;
use std::fs::File;

use FileSrv::srv::*;
use std::io::prelude::*;
use regex::Regex;

fn main() {
    let mut fs = FileServer::new(9000, ("0.0.0.0".to_string(), 8888));

    let parse = Regex::new(r"^((?:\w+/)*)(\w+)$").unwrap();


    // println!("{}",  String::from_utf8_lossy(&buf) );

    //println!("{}", parse.captures("asd2").unwrap().at(1).unwrap() );
    println!("{}", fs.notify_directory_server());
    //
    // let path = fs.path.clone();
    // let mut files: Vec<String> = vec![];
    //
    // fs.read_directory_contents(path, &mut files);
    //
    // for f in files {
    //     println!("{}", f);
    // }

    fs.run();
}
