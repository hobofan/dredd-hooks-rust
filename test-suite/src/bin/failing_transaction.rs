extern crate dredd_hooks;

use dredd_hooks::{HooksServer};

fn main() {
    let mut hooks = HooksServer::new();
    hooks.before("/message > GET", Box::new(|mut tr| {
        tr.insert("fail".to_owned(), "Yay! Failed!".into());
        tr
    }));
    HooksServer::start_from_env(hooks);
}
