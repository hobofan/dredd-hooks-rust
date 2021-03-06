# dredd-hooks-rust • Dredd HTTP API testing integration for Rust
[![Build Status](https://travis-ci.org/hobofan/dredd-hooks-rust.svg?branch=master)](https://travis-ci.org/hobofan/dredd-hooks-rust)
[![Crates.io](https://img.shields.io/crates/v/dredd-hooks.svg)](https://crates.io/crates/dredd-hooks)
[![docs.rs](https://docs.rs/dredd-hooks/badge.svg)](https://docs.rs/dredd-hooks)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)]()

This package contains a Rust Dredd hook handler which provides a bridge between the [Dredd API Testing Framework](http://dredd.readthedocs.org/en/latest/)
 and Rust environment to ease implementation of testing hooks provided by [Dredd](http://dredd.readthedocs.org/en/latest/). Write Dredd hooks in Rust to glue together [API Blueprint](https://apiblueprint.org/) with your Rust project.

Not sure what these Dredd Hooks are?  Read the Dredd documentation on [them](http://dredd.readthedocs.org/en/latest/hooks/).

The following are a few examples of what hooks can be used for:

- loading db fixtures
- cleanup after test step or steps
- handling authentication and sessions
- passing data between transactions (saving state from responses to stash)
- modifying request generated from blueprint
- changing generated expectations
- setting custom expectations
- debugging via logging stuff

## Installation

### Global installation

If you don't have it already, install the Dredd CLI via [npm](npm):

```bash
npm install -g dredd
```

In order for the Dredd CLI to be able to interface with your test binaries, you need to have the `dredd-hooks-rust` binary installed, which you can get by running:

```bash
# This will install both `dredd-hooks-rust` and `cargo-dredd`
cargo install dredd-hooks
```

[npm]: https://docs.npmjs.com/getting-started/what-is-npm

### Per-project setup

To start testing your Rust project with Dredd, just add `dredd-hooks` to your `Cargo.toml`:

```toml
[dependencies]
dredd-hooks = "0.3.0"
```

Or if you have [cargo-edit][cargo-edit] installed you can just run this on the command line:
```bash
cargo add dredd-hooks
```

[cargo-edit]: https://github.com/killercup/cargo-edit

## Quickstart example

Following this is a short example showcasing Dredd tests running against an `iron` server.

The name of the project in this example is assumed to be `dredd-rust-test`:

`test.apib`
```apib
# My Api
## GET /message
+ Response 200 (text/plain)
    Hello World!
```

`main.rs`:
```rust
extern crate iron;
extern crate router;
extern crate dredd_hooks;

use iron::prelude::*;
use router::Router;
use dredd_hooks::{HooksServer};

// HTTP endpoint
fn endpoint(_: &mut Request) -> IronResult<Response> {
    Ok(Response::with((iron::status::Ok, "Hello World!\n\n")))
}

fn main() {
    let mut hooks = HooksServer::new();
    // Start the server before any of the tests are running.
    hooks.before_all(Box::new(|tr| {
        ::std::thread::spawn(move || {
            let mut router = Router::new();
            router.get("/message", endpoint, "endpoint");

            Iron::new(router).http("127.0.0.1:3000").unwrap();
        });
        tr
    }));
    // Execute a hook before a specific test.
    hooks.before("/message > GET", Box::new(|mut tr| {
        // Set the skip flag on this test.
        // Comment out the next line and you should see a passing test.
        tr.insert("skip".to_owned(), true.into());

        tr
    }));
    HooksServer::start_from_env(hooks);
}
```

Run the command:
```bash
cargo build && dredd ./test.apib http://127.0.0.1:3000 --language=dredd-hooks-rust --hookfiles=target/debug/dredd-rust-test
```

You should now see Dredd trying to run the tests against the binary that was just compiled, but actually skipping the single test it tries to run because we told Dredd to do so via a `before` hook.

## Project setup

The quickstart example above assumes that the hookfile is compiled as a `bin` target.
However, in most projects, you will probably want to have a more robust setup that looks like this:

`Cargo.toml`:

```toml
[[test]]
name = "dredd_test_hooks"
path = "tests/dredd/hooks.rs"
test = false
harness = false

[package.metadata.dredd_hooks]
hook_targets = ["dredd_test_hooks"]
```

Setting the `test` value to `false`, is needed so that our blocking hookserver doesn't interfere with the other tests when running `cargo test`.
Setting the `harness` to `false` will result in the test binary being compiled without a test harness, because we already have `dredd` as our test harness.

Finally the values under `package.metadata.dredd_hooks` give us some additional metadata about our test setup, which allows us to use the `cargo dredd` command to simplify the invocation:
```
cargo dredd ./test.apib http://127.0.0.1:3000
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

## Acknowledgements

Thank you to:
- [The developers behind goodman](https://github.com/snikch/goodman) for providing a good example of how to integrate Dredd with a compiled language.
