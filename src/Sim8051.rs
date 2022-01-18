use std::str::FromStr;

// Memory emulation of 8051 -> Partial emulation + simulation
pub struct RegisterBank<'a> {
    pub ptr: &'a mut [u8],
}

impl<'a> RegisterBank<'a> {
    pub fn clear(&mut self) {
        self.ptr.iter_mut().for_each(|x| *x = 0);
    }
    pub fn show_registers(&self) {
        for i in 0..8 {
            println!("R{} -> {:#04x}", i, self.ptr[i]);
        }
    }
}

//  The first 128 bytes of memory are general purpose and the remaining is for internal purpose where various special purpose registers are mapped
//  This is the internal RAM memory
pub struct InternalMemory {
    pub memory: [u8; 256], // This is RAM and address from 20H to 2F H are bit addressable and used along with SETB to address from 00H to 7FH
}

impl Default for InternalMemory {
    fn default() -> InternalMemory {
        InternalMemory {
            memory: [0x00; 256],
        }
    }
}

impl InternalMemory {
    pub fn get_register_bank(&mut self, n: u8) -> RegisterBank {
        assert!(n < 4); // lol nice catch rustc
        let start: usize = n as usize * 8;
        RegisterBank {
            ptr: &mut self.memory[start..start + 8],
        }
    }

    pub fn operate_bit_addressable_memory(&mut self, pos: u8, operator: fn(&mut Self, u8, u8)) {
        operator(self, 0x20 + pos / 8, pos % 8);
    }

    pub fn operate_bit_addressable_registers(
        &mut self,
        pos: u8,
        bit: u8,
        operator: fn(&mut Self, u8, u8),
    ) {
        operator(self, pos, bit);
    }

    pub fn set_bit_addressable(&mut self, pos: u8, bit: u8) {
        self.memory[pos as usize] |= 1 << bit;
    }

    pub fn reset_bit_addressable(&mut self, pos: u8, bit: u8) {
        self.memory[pos as usize] &= !(1 << bit);
    }

    pub fn complement_bit_addressable(&mut self, pos: u8, bit: u8) {
        // let pos = (0x) // Lets continue it later by thinking of what we going to do with assembler parsing and assembling
        self.memory[pos as usize] ^= 1 << bit;
    }
}

// Do pattern matching
#[derive(Debug)]
pub enum IRegs {
    // Non bit adderssable
    SP,
    DPTR,
    // Bit addressables
    PSW,
    ACC,
    B,
    IP,
}

#[derive(Debug)]
pub enum Ports {
    P0,
    P1,
    P2,
    P3,
}

#[derive(Debug)]
pub enum SFR {
    Reg(IRegs),
    Port(Ports),
}

// What to do about Special Function Registers mapping? Since, its partial emulation that need to be considered too
pub struct Sim8051 {
    PC: u16,
    pub internal_memory: InternalMemory,
    // Special purpose registers
    pub code_memory: [u8; 64 * 1024],
    pub data_memory: [u8; 64 * 1024],
    pub accumulator: SFR,
    pub register_b: SFR,
    pub psw: SFR,
}

impl Default for Sim8051 {
    fn default() -> Sim8051 {
        Sim8051 {
            internal_memory: InternalMemory::default(),
            PC: 0x0000,
            code_memory: [0; 64 * 1024],
            data_memory: [0; 64 * 1024],
            accumulator: SFR::Reg(IRegs::ACC),
            register_b: SFR::Reg(IRegs::B),
            psw: SFR::Reg(IRegs::PSW),
        }
    }
}

impl Sim8051 {
    pub fn mov(&mut self, dst: u8, src: u8) {
        self.internal_memory.memory[dst as usize] = src;
    }

    pub fn show_scratchpad_registers(&mut self) {
        let bank = self.get_active_register_bank();
        bank.show_registers();
    }

    pub fn show_flags(&self) {
        let pos = sfr_addr(&SFR::Reg(IRegs::PSW)) as usize;
        let flags = vec!["C", "AC", "F0", "RS1", "RS0", "OV", "_", "P"];
        for i in flags {
            print!("{:<10}", i);
        }
        println!();
        for i in (0..8).rev() {
            print!("{:10}", (self.internal_memory.memory[pos] & (1 << i)) > 0);
        }
        println!();
    }

    pub fn show_sfr_registers(&self) {
        use SFR::*;
        use IRegs::*;
        use Ports::*;
        println!("\nShowing SFR contents :\n");

        let vec = vec!(Reg(PSW),Reg(ACC),Reg(B), Port(P0),Port(P1),Port(P2),Port(P3));
        let vecname = vec!("PSW ","A","B","P0","P1","P2","P3");
        let mapping = vec.iter().zip(vecname.iter());

        for (val,name) in mapping {
            println!("{:<10} -> {:#04x}",name,self.internal_memory.memory[sfr_addr(&val) as usize]);
        }
    }
}

