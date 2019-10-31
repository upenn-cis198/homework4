## A Rust Stracer

A rust strace utility, for logging all system calls a program produces.

### Setup
You will find files: `util.rs` and `system_call_names.rs` which contains functions
you will need. These should be copied over to your `src/` directory.

You will need the following crates. Please use the specified versions:
```
nix = "0.10"
libc = "0.2"
structopt = "0.2"
byteorder = "1"
log = "0.4"
env_logger = "0.5"
```

### Assignment
This assignment is difficult! Start early!

This will be a minimal implementation of the strace Linux utility. I recommend you
play around with the utility if you're not familiar.

Our stracer will intercept all system call that a program makes, here a "program" is
understood as: a tree of processes, possibly running in parallel, possibly spawning
children. A sample output will look like:

```
$> cargo run ls
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

Here we're stracing `ls` notice we have the pid on the left column, the system call
intercepted, some system calls will print their string argument, and all system calls
have their return status next to them.

Our program will also allow the user to specify which system calls to intercept, or
which to ignore, for example:
```
$> ./target/debug/stracer ls --to_trace lstat read -- -ahl
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

Only outputs `read` and `lstat` calls.

### Background
`ptrace` is the underlying system call which allows us to trace other processes
executions. Just like in the previous Rust shell assignment, there will be a parent
process (this is the tracer which will print events) and a child process (the tracee)
which calls `execve` based on the passed command.

The tracer acts like a daemon, waiting for event to come from the tracee. The tracee
is stopped while the tracer handles the event.

A few types of events cause ptrace stops: system calls, clone events, exit event,
signals, etc. We will mostly be interested in tracing system calls.

For system calls events, ptrace receives an event before and after a system call,
we refer to these events as pre-hook and post-hook events.

While the tracee is stopped, we can get it's registers to see the state of the program,
the system call which was intercepted is defined by the `orig_rax` field, notice this isn't a real CPU register but a Linux quirk. The arguments to
the system call are mapped to the following registers: arg1 => rdi, arg2 => rsi,
arg3 => rdx, arg4 => r10, arg5 => r8, arg6 => r9.

Similarly, the tracer can read/write to arbitrary memory of the tracee (this is how
my `read_string` function works.

### Input
Your program will take the executable and arguments for another program through
the command line. We will use the rust crate (structopt)[https://docs.rs/structopt/0.2.12/structopt/] for argument parsing, please see section below.

Running the program with no arguments returns a helpful message of what the input
should look like:
```bash
$> ./target/debug/stracer
error: The following required arguments were not provided:
    <exe>

USAGE:
    stracer [OPTIONS] <exe> [--] [exe_args]...

For more information try --help
```

That is, a command to our program looks like: `./stracer <exe> [--] [exe_args]`, that
is, a mandatory executable file, with optional arguments to `exe` separated by `--`.

Some valid example commands could be:
```
./stracer ls
./stracer ls -- -ahl
```

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

Specifically the options flags which allow us select which system calls to trace
or which ones not to trace. These two flags cannot be used together.

Unfortunately the ordering of commands with optional flags is a little awkward:
`./target/debug/stracer ls --dont_trace read -- -ahl`

More information is given below on the structopt section.

### Output
The output should be as shown in the examples above. Do not worry about small
differences in whitespace, but in general it should look like the examples provided.

In this repo, is also a file sample_output.txt showing what the output for a full run
looks like. Notice the output of our tool can be intertwined with the output of the
program running, this is okay.

Most system calls return a result status as an argument, but not all. Consider:
```
[11942]: close() = 0
[11942]: openat() = -2
[11942]: openat() = -2
[11942]: openat() = -2
[11942]: openat() = 3
[11942]: fstat() = 0
[11942]: mmap() = 0x695ef000
```

mmap returns the address of memory that was mapped, so we follow a simple heuristic:
If the absolute value of a return value is above 10,000 we assume it is an address and
print it using hexadecimal format.

It is very difficult to unit test this type of code, instead we will rely on a detailed
logger to debug this program.

Please use `env_logger` (same as last assignment) to print helpful messages about what
is happening in your code. Use the `info!` macro to print extra messages. An example
output of my logger is:

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
pre-hook events, post-hook events, exit, signaled, ptrace events, stopped, etc.
For example: `info!("[{}] Process Killed by Signal {:?}", pid, signal);`

Additionally you must print the string argument for the following system calls:
`execve`, `access`, `stat`, `lstat`, `chdir` (feel free to add more for funsies).

## Design and Implementation

Aside: It is not enough for your code to work, it should be well written, idiomatic,
logically separated, and modular. Even if every works, you **will not** receive an A on
this assignment if your code is terribly written, e.g. if-else cascades, one giant main
function.

**NOTE**: You must propagate all nix::Result errors up to main using '?', do not use
unwrap or expect on these, you may use unwrap or expect on **other** types of error though.

### Nix
We will use the nix crate for their ptrace bindings. Specifically you should become
best friends with `nix::sys::wait::WaitStatus` enum as you will need to handle all
the cases in this enum.

I have implemented a couple of utility functions, which are difficult to write:

```rust
/// Given an address in a tracee process specified by pid, read a string at
/// that address.
pub fn read_string(address: *mut c_void, pid: Pid) -> String;

/// Nix does not yet have a way to fetch registers. We use our own instead.
/// Given the pid of a process that is currently being traced. Return the registers
/// for that process.
pub fn get_regs(pid: Pid) -> user_regs_struct;
```

You will find these useful.

### Structopt
Structopt allows us to declare a struct which specifies our program's command line
arguments. Structopt takes care of the parsing, and returns an instance of this struct
which we can simply `instance.field` to get the arguments. Familiarize yourself with
this crate and create the command line arguments as specified in the input section.

Please put this struct in it's own module and file called `args.rs`. Declare a type
called `Opt` which defines your command line args. You may find the structopt
per field attributes: "short", "long", "conflicts_with", useful.

Notice the type of the field tells structopt how to parse the commands. My `dont_trace`,
and `to_trace` are of type `Vec<String>`.

### Byteorder
The byteorder trait is used by my provided `util.rs` code, not need to worry about it.


### Using Ptrace
Ptrace is quite complicated. I will attempt to give some brief pointers here, in class
I will cover the general algorithm.

The tracee starts in a stopped state waiting for the tracer to let it continue.
In general we use ptrace(syscall) to allow the tracee to continue, after that we wait
for the next ptrace event using waitpid, waitpid returns a WaitStatus which we can match
on to do many things based on the type of event.

#### Starting ptrace
The child should do the following things:
1) Call ptrace traceme() to set itself up for being traced.
2) Call raise(SIGSTOP) this basically has the child stop itself until the tracer is ready.
   This ensures the tracer has time to get set up, otherwise there is a race condition
   between parent and child. Notice child is stopped here.
