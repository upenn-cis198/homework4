use super::system_call_names::SYSTEM_CALL_NAMES;

use byteorder::{LittleEndian, WriteBytesExt};
use libc::{c_long, c_void, user_regs_struct};
use nix::sys::ptrace::{self, AddressType, Options};
use nix::unistd::Pid;

/// Given an address in a tracee process specified by pid, read a string at
/// that address.
fn read_string(pid: Pid, address: AddressType) -> String {
    let mut string = String::new();
    // Move 8 bytes up each time for next read.
    let mut count = 0;
    let word_size = 8;

    'done: loop {
        let mut bytes: Vec<u8> = vec![];
        let address = unsafe { address.offset(count) };

        let res: c_long = ptrace::read(pid, address).unwrap_or_else(|err| {
            panic!("Failed to read data for pid {}: {}", pid, err);
        });
        bytes.write_i64::<LittleEndian>(res).unwrap_or_else(|err| {
            panic!("Failed to write {} as i64 LittleEndian: {}", res, err);
        });

        for b in bytes {
            if b != 0 {
                string.push(b as char);
            } else {
                break 'done;
            }
        }
        count += word_size;
    }

    string
}

/// Gven the pid of a process, set the tracing options
/// to trace the process
pub fn ptrace_set_options(pid: Pid) -> nix::Result<()> {
    let options = Options::PTRACE_O_TRACESYSGOOD
        | Options::PTRACE_O_TRACECLONE
        | Options::PTRACE_O_TRACEFORK
        | Options::PTRACE_O_TRACEVFORK
        | Options::PTRACE_O_TRACEEXIT
        | Options::PTRACE_O_TRACEEXEC;
    ptrace::setoptions(pid, options)
}

/// Given the pid of a process that is currently being traced,
/// return the registers for that process.
pub fn get_regs(pid: Pid) -> user_regs_struct {
    ptrace::getregs(pid).unwrap_or_else(|err| {
        panic!("Get regs failed: {:?}", err);
    })
}

/// Extract the system call name from the registers
pub fn extract_syscall_name(regs: user_regs_struct) -> &'static str {
    SYSTEM_CALL_NAMES[regs.orig_rax as usize]
}

/// Process a pre-hook event system call
pub fn handle_pre_syscall(regs: user_regs_struct, name: &str, pid: Pid) {
    if name == "execve" || name == "access" {
        let arg1 = regs.rdi as *mut c_void;
        let path = read_string(pid, arg1);
        print!("[{}]: {}(\"{}\") = ", pid, name, path);
    } else {
        print!("[{}]: {}() = ", pid, name);
    }
}

/// Process a post-hook event system call
pub fn handle_post_syscall(regs: user_regs_struct, _name: &str, _pid: Pid) {
    if (regs.rax as i32).abs() > 10000 {
        println!("0x{:x}", regs.rax as i32);
    } else {
        println!("{}", regs.rax as i32);
    }
}
