use std::str::FromStr;

// Memory emulation of 8051 -> Partial emulation + simulation
pub struct RegisterBank<'a>
{
    pub ptr : &'a mut [u8]
}

impl<'a> RegisterBank<'a>
{
    pub fn clear(&mut self)
    {
        self.ptr.iter_mut().for_each(|x| *x = 0);
    }
}

//  The first 128 bytes of memory are general purpose and the remaining is for internal purpose where various special purpose registers are mapped
//  This is the internal RAM memory
pub struct InternalMemory {
    pub memory : [u8;256] // This is RAM and address from 20H to 2F H are bit addressable and used along with SETB to address from 00H to 7FH

}

impl Default for InternalMemory {
    fn default() -> InternalMemory
    {
        InternalMemory {
            memory : [0x00;256]
        }
    }
}

impl InternalMemory
{
    pub fn get_register_bank(&mut self, n : u8) -> RegisterBank
    {
        assert!(n<4); // lol nice catch Rustc
        let start : usize = n as usize * 8;
        RegisterBank{
            ptr : & mut self.memory[start..start+8]
        }
    }

    // I guess this should be filtered one step before
    fn set_bit_addressable(&mut self, pos : usize, bit : u8)
    {
        self.memory[pos] = self.memory[pos] | (1 << bit);
    }

    fn reset_bit_addressable(&mut self, pos : usize, bit : u8)
    {
        self.memory[pos] = self.memory[pos] & !(1 << bit);
    }

    fn complement_bit_addressable(&mut self, pos : usize, bit : u8)
    {
        // let pos = (0x) // Lets continue it later by thinking of what we going to do with assembler parsing and assembling
        let byte         = self.memory[pos];
        self.memory[pos] = byte ^ (1 << bit);
    }
}

// Do pattern matching
#[derive(Debug)]
pub enum IRegs
{
    // Non bit adderssable
    SP   ,
    DPTR ,
    // Bit addressables
    PSW  ,
    ACC  ,
    B    ,
    IP
}


#[derive(Debug)]
pub enum Ports
{
    P0,
    P1,
    P2,
    P3
}

#[derive(Debug)]
pub enum SFR
{
    Reg(IRegs),
    Port(Ports)
}

// What to do about Special Function Registers mapping? Since, its partial emulation that need to be considered too
pub struct Sim8051
{
    PC : u16,
    internal_memory : InternalMemory,
    // Special purpose registers
    code_memory     : [u8;64*1024],
    data_memory     : [u8;64*1024]
}

impl Default for Sim8051
{
    fn default() -> Sim8051 {
        Sim8051 {
            internal_memory : InternalMemory::default(),
            PC              : 0x0000,
            code_memory     : [0;64*1024],
            data_memory     : [0;64*1024]
        }
    }
}

fn sfr_addr(sfr : SFR) -> u8
{
    match sfr {
        SFR::Port(x) => {
            match x {
                Ports::P0 => 0x80,
                Ports::P1 => 0x90,
                Ports::P2 => 0xA0,
                Ports::P3 => 0xB0
            }
        }
        SFR::Reg(reg) => {
            match reg {
                IRegs::ACC  => 0xE0,
                IRegs::B    => 0xF0,
                IRegs::PSW  => 0xD0,
                IRegs::IP   => 0xB8,
                IRegs::DPTR => 0x82, // Returning only the lower order byte .. guess the rest yourself
                IRegs::SP   => 0x81
            }
        }
    }
}

impl FromStr for SFR {
    type Err = ();

    fn from_str(input : &str) -> Result<SFR,Self::Err> {
        match input {
            "P0"     => Ok(SFR::Port(Ports::P0)),
            "P1"     => Ok(SFR::Port(Ports::P1)),
            "P2"     => Ok(SFR::Port(Ports::P2)),
            "P3"     => Ok(SFR::Port(Ports::P3)),
            "ACC"    => Ok(SFR::Reg(IRegs::ACC)),
            "B"      => Ok(SFR::Reg(IRegs::B)),
            "PSW"    => Ok(SFR::Reg(IRegs::PSW)),
            _        => Err(())
        }
    }
}
