extern crate dredd_hooks;

use dredd_hooks::{HooksServer};

fn main() {
    let mut hooks = HooksServer::new();
    hooks.before("/message > GET", Box::new(move |tr| {
        println!("before hook handled");
        tr
    }));
    hooks.after("/message > GET", Box::new(move |tr| {
        println!("after hook handled");
        tr
    }));
    hooks.before_validation("/message > GET", Box::new(move |tr| {
        println!("before validation hook handled");
        tr
    }));
    hooks.before_all(Box::new(move |tr| {
        println!("before all hook handled");
        tr
    }));
    hooks.after_all(Box::new(move |tr| {
        println!("after all hook handled");
        tr
    }));
    hooks.before_each(Box::new(move |tr| {
        println!("before each hook handled");
        tr
    }));
    hooks.before_each_validation(Box::new(move |tr| {
        println!("before each validation hook handled");
        tr
    }));
    hooks.after_each(Box::new(move |tr| {
        println!("after each hook handled");
        tr
    }));
    HooksServer::start_from_env(hooks);
}
