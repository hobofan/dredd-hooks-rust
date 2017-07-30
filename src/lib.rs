//!

#[macro_use] extern crate serde_derive;
extern crate bufstream;
extern crate bytes;
extern crate futures;
extern crate serde_json;
extern crate tokio_core;
extern crate tokio_io;
extern crate tokio_proto;
extern crate tokio_service;
extern crate chan_signal;

use std::collections::HashMap;
use std::io::{BufRead, Write};
use std::net::{TcpStream as StdTcpStream};
use std::net::SocketAddr;
use std::process::{Command, Child};
use std::io;
use std::str;
use std::sync::{RwLock, Arc};

use chan_signal::Signal;
use bufstream::BufStream;
use bytes::BytesMut;
use serde_json::Value as Json;
use tokio_io::{AsyncRead, AsyncWrite};
use tokio_io::codec::{Encoder, Decoder};
use tokio_io::codec::Framed;
use tokio_proto::pipeline::ServerProto;
use tokio_service::Service;
use futures::{future, Future, BoxFuture};
use tokio_proto::TcpServer;

struct LineCodec;

impl Decoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn decode(&mut self, buf: &mut BytesMut) -> io::Result<Option<String>> {
        if let Some(i) = buf.iter().position(|&b| b == b'\n') {
            // remove the serialized frame from the buffer.
            let line = buf.split_to(i);

            // Also remove the '\n'
            buf.split_to(1);

            // Turn this data into a UTF string and return it in a Frame.
            match str::from_utf8(&line) {
                Ok(s) => Ok(Some(s.to_string())),
                Err(_) => Err(io::Error::new(io::ErrorKind::Other,
                                             "invalid UTF-8")),
            }
        } else {
            Ok(None)
        }
    }
}

impl Encoder for LineCodec {
    type Item = String;
    type Error = io::Error;

    fn encode(&mut self, msg: String, buf: &mut BytesMut) -> io::Result<()> {
        buf.extend(msg.as_bytes());
        buf.extend(b"\n");
        Ok(())
    }
}

struct LineProto;

impl<T: AsyncRead + AsyncWrite + 'static> ServerProto<T> for LineProto {
    type Request = String;
    type Response = String;

    /// A bit of boilerplate to hook in the codec:
    type Transport = Framed<T, LineCodec>;
    type BindTransport = Result<Self::Transport, io::Error>;
    fn bind_transport(&self, io: T) -> Self::BindTransport {
        Ok(io.framed(LineCodec))
    }
}

pub type Transaction = serde_json::map::Map<String, Json>;

#[derive(Clone)]
pub struct HooksServer {
    hooks_before_all: Arc<RwLock<Vec<Box<FnMut(Vec<Transaction>) -> Vec<Transaction> + Send + Sync>>>>,
    hooks_before_each: Arc<RwLock<Vec<Box<FnMut(Transaction) -> Transaction + Send + Sync>>>>,
    hooks_before: HashMap<String, Arc<RwLock<Vec<Box<FnMut(Transaction) -> Transaction + Send + Sync>>>>>,
    hooks_before_each_validation: Arc<RwLock<Vec<Box<FnMut(Transaction) -> Transaction + Send + Sync>>>>,
    hooks_before_validation: HashMap<String, Arc<RwLock<Vec<Box<FnMut(Transaction) -> Transaction + Send + Sync>>>>>,
    hooks_after: HashMap<String, Arc<RwLock<Vec<Box<FnMut(Transaction) -> Transaction + Send + Sync>>>>>,
    hooks_after_each: Arc<RwLock<Vec<Box<FnMut(Transaction) -> Transaction + Send + Sync>>>>,
    hooks_after_all: Arc<RwLock<Vec<Box<FnMut(Vec<Transaction>) -> Vec<Transaction> + Send + Sync>>>>,
}

