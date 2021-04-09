# A Rust Stracer

A Rust `strace` utility, for logging all system calls a program produces.

## Overview

This is intended to be a shorter assignment, with most of the code provided for you.
The main goal is to familiarize yourself with some process and system call handling in Rust,
as provided by the `nix` crate.
In particular:

- Spawning subprocesses with `execvp`, `fork`, `ForkResult`, and `Pid`

- Waiting on processes with `waitpid` and `WaitStatus`

- Tracing processes with `ptrace`

There is some optional extra credit at the end of the assignment.

### Setup

The following files are provided: `main.rs`, `util.rs`, `system_call_names.rs`,
and `args.rs`.
Of these, you will be working on implementing the functionality in
`main.rs` and `args.rs`; you shouldn't need to modify the other two.

To start the assignment, run `Cargo init` to initialize the repository with Cargo.
There is already a `src` folder but you need `Cargo.toml`.
You will need to import the following crates in your `Cargo.toml` (under `[dependencies]`). Please use the specified versions:
```
nix = "0.20.0"
libc = "0.2.92"
structopt = "0.3.21"
byteorder = "1.4.3"
```

I've also added `rustfmt.toml`, feel free to modify or delete it for
different `cargo fmt` formatting settings.

## Assignment

This assignment is a minimal implementation of the `strace` Linux utility.
We recommend that you
play around with the utility if you're not familiar: just run `strace` followed by any command or program, and it will show what system calls that program is making. For example:
```
strace echo "hello"
strace cargo build
strace python3 my_python_program.py
strace ls -alh
```

Or even:
```
strace strace echo "hello"
```

Our stracer will intercept all system calls that a program makes.
Here a "program" is
understood as: a tree of processes, possibly running in parallel, possibly spawning
children. A sample output will look like:
```
$> cargo run -- ls
[11364]: rt_sigprocmask() = 0
[11364]: execve("/home/gatowololo/.cargo/bin/ls") = -2
[11364]: execve("/home/gatowololo/.cargo/bin//ls") = -2
[11364]: execve("/home/gatowololo/.cargo/bin/ls") = -2
[11364]: execve("/home/gatowololo/.local/bin/ls") = -2
[11364]: execve("/usr/local/sbin/ls") = -2
[11364]: execve("/usr/local/bin/ls") = -2
[11364]: execve("/usr/sbin/ls") = -2
[11364]: execve("/usr/bin/ls") = -2
[11364]: execve("/sbin/ls") = -2
[11364]: execve("/bin/ls") = 0
[11364]: brk() = 0xbe545000
[11364]: access("/etc/ld.so.nohwcap") = -2
[11364]: mmap() = 0x83364000
[11364]: access("/etc/ld.so.preload") = -2
[11364]: openat() = -2
[11364]: stat() = -2
[11364]: openat() = -2
[11364]: stat() = -2
...
[11364]: close() = 0
[11364]: fstat() = 0
Cargo.lock  Cargo.toml  src  target
[11364]: write() = 35
[11364]: close() = 0
[11364]: close() = 0
[11364]: exit_group() = Process finished!
```

Here we're stracing `ls` notice we have the pid (Process ID)
on the left column (formatted as `[pid]`),
the system call intercepted,
some system calls will print their string argument, and all system calls
have their return status next to them.

Our program will also allow the user to specify which system calls to intercept, or
which to ignore.
For example, the below only outputs `read` and `lstat` calls:
```
$> cargo run -- ls --to_trace lstat read -- -alh
[11547]: read() = 832
[11547]: read() = 832
[11547]: read() = 832
[11547]: read() = 832
[11547]: read() = 832
[11547]: read() = 416
[11547]: read() = 0
[11547]: read() = 2995
[11547]: read() = 0
[11547]: lstat() = 0
[11547]: read() = 545
[11547]: read() = 0
[11547]: read() = 832
[11547]: read() = 832
[11547]: read() = 832
[11547]: read() = 832
[11547]: lstat() = 0
[11547]: lstat() = 0
[11547]: lstat() = 0
[11547]: lstat() = 0
[11547]: lstat() = 0
[11547]: lstat() = 0
[11547]: lstat() = 0
total 48K
[11547]: read() = 3545
[11547]: read() = 2261
...
```

