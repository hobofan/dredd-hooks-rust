extern crate dredd_hooks;

use dredd_hooks::{HooksServer};

fn main() {
    let mut hooks = HooksServer::new();
    hooks.before("/message > GET", Box::new(move |mut tr| {
        let key = "hooks_modifications";
        {
            let mut vec = tr.entry(key).or_insert(Vec::<String>::new().into());
            vec.as_array_mut().unwrap().push("before modification".into());
        }
        tr
    }));
    hooks.after("/message > GET", Box::new(move |mut tr| {
        let key = "hooks_modifications";
        {
            let mut vec = tr.entry(key).or_insert(Vec::<String>::new().into());
            vec.as_array_mut().unwrap().push("after modification".into());
        }
        tr
    }));
    hooks.before_validation("/message > GET", Box::new(move |mut tr| {
        let key = "hooks_modifications";
        {
            let mut vec = tr.entry(key).or_insert(Vec::<String>::new().into());
            vec.as_array_mut().unwrap().push("before validation modification".into());
        }
        tr
    }));
    hooks.before_all(Box::new(move |mut tr| {
        let key = "hooks_modifications";
        {
            let mut vec = tr[0].entry(key).or_insert(Vec::<String>::new().into());
            vec.as_array_mut().unwrap().push("before all modification".into());
        }
        tr
    }));
    hooks.after_all(Box::new(move |mut tr| {
        let key = "hooks_modifications";
        {
            let mut vec = tr[0].entry(key).or_insert(Vec::<String>::new().into());
            vec.as_array_mut().unwrap().push("after all modification".into());
        }
        tr
    }));
    hooks.before_each(Box::new(move |mut tr| {
        let key = "hooks_modifications";
        {
            let mut vec = tr.entry(key).or_insert(Vec::<String>::new().into());
            vec.as_array_mut().unwrap().push("before each modification".into());
        }
        tr
    }));
    hooks.before_each_validation(Box::new(move |mut tr| {
        let key = "hooks_modifications";
        {
            let mut vec = tr.entry(key).or_insert(Vec::<String>::new().into());
            vec.as_array_mut().unwrap().push("before each validation modification".into());
        }
        tr
    }));
    hooks.after_each(Box::new(move |mut tr| {
        let key = "hooks_modifications";
        {
            let mut vec = tr.entry(key).or_insert(Vec::<String>::new().into());
            vec.as_array_mut().unwrap().push("after each modification".into());
        }
        tr
    }));
    HooksServer::start_from_env(hooks);
}
