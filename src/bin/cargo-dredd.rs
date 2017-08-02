#[macro_use] extern crate serde_derive;
extern crate cargo_edit;
extern crate cargo;
extern crate docopt;

use std::process::Command;
use std::os::unix::process::CommandExt;

use cargo_edit::Manifest;
use cargo::core::shell::Verbosity;
use cargo::core::Workspace;
use cargo::ops::{compile, CompileOptions, CompileMode, CompileFilter};
use cargo::util::config::Config;
use docopt::Docopt;

const USAGE: &'static str = "
cargo-dredd

Usage:
  cargo dredd <blueprint> <server-url> [options]

Options:
  --language=<rust>  The language flag that will be passed to dredd [default: rust].
";

#[derive(Debug, Deserialize)]
struct Args {
    arg_blueprint: String,
    arg_server_url: String,
    flag_language: Option<String>,
}

fn main() {
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| d.deserialize())
                            .unwrap_or_else(|e| e.exit());

    let hook_binaries = cargo_compile();

    let mut cmd = Command::new("dredd");
    cmd.args(&[args.arg_blueprint, args.arg_server_url]);
    cmd.arg(format!("--language={}", args.flag_language.unwrap_or("rust".to_owned()).to_owned()));
    for binary in hook_binaries {
        cmd.args(&[format!("--hookfiles={}", binary)]);
    }

    cmd.exec();
}

fn cargo_compile() -> Vec<String> {
    let mut manifest = Manifest::open(&None).unwrap();
    let config = manifest.get_table(&[
        "package".to_owned(),
        "metadata".to_owned(),
        "dredd_hooks".to_owned()]
    ).expect("No [package.metadata.dredd_hooks] value found in Cargo.toml");
    if config.is_empty() {
        panic!("No [package.metadata.dredd_hooks] value found in Cargo.toml");
    }

    let hook_targets: Vec<String> = config.get("hook_targets")
                             .expect("No `hook_targets` value found.")
                             .as_array()
                             .expect("`hook_targets` is not an array.")
                             .iter()
                             .filter(|n| n.is_str())
                             .map(|n| n.as_str().unwrap().to_owned())
                             .collect();

    let config = Config::default().unwrap();
    config.shell().set_verbosity(Verbosity::Normal);
    let manifest_path = config.cwd().join("Cargo.toml");
    let workspace = Workspace::new(&manifest_path, &config).unwrap();
    let mut compile_config = CompileOptions::default(&config, CompileMode::Test);
    compile_config.filter = CompileFilter::new(false, &[],
                                               false, &hook_targets,
                                               false, &[],
                                               false, &[],
                                               false);

    let res = compile(&workspace, &compile_config).unwrap();
    res.tests.iter().map(|n| n.3.to_str().unwrap().to_owned()).collect()
}