pub fn sfr_addr(sfr: &SFR) -> u8 {
    match sfr {
        SFR::Port(x) => match x {
            Ports::P0 => 0x80,
            Ports::P1 => 0x90,
            Ports::P2 => 0xA0,
            Ports::P3 => 0xB0,
        },
        SFR::Reg(reg) => {
            match reg {
                IRegs::ACC => 0xE0,
                IRegs::B => 0xF0,
                IRegs::PSW => 0xD0,
                IRegs::IP => 0xB8,
                IRegs::DPTR => 0x82, // Returning only the lower order byte .. guess the rest yourself
                IRegs::SP => 0x81,
            }
        }
    }
}

// Only bit addressable register implement this traits
impl FromStr for SFR {
    type Err = ();

    fn from_str(input: &str) -> Result<SFR, Self::Err> {
        match input {
            "P0" => Ok(SFR::Port(Ports::P0)),
            "P1" => Ok(SFR::Port(Ports::P1)),
            "P2" => Ok(SFR::Port(Ports::P2)),
            "P3" => Ok(SFR::Port(Ports::P3)),
            "ACC" => Ok(SFR::Reg(IRegs::ACC)),
            "B" => Ok(SFR::Reg(IRegs::B)),
            "PSW" => Ok(SFR::Reg(IRegs::PSW)),
            _ => Err(()), // indicates not a bit addressable register
        }
    }
}

#[derive(Debug)]
pub enum ScratchpadRegisters {
    R0,
    R1,
    R2,
    R3,
    R4,
    R5,
    R6,
    R7,
}

impl ScratchpadRegisters {
    pub fn reg_count(&self) -> u8 {
        match self {
            Self::R0 => 0,
            Self::R1 => 1,
            Self::R2 => 2,
            Self::R3 => 3,
            Self::R4 => 4,
            Self::R5 => 5,
            Self::R6 => 6,
            _ => 7,
        }
    }
}

impl FromStr for ScratchpadRegisters {
    type Err = ();

    fn from_str(input: &str) -> Result<ScratchpadRegisters, Self::Err> {
        match input {
            "R0" => Ok(ScratchpadRegisters::R0),
            "R1" => Ok(ScratchpadRegisters::R1),
            "R2" => Ok(ScratchpadRegisters::R2),
            "R3" => Ok(ScratchpadRegisters::R3),
            "R4" => Ok(ScratchpadRegisters::R4),
            "R5" => Ok(ScratchpadRegisters::R5),
            "R6" => Ok(ScratchpadRegisters::R6),
            "R7" => Ok(ScratchpadRegisters::R7),
            _ => Err(()),
        }
    }
}

impl Sim8051 {
    pub fn get_active_register_bank(&mut self) -> RegisterBank {
        let pswloc = sfr_addr(&SFR::Reg(IRegs::PSW)) as usize;
        let count = (0x18 & self.internal_memory.memory[pswloc]) >> 3;
        let start: usize = count as usize * 8;
        RegisterBank {
            ptr: &mut self.internal_memory.memory[start..start + 8],
        }
    }

    // TODO :: Later
    pub fn set_parity_bit(&mut self, val : u8) {
        // Don't ask what below code does :D -_-
        let parity : u64 =
            (((val as u64 * 0x0101010101010101 as u64) & 0x8040201008040201 as u64) % 0x1FF as u64) & (1 as u64);
        let even_parity = parity == 0;
        // what's the bit count of PSW.7?
        let addr = sfr_addr(&self.psw);
        // retrieve its value, either set it or reset it
        if even_parity {
            self.internal_memory.memory[addr as usize] &= 0xFE;
        }
        else {
            self.internal_memory.memory[addr as usize] |= 0x01;
        }
    }

    pub fn set_carry_bit(&mut self, set : bool){
        let addr = sfr_addr(&self.psw);
        if set {
            self.internal_memory.memory[addr as usize] |= 0x80;
        }
        else {
            self.internal_memory.memory[addr as usize] &= 0x7F;
        }
    }

    pub fn set_auxiliary_carry_bit(&mut self, set : bool) {
        let addr = sfr_addr(&self.psw);
        if set{
            self.internal_memory.memory[addr as usize] |= 0x40;
        }
        else {
            self.internal_memory.memory[addr as usize] &= 0xBF;
        }
    }

    pub fn set_overflow_bit(&mut self,set : bool){
        let addr = sfr_addr(&self.psw);
        if set{
            self.internal_memory.memory[addr as usize] |= 0x04;
        }
        else {
            self.internal_memory.memory[addr as usize] &= 0xFB;
        }
    }
}
