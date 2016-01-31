extern crate LockSrv;

use LockSrv::srv::*;

fn main() {
    let mut lock_server = LockServer::new(8889, ("0.0.0.0".to_string(), 8888));

    lock_server.run();
}
