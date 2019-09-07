use crate::manifest::resolve;
use std::{
    fs::{self, OpenOptions},
    io::Write,
    path::Path,
};
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
pub struct New {
    /// Name of the test target
    test: String,

    /// Package to run tests for
    #[structopt(short = "p", long = "package")]
    package: Option<String>,

    /// Path to Cargo.toml
    #[structopt(long = "manifest-path")]
    manifest_path: Option<String>,

    /// Generates a generator test
    #[structopt(short = "g", long = "generator")]
    generator: bool,
}

const FUZZ_FILE: &str = r#"
use bolero::fuzz;

fuzz!(|input| {
    if input.len() < 3 {
        return;
    }

    if input[0] == 0 && input[1] == 1 && input[2] == 2 {
        panic!("you found me!");
    }
});
"#;

const GENERATOR_FILE: &str = r#"
use bolero::{fuzz, generator::*};

fuzz!(for value in each(u8::gen()) {
    assert!(value * 2 > value);
});
"#;

impl New {
    pub fn exec(&self) {
        let file = if self.generator {
            GENERATOR_FILE
        } else {
            FUZZ_FILE
        }
        .trim_start();

        let manifest_path = resolve(&self.manifest_path, &self.package);
        let project_dir = manifest_path.parent().unwrap();
        let target_dir = project_dir.join("tests").join(&self.test);

        mkdir(&target_dir);
        write(target_dir.join("main.rs"), file);

        mkdir(target_dir.join("corpus"));
        write(target_dir.join("corpus").join(".gitkeep"), "");
        mkdir(target_dir.join("crashes"));
        write(target_dir.join("crashes").join(".gitkeep"), "");

        let mut cargo_toml = OpenOptions::new()
            .append(true)
            .open(manifest_path)
            .expect("could not open Cargo.toml");

        cargo_toml
            .write_all(
                format!(
                    r#"
[[test]]
name = "{name}"
path = "tests/{name}/main.rs"
harness = false
"#,
                    name = self.test
                )
                .as_ref(),
            )
            .expect("could not write test config");

        println!("Created {:?}", &self.test);
    }
}

fn mkdir<P: AsRef<Path>>(path: P) {
    fs::create_dir_all(path).expect("could not create test directory");
}

fn write<P: AsRef<Path>, C: AsRef<[u8]>>(path: P, contents: C) {
    let path = path.as_ref();
    fs::write(path, contents).expect("could not create file");
    println!("wrote {:?}", path);
}
