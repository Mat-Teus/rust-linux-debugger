use std::{env, io, string};
use std::io::Write;
use nix::unistd::{fork, ForkResult, Pid};
use nix::sys::ptrace;
use nix::sys::wait::waitpid;
use std::collections::HashMap;
use nix::unistd::execv;
use std::ffi::CString;
use nix::sys::personality::{set,Persona,};

const N_REGISTERS:usize = 27;
const REGISTER_DESCRIPTION:[Register_Description; N_REGISTERS] = [
    Register_Description{
        r: Register::R15,
        dwarf_r: 15,
        name: "r15"
    },

    Register_Description{
        r: Register::R14,
        dwarf_r: 14,
        name: "r14"
    },

    Register_Description{
        r: Register::R13,
        dwarf_r: 13,
        name: "r13" 
    },

    Register_Description{
        r: Register::R12,
        dwarf_r: 12,
        name: "r12" 
    },

    Register_Description{
        r: Register::Rbp,
        dwarf_r: 6,
        name: "rbp" 
    },

    Register_Description{
        r: Register::Rbx,
        dwarf_r: 3,
        name: "rbx" 
    },

    Register_Description{
        r: Register::R11,
        dwarf_r: 11,
        name: "r11" 
    },

    Register_Description{
        r: Register::R10,
        dwarf_r: 10,
        name: "r10" 
    },

    Register_Description{
        r: Register::R9,
        dwarf_r: 9,
        name: "r9" 
    },

    Register_Description{
        r: Register::R8,
        dwarf_r: 8,
        name: "r8" 
    },

    Register_Description{
        r: Register::Rax,
        dwarf_r: 0,
        name: "rax" 
    },

    Register_Description{
        r: Register::Rcx,
        dwarf_r: 2,
        name: "rcx" 
    },

    Register_Description{
        r: Register::Rdx,
        dwarf_r: 1,
        name: "rdx" 
    },

    Register_Description{
        r: Register::Rsi,
        dwarf_r: 4,
        name: "rsi" 
    },

    Register_Description{
        r: Register::Rdi,
        dwarf_r: 5,
        name: "rdi" 
    },

    Register_Description{
        r: Register::Orig_rax,
        dwarf_r: -1,
        name: "orig_rax" 
    },

    Register_Description{
        r: Register::Rip,
        dwarf_r: -1,
        name: "rip" 
    },

    Register_Description{
        r: Register::Cs,
        dwarf_r: 51,
        name: "cs" 
    },

    Register_Description{
        r: Register::Rflags,
        dwarf_r: 49,
        name: "eflags" 
    },

    Register_Description{
        r: Register::Rsp,
        dwarf_r: 7,
        name: "rsp" 
    },

    Register_Description{
        r: Register::Ss,
        dwarf_r: 52,
        name: "ss" 
    },

    Register_Description{
        r: Register::Fs_base,
        dwarf_r: 58,
        name: "fs_base" 
    },

    Register_Description{
        r: Register::Gs_base,
        dwarf_r: 59,
        name: "gs_base" 
    },

    Register_Description{
        r: Register::Ds,
        dwarf_r: 53,
        name: "ds" 
    },

    Register_Description{
        r: Register::Es,
        dwarf_r: 50,
        name: "es" 
    },

    Register_Description{
        r: Register::Fs,
        dwarf_r: 54,
        name: "fs" 
    },

    Register_Description{
        r: Register::Gs,
        dwarf_r: 55,
        name: "gs" 
    },

];

#[derive(Clone,Copy, PartialEq)]
enum Register{
    Rax, Rbx, Rcx, Rdx,
    Rdi, Rsi, Rbp, Rsp,
    R8,  R9,  R10, R11,
    R12, R13, R14, R15,
    Rip, Rflags, Cs,
    Orig_rax, Fs_base,
    Gs_base,
    Fs, Gs, Ss, Ds, Es
}

//stores debugger state and target process information
struct Debugger{
    pid: Pid,
    program_name: String,
    breakpoints: HashMap<usize, Breakpoint>
}

struct Breakpoint{
    m_pid: Pid, //process where the break point is act
    m_addr: usize, //memory address of the proccess where INT3 will be inserted
    m_enabled: bool, //breakpoint is active or not
    m_saved_data: u8 //original byte overwritten by INT3
}

