extern crate dredd_hooks;

use dredd_hooks::{HooksServer};

fn main() {
    let mut hooks = HooksServer::new();
    hooks.before("/message > GET", Box::new(|mut tr| {
        println!("It's me, File3");
        tr
    }));
    HooksServer::start_from_env(hooks);
}