When passing arguments to `cargo run`, remember that `--`
separates arguments to `cargo` from arguments used by our Rust program.
Above, we are using a second `--` to separate additional arguments to be passed
to the command we are running.
So the pattern is `cargo run -- <command to trace> <arguments to stracer> -- <arguments to command>`.

## Background

### Ptrace

To implement the tracer, we rely on
`ptrace`, a special system call which allows us to trace other processes
executions.
We will write our tracer with two processes: a parent
process (this is the tracer which will print events) and a child process (the tracee)
which executes the passed command.
The tracer acts like a daemon, waiting for event to come from the tracee. The tracee
is stopped while the tracer handles the event.

A few types of events cause `ptrace` stops: system calls, clone events, exit event,
signals, etc. We will mostly be interested in tracing system calls.
For system calls events, `ptrace` receives an event **before and after a system call; we refer to these events as pre-hook and post-hook events.**
So you will be notified of a `ptrace` stop twice for each system call,
not just once.
In `util.rs`, we have provided code which traces the system call
at the beginning and end:
`handle_pre_syscall` and `handle_post_syscall`.

### Nix

Process management is done with the `nix` crate.
To help you filter out the parts of the `nix` API
and the standard library that are relevant,
in `main.rs` we have imported
exactly the functions that we use in our implementation
(commented out to suppress errors):
```
use nix::sys::ptrace;
use nix::sys::signal::{raise, Signal};
use nix::sys::wait::{waitpid, WaitStatus};
use nix::unistd::{execvp, fork, ForkResult, Pid};
use std::collections::HashMap;
use std::collections::HashSet;
use std::error::Error;
use std::ffi::CString;
```

Specifically, you should become
best friends with `nix::sys::wait::WaitStatus` enum as you will need to handle all
the cases in this enum.

**Important Note:**
You must propagate all `nix::Result` errors up to main using `?`; do not use
`unwrap` or `expect` on these. You may use `unwrap` or `expect` on other types of error though.

## Detailed Instructions

### Step 1: Parse command line arguments

