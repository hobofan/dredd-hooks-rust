extern crate dredd_hooks;

use dredd_hooks::HooksServer;

fn main() {
    let mut server = HooksServer::new();
    server.before_all(Box::new(|tr| { println!("I am here"); tr } ));
    server.before_all(Box::new(|mut trs| {
        // let ref mut tr = trs[0];

        trs[0].insert("skip".to_owned(), true.into());

        trs
    } ));
    // server.before("Toasyt", Box::new(|mut tr| {
    //     println!("Toasty was run");
    //
    //     tr
    // } ));
    HooksServer::start_from_env(server);
}
