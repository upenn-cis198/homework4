// use super::system_call_names::SYSTEM_CALL_NAMES;
use std::collections::HashSet;
use std::error::Error;
use std::fmt;
use structopt::StructOpt;

#[derive(Debug)]
pub struct InvalidOption(String);

impl fmt::Display for InvalidOption {
    fn fmt(&self, _f: &mut fmt::Formatter) -> fmt::Result {
        unimplemented!()
    }
}

impl Error for InvalidOption {}

#[derive(StructOpt, Debug)]
#[structopt(
    name = "stracer",
    about = "Homework 4: a system call tracer written in Rust"
)]
pub struct Opt {
    #[structopt(short = "t", long = "to_trace")]
    pub to_trace: Vec<String>,

    #[structopt(short = "d", long = "dont_trace", conflicts_with = "to_trace")]
    pub dont_trace: Vec<String>,

    pub exe: String,
    pub exe_args: Vec<String>,
}
impl Opt {
    /// Validate that the input is correct:
    /// - Every string in to_trace should be a valid system call name
    ///   (in SYSTEM_CALL_NAMES)
    /// - Likewise, every string in dont_trace should be a valid system call
    ///   name
    /// - Feel free to add your own additional checks; to do so you would make
    ///   InvalidOption into an enum, to code the different errors.
    #[allow(dead_code)]
    pub fn validate(&self) -> Result<(), InvalidOption> {
        unimplemented!()
    }

    /// Return the set of system calls to trace:
    /// - The set to_trace, if given
    /// - Otherwise, the set of all system calls NOT in dont_trace, if given
    /// - Otherwise, the set of all system calls (trace everything)
    #[allow(dead_code)]
    pub fn syscalls_to_trace(&self) -> HashSet<&str> {
        unimplemented!()
    }
}