//Description of the register that will be used
struct Register_Description{
    name: &'static str,
    dwarf_r: i64,
    r: Register
}

impl Breakpoint{
    fn new(pid: Pid, addr: usize) -> Self{
        Self { m_pid: pid, m_addr: addr, m_enabled: false, m_saved_data: 0 } //creates a new breakpoint
    }

    fn is_enabled(&self) -> bool{
        return self.m_enabled; //returns if the breakpoint is enabled or not
    }

    fn get_address(&self) -> usize{
        return self.m_addr; //returns the address of memory where the breakpoint is set
    }

    fn enable(&mut self){
        let data = ptrace::read(self.m_pid, self.m_addr as ptrace::AddressType).expect("ptrace read falied");//reads the state of current process
        self.m_saved_data = (data & 0xff) as u8; //saves the byte that will be modified
        let int3 = 0xcc; //INT3 creation
        let data_with_int3 = (data & !0xff) | int3; //replaces data with the INT3 on the lowest byte
        ptrace::write(self.m_pid, self.m_addr as ptrace::AddressType, data_with_int3 as i64).expect("ptrace failed"); //writes the modified instruction with INT3 back to memory
        self.m_enabled = true;
    }

    fn disable(&mut self){
        let data = ptrace::read(self.m_pid, self.m_addr as ptrace::AddressType).expect("ptrace read falied");//reads the current satet of the process
        let restored_data = (data & !0xff) | self.m_saved_data as i64;//retores the original lowest byte data
        ptrace::write(self.m_pid, self.m_addr as ptrace::AddressType, restored_data).expect("ptrace failed");//writes the original instruction back to memory
        self.m_enabled = false;
    }
}

impl Debugger {
    //creates a new debugger instance for the target process
    fn new(program_name: String, pid: Pid) -> Self {
        Self {
            pid,
            program_name,
            breakpoints: HashMap::new()
        }
    }

    //Run the debugger
    fn run(&mut self) {
        //Waits until child process (debugee) stops
        match waitpid(self.pid, None){
            Ok(status) => {
                println!("{:?}", status);
            }

            Err(err) => {
                println!("Error");
            }
        }

        //main debugger command loop 
        loop{
            let mut command = String::new();

            print!("dbg> ");
            io::stdout().flush().unwrap();

            //reading command
            io::stdin().read_line(&mut command).expect("failed to read line");
            
            if !self.handle_command(command.trim()){
                break;
            }
        }
    }    

    fn set_a_breakpoint_at_a_address (&mut self, addr: usize){
        println!("Set breakpoint at 0x{:x}", addr);
        let mut bp = Breakpoint::new(self.pid, addr);//creates breakpoint using the address and the pid of the debugee process
        bp.enable();
        self.breakpoints.insert(addr, bp);//inserts breakpoint on the hashmap
    }

    fn handle_command(&mut self, command: &str) -> bool{
        //execution of debugger commands
        let args:Vec<&str> = command.split_whitespace().collect();//splits the command into arguments

        if args.is_empty(){
            return true;
        }

        match args[0]{
            //Quits the debugger
            "quit" => false,
            //Continues the execution of the debugee
            "continue" => {
                ptrace::cont(self.pid, None).expect("Continue failed");
                waitpid(self.pid, None).expect("waitpid failed");
                true
            }
            "breakpoint" => {
                //breakpoint at given memory address
                let addr = usize::from_str_radix(&args[1][2..], 16).expect("invalid address");//handles the first two chars
                self.set_a_breakpoint_at_a_address(addr); //sets a breakpoint at an address
                true
            }
            _ => {
                println!("Unknown command");
                true
            }
        }
    }

