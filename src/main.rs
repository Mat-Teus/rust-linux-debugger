use std::{env, io};
use std::io::Write;
use std::hash::Hash;
use nix::libc::ptrace;
use nix::unistd::{fork, ForkResult, Pid};
use nix::sys::ptrace;
use nix::sys::wait::waitpid;
use std::collections::HashMap;
use nix::unistd::execv;
use std::ffi::CString;
use nix::sys::personality::{set,Persona,};

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