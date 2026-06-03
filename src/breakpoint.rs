use nix::unistd::{Pid};
use nix::sys::ptrace;

pub struct Breakpoint{
    m_pid: Pid, //process where the break point is act
    m_addr: usize, //memory address of the proccess where INT3 will be inserted
    m_enabled: bool, //breakpoint is active or not
    m_saved_data: u8 //original byte overwritten by INT3
}

impl Breakpoint{
    pub fn new(pid: Pid, addr: usize) -> Self{
        Self { m_pid: pid, m_addr: addr, m_enabled: false, m_saved_data: 0 } //creates a new breakpoint
    }

    pub fn is_enabled(&self) -> bool{
        return self.m_enabled; //returns if the breakpoint is enabled or not
    }

    pub fn get_address(&self) -> usize{
        return self.m_addr; //returns the address of memory where the breakpoint is set
    }

    pub fn enable(&mut self){
        let data = ptrace::read(self.m_pid, self.m_addr as ptrace::AddressType).expect("ptrace read falied");//reads the state of current process
        self.m_saved_data = (data & 0xff) as u8; //saves the byte that will be modified
        let int3 = 0xcc; //INT3 creation
        let data_with_int3 = (data & !0xff) | int3; //replaces data with the INT3 on the lowest byte
        ptrace::write(self.m_pid, self.m_addr as ptrace::AddressType, data_with_int3 as i64).expect("ptrace failed"); //writes the modified instruction with INT3 back to memory
        self.m_enabled = true;
    }

    pub fn disable(&mut self){
        let data = ptrace::read(self.m_pid, self.m_addr as ptrace::AddressType).expect("ptrace read falied");//reads the current satet of the process
        let restored_data = (data & !0xff) | self.m_saved_data as i64;//retores the original lowest byte data
        ptrace::write(self.m_pid, self.m_addr as ptrace::AddressType, restored_data).expect("ptrace failed");//writes the original instruction back to memory
        self.m_enabled = false;
    }
}