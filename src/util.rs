use byteorder::{LittleEndian, WriteBytesExt};
use libc::{c_long, user_regs_struct};
use nix::sys::ptrace::{self, AddressType, Options};
use nix::unistd::Pid;

/// Given an address in a tracee process specified by pid, read a string at
/// that address.
pub fn read_string(pid: Pid, address: AddressType) -> String {
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

pub fn ptrace_set_options(pid: Pid) -> nix::Result<()> {
    let options = Options::PTRACE_O_TRACESYSGOOD
        | Options::PTRACE_O_TRACECLONE
        | Options::PTRACE_O_TRACEFORK
        | Options::PTRACE_O_TRACEVFORK
        | Options::PTRACE_O_TRACEEXIT
        | Options::PTRACE_O_TRACEEXEC;
    ptrace::setoptions(pid, options)
}

/// Given the pid of a process that is currently being traced, return the registers
/// for that process.
pub fn get_regs(pid: Pid) -> user_regs_struct {
    ptrace::getregs(pid).unwrap_or_else(|err| {
        panic!("Get regs failed: {:?}", err);
    })
}