impl HooksServer {
    pub fn new() -> Self {
        Self {
            hooks_before_all: Arc::new(RwLock::new(Vec::new())),
            hooks_before_each: Arc::new(RwLock::new(Vec::new())),
            hooks_before: HashMap::new(),
            hooks_before_each_validation: Arc::new(RwLock::new(Vec::new())),
            hooks_before_validation: HashMap::new(),
            hooks_after: HashMap::new(),
            hooks_after_each: Arc::new(RwLock::new(Vec::new())),
            hooks_after_all: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Start hook server with provided port.
    ///
    /// Since the port of the hook server is usually determined by a `IntegrationServer` (= `dredd-hooks-rust` command)
    /// there is a convenience method to pick up the correct port with `start_from_env`.
    pub fn start(srv: HooksServer, port: usize) {
        let address = SocketAddr::new("127.0.0.1".parse().unwrap(), port as u16);
        println!("Started hook server on port {}", port);

        let server = TcpServer::new(LineProto, address);
        server.serve(move || Ok(srv.clone()));
    }

    /// Start hook server with port taken from the env as it is set by a `IntegrationServer`.
    pub fn start_from_env(srv: HooksServer) {
        let port: usize = ::std::env::var("DREDD_RUNNER_PORT").expect("DREDD_RUNNER_PORT not set").parse().unwrap();
        Self::start(srv, port);
    }

    fn run_hooks_before_all(&self, mut transaction: MultiTransaction) -> MultiTransaction {
        for mut hook in &mut self.hooks_before_all.write().unwrap().iter_mut() {
            transaction.data = hook(transaction.data);
        }
        transaction
    }

    fn run_hooks_before_each(&self, mut transaction: SingleTransaction) -> SingleTransaction {
        for mut hook in &mut self.hooks_before_each.write().unwrap().iter_mut() {
            transaction.data = hook(transaction.data);
        }
        transaction
    }

    fn run_hooks_before(&self, mut transaction: SingleTransaction) -> SingleTransaction {
        if let Some(hooks) = self.hooks_before.get(transaction.data["name"].as_str().unwrap()) {
            for mut hook in hooks.write().unwrap().iter_mut() {
                transaction.data = hook(transaction.data);
            }
            transaction
        } else {
            transaction
        }
    }

    fn run_hooks_before_each_validation(&self, mut transaction: SingleTransaction) -> SingleTransaction {
        for mut hook in &mut self.hooks_before_each_validation.write().unwrap().iter_mut() {
            transaction.data = hook(transaction.data);
        }
        transaction
    }

    fn run_hooks_before_validation(&self, mut transaction: SingleTransaction) -> SingleTransaction {
        if let Some(hooks) = self.hooks_before_validation.get(transaction.data["name"].as_str().unwrap()) {
            for mut hook in hooks.write().unwrap().iter_mut() {
                transaction.data = hook(transaction.data);
            }
            transaction
        } else {
            transaction
        }
    }

    fn run_hooks_after(&self, mut transaction: SingleTransaction) -> SingleTransaction {
        if let Some(hooks) = self.hooks_after.get(transaction.data["name"].as_str().unwrap()) {
            for mut hook in hooks.write().unwrap().iter_mut() {
                transaction.data = hook(transaction.data);
            }
            transaction
        } else {
            transaction
        }
    }

    fn run_hooks_after_each(&self, mut transaction: SingleTransaction) -> SingleTransaction {
        for mut hook in &mut self.hooks_after_each.write().unwrap().iter_mut() {
            transaction.data = hook(transaction.data);
        }
        transaction
    }

    fn run_hooks_after_all(&self, mut transaction: MultiTransaction) -> MultiTransaction {
        for mut hook in &mut self.hooks_after_all.write().unwrap().iter_mut() {
            transaction.data = hook(transaction.data);
        }
        transaction
    }

    /// Register a hook that will run once before running any transactions.
    pub fn before_all(&mut self, closure: Box<FnMut(Vec<Transaction>) -> Vec<Transaction> + Send + Sync>) {
        self.hooks_before_all.write().unwrap().push(closure);
    }

    /// Register a hook that will run before each indiviual transactions.
    pub fn before_each(&mut self, closure: Box<FnMut(Transaction) -> Transaction + Send + Sync>) {
        self.hooks_before_each.write().unwrap().push(closure);
    }

    /// Register a hook that will run before a specific transactions.
    pub fn before<T: Into<String>>(&mut self, name: T, closure: Box<FnMut(Transaction) -> Transaction + Send + Sync>) {
        let old_hooks = self.hooks_before.entry(name.into()).or_insert(Arc::new(RwLock::new(Vec::new())));
        old_hooks.write().unwrap().push(closure);
    }

    /// Register a hook that will run before the validation of each indiviual transactions.
    pub fn before_each_validation(&mut self, closure: Box<FnMut(Transaction) -> Transaction + Send + Sync>) {
        self.hooks_before_each_validation.write().unwrap().push(closure);
    }

    /// Register a hook that will run before a specific transactions will be validated.
    pub fn before_validation<T: Into<String>>(&mut self, name: T, closure: Box<FnMut(Transaction) -> Transaction + Send + Sync>) {
        let old_hooks = self.hooks_before_validation.entry(name.into()).or_insert(Arc::new(RwLock::new(Vec::new())));
        old_hooks.write().unwrap().push(closure);
    }

    /// Register a hook that will run after a specific transactions.
    pub fn after<T: Into<String>>(&mut self, name: T, closure: Box<FnMut(Transaction) -> Transaction + Send + Sync>) {
        let old_hooks = self.hooks_after.entry(name.into()).or_insert(Arc::new(RwLock::new(Vec::new())));
        old_hooks.write().unwrap().push(closure);
    }

    /// Register a hook that will run after each indiviual transactions.
    pub fn after_each(&mut self, closure: Box<FnMut(Transaction) -> Transaction + Send + Sync>) {
        self.hooks_after_each.write().unwrap().push(closure);
    }

    /// Register a hook that will run once after running all other transactions.
    pub fn after_all(&mut self, closure: Box<FnMut(Vec<Transaction>) -> Vec<Transaction> + Send + Sync>) {
        self.hooks_after_all.write().unwrap().push(closure);
    }
}

impl Service for HooksServer {
    type Request = String;
    type Response = String;

    type Error = io::Error;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let probe_parsed: Result<ProbeEvent, serde_json::Error> = serde_json::from_str(&req);

        #[allow(unused_assignments)]
        let mut res = String::new();
        if let Ok(probe) = probe_parsed {
            let response_event = match probe.event {
                EventType::BeforeAll |
                EventType::AfterAll => {
                    let mut event: MultiTransaction = serde_json::from_str(&req).unwrap();
                    event = match event.event {
                        EventType::BeforeAll => self.run_hooks_before_all(event),
                        EventType::AfterAll => self.run_hooks_after_all(event),
                        _ => unreachable!(),
                    };
                    serde_json::to_string(&event).unwrap()
                },
                EventType::BeforeEach |
                EventType::Before |
                EventType::BeforeEachValidation |
                EventType::BeforeValidation |
                EventType::After |
                EventType::AfterEach => {
                    let mut event: SingleTransaction = serde_json::from_str(&req).unwrap();
                    event = match event.event {
                        EventType::BeforeEach => self.run_hooks_before_each(event),
                        EventType::Before => self.run_hooks_before(event),
                        EventType::BeforeEachValidation => self.run_hooks_before_each_validation(event),
                        EventType::BeforeValidation => self.run_hooks_before_validation(event),
                        EventType::After => self.run_hooks_after(event),
                        EventType::AfterEach => self.run_hooks_after_each(event),
                        _ => unreachable!(),
                    };
                    serde_json::to_string(&event).unwrap()
                }
            };

            res = response_event;
            future::ok(res).boxed()
        } else {
            future::err(io::Error::new(io::ErrorKind::Other, "Could not parse input as JSON")).boxed()
        }
    }
}

#[derive(Clone)]
pub struct IntegrationServer {
    next_port: usize,
    pub runners: Arc<RwLock<Vec<(usize, Child)>>>
}

impl IntegrationServer {
    pub fn new() -> Self {
        Self {
            next_port: 61322,
            runners: Arc::new(RwLock::new(Vec::new())),
        }
    }

