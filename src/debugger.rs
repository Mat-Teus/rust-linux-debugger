use std::{io};
use std::io::Write;
use nix::unistd::{ForkResult, Pid};
use nix::sys::ptrace;
use nix::sys::wait::waitpid;
use std::collections::HashMap;
use nix::unistd::execv;
use std::ffi::CString;
use nix::sys::personality::{Persona,};
use crate::registers::*;
use crate::breakpoint::*;

//stores debugger state and target process information
pub struct Debugger{
    pid: Pid,
    program_name: String,
    breakpoints: HashMap<usize, Breakpoint>
}

impl Debugger {
    //creates a new debugger instance for the target process
    pub fn new(program_name: String, pid: Pid) -> Self {
        Self {
            pid,
            program_name,
            breakpoints: HashMap::new()
        }
    }

    //Run the debugger
    pub fn run(&mut self) {
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

    pub fn read_memory(&self,address:usize) -> i64{
        return ptrace::read(self.pid, address as ptrace::AddressType).expect("memory read failed");
    }

    pub fn write_memory(&self,address:usize,value:i64){
        ptrace::write(self.pid, address as ptrace::AddressType, value ).expect("memory write failed");
    }

    pub fn set_a_breakpoint_at_a_address (&mut self, addr: usize){
        println!("Set breakpoint at 0x{:x}", addr);
        let mut bp = Breakpoint::new(self.pid, addr);//creates breakpoint using the address and the pid of the debugee process
        bp.enable();
        self.breakpoints.insert(addr, bp);//inserts breakpoint on the hashmap
    }

    pub fn get_pc(&self) -> u64{
        return self.get_register_value(Register::Rip);
    }

    pub fn set_pc(&self, pc:u64){
        self.set_register_value(Register::Rip, pc);
    }

    pub fn step_over_breakpoint(&mut self){
        let possible_breakpoint_position = (self.get_pc() - 1) as usize;

        if self.breakpoints.contains_key(&possible_breakpoint_position){
            if self.breakpoints.get(&possible_breakpoint_position).unwrap().is_enabled(){
                let previous_instruction_address = possible_breakpoint_position;
                self.set_pc(previous_instruction_address as u64);

                self.breakpoints.get_mut(&possible_breakpoint_position).unwrap().disable();

                ptrace::step(self.pid, None);
                waitpid(self.pid, None);

                self.breakpoints.get_mut(&possible_breakpoint_position).unwrap().enable();
            }
        }
    }

    pub fn wait_for_signal(&self){
        let status = waitpid(self.pid, None).expect("error");
        println!("{:?}", status);
    }

    pub fn handle_command(&mut self, command: &str) -> bool{
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
                self.step_over_breakpoint();
                ptrace::cont(self.pid, None).expect("Continue failed");
                self.wait_for_signal();
                true
            }

            "breakpoint" => {
                //breakpoint at given memory address
                let addr = usize::from_str_radix(&args[1][2..], 16).expect("invalid address");//handles the first two chars
                self.set_a_breakpoint_at_a_address(addr); //sets a breakpoint at an address
                true
            }
            "register" => {match args[1]{
                    "dump" => {
                        self.dump_registers();
                        true
                    }

                    "read" => {
                        let reg = self.get_register_from_name(args[2]); 
                        let value = self.get_register_value(reg);

                        println!("{} = 0x{:016x}", args[2], value); //print value of the desired register
                        true
                    }

                    "write" => {
                        let reg = self.get_register_from_name(args[2]);
                        let value = u64::from_str_radix(&args[3][2..], 16).expect("Error");

                        self.set_register_value(reg, value); //changes register values according to what the user typed on the CLI
                        true
                    }

                    _ => {
                        println!("error");
                        true
                    }
                }
            }

            "memory" =>{
                match args[1]{

                    "write" => {
                        let address = usize::from_str_radix(&args[2][2..], 16).expect("error");
                        let value = i64::from_str_radix(&args[3][2..], 16).expect("Error");
                        self.write_memory(address, value);
                        true
                    }

                    "read" => {
                        let address = usize::from_str_radix(&args[2][2..], 16).expect("error");
                        println!("0x{:016x}", self.read_memory(address));
                        true
                    }

                    _ => {
                        println!("error");
                        true
                    }

                }
            }
            

            _ => {
                println!("Unknown command");
                true
            }
        }
    }

    pub fn get_register_value(&self, reg: Register) -> u64{
        let regs = ptrace::getregs(self.pid).expect("Failed to get register"); //get register info
        match reg{ //returns values of the designated register
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

    pub fn set_register_value(&self, reg: Register, value: u64){
        let mut regs = ptrace::getregs(self.pid).expect("Failed to get register"); //get register that will be used by the process
        match reg{ //this match sets the value in the variable regs
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

        ptrace::setregs(self.pid, regs).expect("failed to set registers"); //sets the value on regs on the actual register
    }   

    pub fn get_register_value_from_dwarf_register(&self, regnum: i64) -> u64{
        let reg = REGISTER_DESCRIPTION.iter().find(|rd| rd.dwarf_r == regnum).expect("unknown dwarf register");
        return self.get_register_value(reg.r); //returns the value of dwarf register
    }

    pub fn get_register_name(&self, reg: Register) -> &'static str{
        return REGISTER_DESCRIPTION.iter().find(|rd| rd.r == reg).expect("register unknown").name //returns register name after knowing who it is
    }

    pub fn get_register_from_name(&self, name: &str) -> Register {
        return REGISTER_DESCRIPTION.iter().find(|rd| rd.name == name).expect("register unknown").r //returns the register after knowing its name
    }

    pub fn dump_registers(&self){
        for r in REGISTER_DESCRIPTION.iter(){
            println!("{} 0x{:016x}", r.name, self.get_register_value(r.r)); //print all of the current registers values
        }
    }
}