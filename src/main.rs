use std::{env};
use nix::unistd::{fork, ForkResult};
use nix::sys::ptrace;
use nix::unistd::execv;
use std::ffi::CString;
use nix::sys::personality::{set,Persona,};

mod breakpoint;
mod registers;
mod debugger;


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
                let mut dbg = debugger::Debugger::new(prog.to_string(), child);
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