    fn run_hookfile(hookfile: String, port: usize) -> Child {
        Command::new(&hookfile)
        .env("DREDD_RUNNER_PORT", port.to_string())
        .spawn()
        .expect(&format!("failed to start {}", hookfile))
    }

    pub fn setup_hooks(&mut self, hookfiles: Vec<String>) {
        for hookfile in hookfiles {
            let child = Self::run_hookfile(hookfile, self.next_port);
            self.runners.write().unwrap().push((self.next_port, child));
            self.next_port += 1;
        }
    }

    pub fn start(mut srv: IntegrationServer, hookfiles: Vec<String>) {
        srv.setup_hooks(hookfiles);

        // HACK: short waiting time; This is apparently recommended by dredd and the other language implementations do the same thing
        // An alternative would be some kind of check if we can open a connection to all the runners.
        ::std::thread::sleep(::std::time::Duration::from_millis(100));

        let port = 61321;
        let address = SocketAddr::new("127.0.0.1".parse().unwrap(), port as u16);

        let server = TcpServer::new(LineProto, address);
        println!("Starting");
        let runners = srv.runners.clone();

        let signal = chan_signal::notify(&[Signal::TERM]);
        ::std::thread::spawn(move || server.serve(move || Ok(srv.clone())));
        signal.recv().unwrap();

        for runner in runners.write().unwrap().iter_mut() {
            runner.1.kill().unwrap();
        }
    }
}


impl Service for IntegrationServer {
    type Request = String;
    type Response = String;

    type Error = io::Error;
    type Future = BoxFuture<Self::Response, Self::Error>;

    fn call(&self, req: Self::Request) -> Self::Future {
        let mut res = req;
        for runner in self.runners.write().unwrap().iter() {
            let port = runner.0;
            let outgoing_stream = StdTcpStream::connect(format!("127.0.0.1:{}", port))
                .expect(&format!("could not connect to port {}", port));
            let mut outgoing = BufStream::new(outgoing_stream);

            outgoing.write(res.as_bytes()).unwrap();
            outgoing.write("\n".as_bytes()).unwrap();
            outgoing.flush().unwrap();

            res = String::new();
            outgoing.read_line(&mut res).unwrap();
        }
        res.pop();

        future::ok(res).boxed()
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct MultiTransaction {
    event: EventType,
    uuid: String,
    data: Vec<Transaction>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct SingleTransaction {
    event: EventType,
    uuid: String,
    data: Transaction,
}

// Only deserialize event at first so that we can decide on a more specific struct.
#[derive(Serialize, Deserialize, Debug)]
struct ProbeEvent {
    event: EventType,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
enum EventType {
    #[serde(rename = "beforeAll")]
    BeforeAll,
    #[serde(rename = "beforeEach")]
    BeforeEach,
    #[serde(rename = "before")]
    Before,
    #[serde(rename = "beforeEachValidation")]
    BeforeEachValidation,
    #[serde(rename = "beforeValidation")]
    BeforeValidation,
    #[serde(rename = "after")]
    After,
    #[serde(rename = "afterEach")]
    AfterEach,
    #[serde(rename = "afterAll")]
    AfterAll,
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
    }
}