Your program will take the executable and arguments for another program through
the command line. As we did in homework2, we will use the rust crate (structopt)[https://docs.rs/structopt/0.2.12/structopt/] for argument parsing.

Inside `args.rs`,
the command line arguments struct `Opt`, deriving `StructOpt`,
is provided for you.
Running the program with `--help` returns:
```
./target/debug/stracer --help
stracer 0.1.0

A simple stracer written in Rust

USAGE:
    stracer [OPTIONS] <exe> [--] [exe_args]...

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -d, --dont_trace <dont_trace>...
    -t, --to_trace <to_trace>...

ARGS:
    <exe>
    <exe_args>...
```

As shown above, there are only four command-line arguments,
`to_trace`, `dont_trace`, `exe`, and `exe_args`.
Of the first two, at most one should be provided:
if `to_trace` is provided, it lists a set of system call names to trace
(i.e. ignore all others),
and if `dont_trace` is provided, it lists a set of system call names to
NOT trace (i.e. trace all others).
Finally, `exe` gives the name of the command to trace (e.g. `ls`),
and `exe_args` gives the list of arguments to that command (e.g. `["-alh"]`).

- Please implement the function
`validate`, which checks that the input arguments are correct:
in particular that all the system call names are valid names in
`system_call_names.rs`.
Use the `InvalidOption` struct to indicate an error.

- Also, implement `Display` for `InvalidOption`.

- Finally implement `syscalls_to_trace`, which returns a `HashSet`
  of the system calls to be traced by the call; this should make use of
  `to_trace`, `dont_trace`, and the list of system call names in
  `system_call_names.rs`.
  If neither `to_trace` or `dont_trace` is provided,
  return the set of all system calls.

In the `main()` function in `main.rs`, add code which initializes the command
line options struct from the command line arguments (StructOpt's `::from_args()`),
then use your `.validate()` function with `?` to propagate any errors.

You can also print out the command line arguments to see that they are being parsed correctly, and print out your `syscalls_to_trace` to see that it is working.

### Step 2: Implement the trace-ee (execute a subprocess)

Inside `main.rs`, you have three functions to implement:
`main()`, `run_tracee`, and `run_tracer`.

For this part, inside `main()` (after command line arguments are parsed and validated), use the unsafe **fork** function (`nix::unistd::fork`) to fork into a child and a parent; call `run_tracer` on the parent, and `run_tracee` on the child.

The child should do the following things:

1. Call `ptrace traceme()` to set itself up for being traced.

2. Raise the `SIGSTOP` signal (see `nix::sys::signal::{raise, Signal};`).
   This basically has the child stop itself until the tracer is ready.
   This ensures the tracer has time to get set up, otherwise there is a race condition
   between parent and child.

3. Once the child is continued by the parent, use the `execvp` function
   (this is one of the variants of the `exec` system call)
   to execute the child process from a command and list of arguments.
   See documentation [here](https://docs.rs/nix/0.20.0/nix/unistd/fn.execv.html)

   To call `execvp` you first need to convert the string arguments
   to `CString`, to ensure they are valid C strings.
   The type `std::ffi::CString` is imported for you.
   See `CString::new`.

### Step 3: Implement the tracer (trace a subprocess)

The last function to implement is `run_tracer`.
As we saw in (2) above,
the tracee starts in a stopped state waiting for the tracer to let it continue.
In general we use `ptrace(syscall)` to allow the tracee to continue, after that we wait
for the next `ptrace` event using `waitpid`, `waitpid` returns a `WaitStatus` which we can match
on to do many things based on the type of event.

`waitpid` is called like: `waitpid(child pid, None)` or,
to wait on ANY child process ID, `wiatpid(None, None)`.
See [documentation](https://docs.rs/nix/0.20.0/nix/sys/wait/fn.waitpid.html).

The parent should do the following things.

1. Wait for child to be ready by calling `waitpid` on it's `pid` (this waits on the child to send the `SIGSTOP` to itself (see child actions above)).

2. Call `ptrace_set_options` (provided to you in `util`) to properly set up `ptrace` to track all the events we're interested in.

3. Call `ptrace(syscall, child_pid)` to let it continue (step 3 on the child).

4. In a loop, repeatedly wait using `waitpid` on ANY child process ID.
For this you should set up a tracing loop like this:
```Rust
loop {
    // Wait for any event from any tracee to come.
    let actual_pid = match waitpid(None, None) {
      // Handle all WaitStatus events.
      Event1 => {
       ...
      }

      // The current process has exited.
      Exited => {

      }

      // A system call event.
      SystemCallEvent => {
          // This is the important part
      }
    }

    // Allow this process to continue running and loop around for next event.
    ptrace::syscall(pid, None);
}
```

Notice a few things:

- There is no difference between a post-hook event and a pre-hook event,
  you will have to keep track of this using `bool`, alternating between them as
  `SystemCallEvent` come.

- When does this loop end? When we hit the `Exited` you can break out of the loop.

To process the system call events,
you will need `extract_syscall_name` (provided in `util`)
to get the system call name.
You can start out just by printing out to see how it looks.
Then
use
`handle_pre_syscall` to process pre-hook events
and `handle_post_syscall` to process post-hook events.

- Make sure you ONLY process the system call names that are being traced (in the given hash set that you computed from the command line arguments in step 1).

- Besides `extract_syscall_name`, you also need `get_regs` (provided in `util`)
  to pass to the `handle_pre_syscall` and `handle_post_syscall` tracing functions.

- All of the actual tracing is done for you in these functions, but take a look if you are interested; it involves parsing the registers of the subprocess to get the arguments and return value of each system call. There are also more details under "technical details" below.

## Design and Implementation

Make sure that you run `cargo clippy` and `cargo fmt` and deal with all warnings (yellow text, not just red text) before submitting.

**Unit tests are not required for this assignment.**
It is very difficult to unit test this type of code, instead
you should rely on running examples to make sure they work,
and logging or `println!` statements to debug.

## Extra Credit

### Extra Credit 1 (20pts): Logging

Use `env_logger` to print helpful messages about what
is happening in your code. Use the `info!` macro to print extra messages.
Add the following to your `Cargo.toml`:

```
log = "0.4.14"
env_logger = "0.8.3"
```

Once these are important, all you have to do to add logging is import
the logging function you want, typically
`use log::info;`
then use the `info!` macro:
```Rust
info!("{This is a logging statement with formatting {} and more {}", arg1, arg2)
```

Add logging throughout the program (in `main.rs`, `args.rs`, and `util.rs`),
wherever you see fit, or where there are interesting events.
An example
possible output of the logger is below:
```
$> RUST_LOG=stracer=info cargo run ls
 INFO 2018-10-16T17:14:56Z: stracer: Pre-hook event. Nothing to do.
 INFO 2018-10-16T17:14:56Z: stracer: Post-hook event.
[12023]: rt_sigprocmask() = 0
...
[12023]: execve("/usr/sbin/ls") = -2
 INFO 2018-10-16T17:14:56Z: stracer: Pre-hook event. Nothing to do.
 INFO 2018-10-16T17:14:56Z: stracer: Post-hook event.
[12023]: execve("/usr/bin/ls") = -2
 INFO 2018-10-16T17:14:56Z: stracer: Pre-hook event. Nothing to do.
 INFO 2018-10-16T17:14:56Z: stracer: Post-hook event.
[12023]: execve("/sbin/ls") = -2
 INFO 2018-10-16T17:14:56Z: stracer: Pre-hook event. Nothing to do.
 INFO 2018-10-16T17:14:56Z: stracer: [12023] Ptrace Event 4
 ```

Your logger should similarly print messages for the following events:
pre-hook events, post-hook events, exit, signaled, `ptrace` events, stopped, etc.
For example: `info!("[{}] Process Killed by Signal {:?}", pid, signal);`

You could also print out the string argument for the following system calls:
`execve`, `access`, `stat`, `lstat`, `chdir` (feel free to add more).
To do this, edit `handle_pre_syscall`.

### Extra Credit 2 (20pts): Making your tracer work for multiple processes

What we did above (the single loop logger)
works fine for a single process, but won't work if the process you are
tracing spawns its own processes!
To make it work for multiple processes, we should only need
to edit the `run_tracer` main loop.

For this you will need a few more things:

1. You cannot just break out of the loop when a single process exits. You must know all
   live processes are done, we will need to keep track of this. We recommend a `HashMap`.
   You should add new processes when you see them, for simplicity, do this at the
    `SystemCallEvent` branch. Delete it once it has exited.

2. A single boolean isn't enough to keep track of the post-hook/pre-hook events; you
   will need a boolean per live process. We recommend a `HashMap` for this. As in (1), for
   never seen processes add an entry in `PtraceSyscall` with the starting value for
   pre-hook event.

You can do both (1) and (2) with a single `HashMap` of the known live processes,
where the key is the process ID and the value is the Boolean of whether it is
in a pre-hook or post-hook stage.

**Testing this:** An easy command to run which spawns child processes is `bash -c "ls"` if that works,
you can try slightly more complicated variant: `bash -c "ls && ls -ahl"`.

**Note**: Getting `ptrace` to work for all processes is very difficult! For example,
our implementation does not properly propagate signals. Do not expect your final solution
to work for all programs.

## Technical details (you can skip this)

Here are some more technical details about what's going on in `util.rs` for the curious; you can skip this otherwise.

### System call return values

The provided tracing code prints the *arguments* and *return value* for system calls done by the tracee.
When printing a return value,
most system calls return a result status (typically a small integer like 0 or -1)
as an argument, but not all. Consider:
```
[11942]: close() = 0
[11942]: openat() = -2
[11942]: openat() = -2
[11942]: openat() = -2
[11942]: openat() = 3
[11942]: fstat() = 0
[11942]: mmap() = 0x695ef000
```

`mmap` returns the address of memory that was mapped, so we follow a simple heuristic:
If the absolute value of a return value is above 10,000 we assume it is an address and
print it using hexadecimal format.

### Registers of a stopped process

While the tracee is stopped, we can get it's registers to see the state of the program,
the system call which was intercepted is defined by the `orig_rax` field, notice this isn't a real CPU register but a Linux quirk. The arguments to
the system call are mapped to the following registers: `arg1 => rdi`, `arg2 => rsi`,
`arg3 => rdx`, `arg4 => r10`, `arg5 => r8`, `arg6 => r9`.
The function `get_regs` gets the register information about the tracee
and this is how we figure out the system call arguments and return value.

Similarly, the tracer can read/write to arbitrary memory of the tracee.
This is how
the provided `read_string` function works;
it reads from the memory of the tracee,
in sequences of bytes.

### Byteorder

The byteorder crate is used by the provided `util.rs` code
to deal with reading
bytes in `read_string`: this is because
we have to distinguish between little-endian order and big-endian order.
This is done through the ByteOrder *trait*,
which describes types that can be used to read/write integers
in either of these orders.
See [here](https://docs.rs/byteorder/1.4.3/byteorder/trait.ByteOrder.html).

## Deadline

This assignment is due on Wednesday, April 20 at 11:59pm.