3) Child is continued by parent... (See parent actions below)
4) Child calls execvp to run process.

The parent should do the following things.
1) Wait for child to be ready by calling waitpid on it's pid (this waits on the child to send the SIGSTOP to itself (see child actions above)).
2) Call ptrace_set_options (provided by me) to properly set up ptrace to track all the
   events we're interested in.
3) Call ptrace(syscall, child_pid) to let it continue (step #3 on the child).
4) From here the parent and child are ready to work together to trace system calls.

You should set your system tracing loop as follows:
```rust
loop {
    // Wait for any event from any tracee to come.
    let actual_pid = match waitpid(any_pid) {
      // Handle all WaitStatus events.
      Event1 => {
       ...
      }

      // The current process has exited.
      Exited => {

      }

      // A system call event.
      SystemCallEvent => {

      }
    }

    // Allow this process to continue running and loop around for next event.
    ptrace(syscall, actual_pid);
}

```

Notice a few things:
- There is no difference between a post-hook event and a pre-hook event,
  you will have to keep track of this using bool, alternating between them as
  SystemCallEvent come.
- When does this loop end? When we hit the Exited even you can break out of the loop.

This works fine for a single process, but won't scale to multiple processes.
For this you will need a few more things:
1) You cannot just break out of the loop when a single process exits. You must know all
   live processes are done, we will need to keep track of this. I recommend a HashSet.
   You should add new processes when you see them, for simplicity, do this at the
    SystemCallEvent branch. Delete it once it has exited.
2) A single boolean isn't enough to keep track of the post-hook/pre-hook events, you
   will need a boolean per live process, I recommend a HashMap for this. As in #1, for
   never seen processes add an entry in PtraceSyscall with the starting value for
   pre-hook event.

An easy command to run which spawns child processes is `bash -c "ls"` if that works,
you can try slightly more complicated variant: `bash -c "ls && ls -ahl"`.

**Note**: Getting ptrace to work for all processes is very difficult! For example,
our implementation does not properly propagate signals. Do not expect your final solution
to work for all programs.

## Recommended Steps for implementation
1) Start with a simple command (say ls), and hard code this command into the child. Ensure
   you're able to properly ptrace the child, iterating through it's system calls all
   the way to completion. The only ptrace events you should need to handle at this step
   is the Exited and PtraceSyscall event.
2) Implement the command line argument options to allow arbitrary processes to be passed
   in along with that process's command line arguments. You should only attempt to run
   programs which themselves don't fork, like `ls`.
3) Implement multi-processing, this requires a few extensions as explained above. You
   must be able to handled arbitrary events from arbitrary processes.
4) Lastly, add the functionality to print only specific events based on what the user
   provided, I found the HashSet useful for this task.
