extern crate dredd_hooks;

use std::env;
use dredd_hooks::IntegrationServer;

fn main() {
    let arguments: Vec<String> = env::args().collect();
    let hookfiles = arguments[1..].to_vec();

    let server = IntegrationServer::new();
    IntegrationServer::start(server, hookfiles);
}
