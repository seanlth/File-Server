
extern crate DirSrv;

extern crate regex;

use regex::Regex;

use DirSrv::srv::*;

fn main() {
    let mut dir = DirectoryServer::new(8888, 3);

    //let parse = Regex::new(r"^ADD (([1-9]\d\d)|([1-9]\d)|(\d)).(([1-9]\d\d)|([1-9]\d)|(\d)).(([1-9]\d\d)|([1-9]\d)|(\d)).(([1-9]\d\d)|([1-9]\d)|(\d))$").unwrap();

    // let parse = Regex::new(r"^((\w+)/)?(\w+)$").unwrap();
    // println!("{}", parse.is_match("asd/asd") );
    // println!("{}", parse.captures("asd/asd").unwrap().at(1).unwrap() );
    //
    //
    // dir.servers.insert(0, ("127.0.0.1".to_string(), 8887));

    dir.run();
}
