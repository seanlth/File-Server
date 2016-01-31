extern crate Clnt;


use Clnt::client::*;

fn main() {
    let mut client = Client::new(("0.0.0.0".to_string(), 8888), ("0.0.0.0".to_string(), 8889));

    client.run();
}
