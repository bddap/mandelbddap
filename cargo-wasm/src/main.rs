use std::path::{Path, PathBuf};
use std::process::Command;

use structopt::StructOpt;

const PROGRAM_NAME: &str = "mandelbddap";

#[derive(Debug, StructOpt)]
enum Args {
    /// Build the required artifacts to run the program on the web.
    Build(Build),
    /// Build the required artifacts then serve them locally.
    Serve(Serve),
}

#[derive(Debug, StructOpt)]
struct Build {
    #[structopt(short, long)]
    release: bool,
}

#[derive(Debug, StructOpt)]
struct Serve {
    #[structopt(flatten)]
    build: Build,
    #[structopt(short, long, default_value = "localhost")]
    host: String,
    #[structopt(short, long, default_value = "8000")]
    port: u16,
}

impl Build {
    fn run(&self) {
        if !wasm_target_is_installed() {
            println!(
                "You are going to need to install the wasm32-unknown-unknown target.\n\
                 This command should do the trick:\n  \
                 rustup target add wasm32-unknown-unknown"
            );
            std::process::exit(1);
        }

        let profile = if self.release { "release" } else { "debug" };

        let mut cargo_args = vec!["build", "--target", "wasm32-unknown-unknown"];
        if self.release {
            cargo_args.push("--release");
        }
        let status = Command::new(&"cargo")
            .current_dir(&project_root())
            .args(&cargo_args)
            .status()
            .unwrap();
        if !status.success() {
            std::process::exit(status.code().unwrap());
        }

        // run wasm-bindgen on wasm file output by cargo, write to the destination folder
        let wasm_source = Path::new("target/wasm32-unknown-unknown")
            .join(profile)
            .join(format!("{PROGRAM_NAME}.wasm"));

        std::fs::create_dir_all(dest()).unwrap();
        let mut bindgen = wasm_bindgen_cli_support::Bindgen::new();
        bindgen
            .web(true)
            .unwrap()
            .omit_default_module_path(false)
            .input_path(&wasm_source)
            .generate(dest())
            .unwrap();

        // process template index.html and write to the destination folder
        let index_text = include_str!("index.html");
        std::fs::write(dest().join("index.html"), index_text).unwrap();

        println!("Wrote static web assets to {}", dest().to_str().unwrap());
    }
}

impl Serve {
    fn run(&self) {
        self.build.run();
        println!("Serving on http://{}:{}", self.host, self.port);
        devserver_lib::run(
            &self.host,
            self.port as u32,
            dest().as_os_str().to_str().unwrap(),
            false,
            "",
        );
    }
}

/// The root of this cargo workspace.
fn project_root() -> PathBuf {
    Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap())
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

/// the directory in which to store output
fn dest() -> PathBuf {
    project_root().join("target").join("dist")
}

fn wasm_target_is_installed() -> bool {
    let out = Command::new("rustup")
        .args(["target", "list", "--installed"])
        .output()
        .unwrap()
        .stdout;
    std::str::from_utf8(&out)
        .unwrap()
        .lines()
        .any(|l| l == "wasm32-unknown-unknown")
}

fn main() {
    match Args::from_args() {
        Args::Build(b) => b.run(),
        Args::Serve(s) => s.run(),
    }
}
