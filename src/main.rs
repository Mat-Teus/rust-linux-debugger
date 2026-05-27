use std::{env, io};
use std::io::Write;
use std::hash::Hash;
use nix::unistd::{fork, ForkResult, Pid};
use nix::sys::ptrace;
use nix::sys::wait::waitpid;
use std::collections::HashMap;
use nix::unistd::execv;
use std::ffi::CString;

//stores debugger state and target process information
struct Debugger{
    pid: Pid,
    program_name: String,
}

impl Debugger {
    //creates a new debugger instance for the target process
    fn new(program_name: String, pid: Pid) -> Self {
        Self {
            pid,
            program_name,
        }
    }

    //Run the debugger
    fn run(&self) {
        //Waits until child process (debugee) stops
        match waitpid(self.pid, None){
            Ok(status) => {
                println!("");
            }

            Err(err) => {
                println!("");
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

    fn handle_command(&self, command: &str) -> bool{
        //execution of debugger commands
        match command.trim(){
            //Quits the debugger
            "quit" => false,
            //Continues the execution of the debugee
            "continue" => {
                ptrace::cont(self.pid, None).expect("Continue failed");
                waitpid(self.pid, None).expect("waitpid failed");
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
                let dbg = Debugger::new(prog.to_string(), child);
                dbg.run();
            }

            Ok(ForkResult::Child) => {
                //Child process
                //Execute debugee
                ptrace::traceme().expect("traceme failed");
                let prog_c = CString::new(prog.as_str()).expect("CString conversion failed");
                execv(&prog_c, &[prog_c.clone()]).expect("execv failed");
            }

            Err(err) => {
                println!("Fork failed");
            }
        }
    }

}