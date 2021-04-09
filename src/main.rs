// The command line struct from args.rs
// use args::Opt;

// These are the functions that you will need from nix and std:
// use nix::sys::ptrace;
// use nix::sys::signal::{raise, Signal};
// use nix::sys::wait::{waitpid, WaitStatus};
// use nix::unistd::{execvp, fork, ForkResult};
use nix::unistd::Pid;
// use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
// use std::ffi::CString;

fn main() -> Result<(), Box<dyn Error>> {

    // TODO

    Ok(())
}

// Code to be executed by the tracee (child process)
#[allow(dead_code)]
fn run_tracee(_exe: String, _exe_args: Vec<String>) -> nix::Result<()> {
    unimplemented!();
}

// Code to be executed by the tracer (parent process)
#[allow(dead_code)]
fn run_tracer(
    _child_pid: Pid,
    _syscalls_to_trace: &HashSet<&str>,
) -> nix::Result<()> {
    unimplemented!();
}