    fn get_register_value(&self, reg: Register) -> u64{
        let regs = ptrace::getregs(self.pid).expect("Failed to get register");
        match reg{
            Register::R15 => regs.r15,
            Register::R14 => regs.r14,
            Register::R13 => regs.r13,
            Register::R12 => regs.r12,
            Register::Rbp => regs.rbp,
            Register::Rbx => regs.rbx,
            Register::R11 => regs.r11,
            Register::R10 => regs.r10,
            Register::R9 => regs.r9,
            Register::R8 => regs.r8,
            Register::Rax => regs.rax,
            Register::Rcx => regs.rcx,
            Register::Rdx => regs.rdx,
            Register::Rsi => regs.rsi,
            Register::Rdi => regs.rdi,
            Register::Orig_rax => regs.orig_rax,
            Register::Rip => regs.rip,
            Register::Cs => regs.cs,
            Register::Rflags => regs.eflags,
            Register::Rsp => regs.rsp,
            Register::Ss => regs.ss,
            Register::Fs_base => regs.fs_base,
            Register::Gs_base => regs.gs_base,
            Register::Ds => regs.ds,
            Register::Es => regs.es,
            Register::Fs => regs.fs,
            Register::Gs => regs.gs,
        }
    }

    fn set_register_value(&self, reg: Register, value: u64){
        let mut regs = ptrace::getregs(self.pid).expect("Failed to get register");
        match reg{
            Register::R15 => regs.r15 = value,
            Register::R14 => regs.r14 = value,
            Register::R13 => regs.r13 = value,
            Register::R12 => regs.r12 = value,
            Register::Rbp => regs.rbp = value,
            Register::Rbx => regs.rbx = value,
            Register::R11 => regs.r11 = value,
            Register::R10 => regs.r10 = value,
            Register::R9 => regs.r9 = value,
            Register::R8 => regs.r8 = value,
            Register::Rax => regs.rax = value,
            Register::Rcx => regs.rcx = value,
            Register::Rdx => regs.rdx = value,
            Register::Rsi => regs.rsi = value,
            Register::Rdi => regs.rdi = value,
            Register::Orig_rax => regs.orig_rax = value,
            Register::Rip => regs.rip = value,
            Register::Cs => regs.cs = value,
            Register::Rflags => regs.eflags = value,
            Register::Rsp => regs.rsp = value,
            Register::Ss => regs.ss = value,
            Register::Fs_base => regs.fs_base = value,
            Register::Gs_base => regs.gs_base = value,
            Register::Ds => regs.ds = value,
            Register::Es => regs.es = value,
            Register::Fs => regs.fs = value,
            Register::Gs => regs.gs = value,
    }

        ptrace::setregs(self.pid, regs).expect("failed to set registers");
    }   

    fn get_register_value_from_dwarf_register(&self, regnum: i64) -> u64{
        let reg = REGISTER_DESCRIPTION.iter().find(|rd| rd.dwarf_r == regnum).expect("unknown dwarf register");
        return self.get_register_value(reg.r);
    }

    fn get_register_name(&self, reg: Register) -> &'static str{
        return REGISTER_DESCRIPTION.iter().find(|rd| rd.r == reg).expect("register unknown").name
    }

    fn get_register_from_name(&self, name: &str) -> Register {
        return REGISTER_DESCRIPTION.iter().find(|rd| rd.name == name).expect("register unknown").r
    }
}

fn main() {
    //The variable args saves the command line inputs
    let args:Vec<String> = env::args().collect();

    //Error message in case there's no minimum arguments
    if args.len() < 2{
        println!("Program name not specified");
        return;
    }

    //Stores program name
    let prog = &args[1];

    //Rust needs unsafe because of the fork, which is unsafe because only one thread is duplicated
    unsafe{
        //This parts of the code creates the core of the project, calling parent (debugger) and child (Program that will be debbuged)
        match fork() {
            Ok(ForkResult::Parent {child, ..}) => {
                //Parent process
                //Execute debugger
                println!("Starting debugging");
                let mut dbg = Debugger::new(prog.to_string(), child);
                dbg.run();
            }

            Ok(ForkResult::Child) => {
                //Child process
                //Execute debugee
                set(Persona::ADDR_NO_RANDOMIZE).expect("personality failed");//disabling pie.
                ptrace::traceme().expect("traceme failed");
                let prog_c = CString::new(prog.as_str()).expect("CString conversion failed");
                execv(&prog_c, &[prog_c.clone()]).expect("execv failed");
            }

            Err(_err) => {
                println!("Fork failed");
            }
        }
    }

}