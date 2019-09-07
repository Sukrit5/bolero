use crate::{exec, Config, FuzzArgs, ShrinkArgs};
use std::fs;

macro_rules! optional_arg {
    ($cmd:ident, $arg:expr, $fmt:expr) => {
        if let Some(value) = $arg {
            $cmd.arg(&format!($fmt, value));
        }
    };
}

const FLAGS: &[&str] = &[
    "--cfg fuzzing_libfuzzer",
    "-Cllvm-args=-sanitizer-coverage-trace-compares",
    "-Cllvm-args=-sanitizer-coverage-trace-divs",
    "-Cllvm-args=-sanitizer-coverage-trace-geps",
];

pub(crate) fn fuzz(config: &Config, fuzz: &FuzzArgs) {
    let workdir = config.workdir();
    let mut cmd = config.cmd("test", FLAGS);
    let corpus_dir = workdir.join("corpus");
    let crashes_dir = workdir.join("crashes");

    fs::create_dir_all(&crashes_dir).unwrap();

    cmd.arg(corpus_dir)
        .arg(&format!("-artifact_prefix={}/", crashes_dir.display()));

    optional_arg!(cmd, fuzz.seed, "-seed={}");
    optional_arg!(cmd, fuzz.runs, "-runs={}");
    optional_arg!(cmd, fuzz.time, "-max_total_time={}");
    optional_arg!(cmd, fuzz.max_input_length, "-max_len={}");

    // TODO figure out log file location
    optional_arg!(cmd, fuzz.jobs, "-jobs={}");

    exec(cmd).exit_on_error();
}

pub(crate) fn shrink(config: &Config, _shrink: &ShrinkArgs) {
    let workdir = config.workdir();
    let mut cmd = config.cmd("test", FLAGS);
    let corpus_dir = workdir.join("corpus");

    let tmp = tempfile::tempdir().expect("could not create tmpdir");
    let tmp_corpus = tmp.path().join("corpus");
    fs::create_dir(&tmp_corpus).unwrap();

    cmd.arg(&tmp_corpus)
        .arg(&corpus_dir)
        .arg("-shrink=1")
        .arg("-merge=1")
        .arg("-reduce_inputs=1");

    let status = exec(cmd);

    fs::remove_dir_all(&corpus_dir).unwrap();
    fs::rename(&tmp_corpus, &corpus_dir).unwrap();

    status.exit_on_error()
}
