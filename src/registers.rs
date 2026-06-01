pub const N_REGISTERS:usize = 27;

//Register description is a array of register with the information of all the 27 registers that can be used during the debug
pub const REGISTER_DESCRIPTION:[Register_Description; N_REGISTERS] = [
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
pub enum Register{//enum with name of the 27 registers that will be used
    Rax, Rbx, Rcx, Rdx,
    Rdi, Rsi, Rbp, Rsp,
    R8,  R9,  R10, R11,
    R12, R13, R14, R15,
    Rip, Rflags, Cs,
    Orig_rax, Fs_base,
    Gs_base,
    Fs, Gs, Ss, Ds, Es
}

//Description of the register that will be used
pub struct Register_Description{
    pub name: &'static str,
    pub dwarf_r: i64,
    pub r: Register
}