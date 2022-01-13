// Lets start our 8051 Simulator here
// First need to learn some 8051 first
//
// 8051 has internal 256 bytes memory containing register banks, BIT space, direct DATA space and indirect IDATA space
// #[derive(Clone,Copy)]
// struct RegisterBank {
//     register : [u8;8],
//     location : u8
// }
// impl RegisterBank
// {
//     fn clear(&mut self) {
//         self.register.iter_mut().for_each(|x| *x = 0);
//     }
// }

// impl Default for RegisterBank {
//     fn default() -> RegisterBank
//     {
//         RegisterBank{register : [0;8],location : 0x00}
//     }
// }

// // For internal memory we have 4 register banks starting at 0x00 t0 0x18
// // It also stores flag contents inside it

// // We going for simulator + emulator :=> memory organization will be emulated
// //
// struct InternalMemory {
//     registerbanks : [RegisterBank;4],
//     flags         : u8,
//     psw           : u8
//         // Bit 7 -> CY
//         // Bit 6 -> AC
//         // Bit 5 -> F0
//         // Bit 4 -> RS1
//         // Bit 3 -> RS0
//         // Bit 2 -> OV
//         // Bit 1 -> UD
//         // Bit 0 -> P
// }

// impl Default for InternalMemory {
//     fn default() -> InternalMemory
//     {
//         let mut memory = InternalMemory {registerbanks : [RegisterBank::default();4],
//                                          flags : 0,
//                                          psw   : 0};

//         for (loc,bank) in memory.registerbanks.iter_mut().enumerate()
//         {
//             bank.location = 0x8 * loc as u8;
//         }
//         memory
//     }
// }

// fn main() {
//     // Internal memory of 8051 starts here
//     let memory = InternalMemory::default();
//     println!();
//     for (i,register) in memory.registerbanks.iter().enumerate()
//     {
//         println!("Bank {} -> location : {}",i,register.location);
//     }
//     let mut s1 = String::from("Hello");
//     s1.push_str(" from The Rust Programming Language");
//     println!("{}",s1);
// }

// Emulating memory organization of 8051 micro-controller
//
//
// The first 128 bytes is general purpose register and remaining 128 bytes is special purpose registers and cannot be addressed directly or indirectly
// Among first 128 bytes,
// 0x00 to 0x1F -> 4 register banks
// 0x20 to 0x2F -> Bit addressable memory
// 0x30 to 0x7F -> General purpose registers
//

// // Memory emulation of 8051 -> Partial emulation + simulation

// struct RegisterBank<'a>
// {
//     ptr : &'a mut [u8]
// }

// impl<'a> RegisterBank<'a>
// {
//     fn clear(&mut self)
//     {
//         self.ptr.iter_mut().for_each(|x| *x = 0);
//     }
// }

// //  The first 128 bytes of memory are general purpose and the remaining is for internal purpose where various special purpose registers are mapped
// //  This is the internal RAM memory
// struct InternalMemory {
//     memory : [u8;256] // This is RAM and address from 20H to 2F H are bit addressable and used along with SETB to address from 00H to 7FH

// }

// impl Default for InternalMemory {
//     fn default() -> InternalMemory
//     {
//         InternalMemory {
//             memory : [0x00;256]
//         }
//     }
// }

// impl InternalMemory
// {
//     fn get_register_bank(&mut self, n : u8) -> RegisterBank
//     {
//         assert!(n<4); // lol nice catch Rustc
//         let start : usize = n as usize * 8;
//         RegisterBank{
//             ptr : & mut self.memory[start..start+8]
//         }
//     }

//     // I guess this should be filtered one step before
//     fn set_bit_addressable(&mut self, pos : usize, bit : u8)
//     {
//         self.memory[pos] = self.memory[pos] | (1 << bit);
//     }

//     fn reset_bit_addressable(&mut self, pos : usize, bit : u8)
//     {
//         self.memory[pos] = self.memory[pos] & !(1 << bit);
//     }

//     fn complement_bit_addressable(&mut self, pos : usize, bit : u8)
//     {
//         // let pos = (0x) // Lets continue it later by thinking of what we going to do with assembler parsing and assembling
//         let byte         = self.memory[pos];
//         self.memory[pos] = byte ^ (1 << bit);
//     }
// }

// // Do pattern matching
// enum IRegs
// {
//     // Non bit adderssable
//     SP   ,
//     DPTR ,
//     // Bit addressables
//     PSW  ,
//     ACC  ,
//     B    ,
//     IP
// }

// enum Ports
// {
//     P0,
//     P1,
//     P2,
//     P3
// }

// enum SFR
// {
//     Reg(IRegs),
//     Port(Ports)
// }

// // What to do about Special Function Registers mapping? Since, its partial emulation that need to be considered too
// struct Sim8051
// {
//     PC : u16,
//     internal_memory : InternalMemory,
//     // Special purpose registers
//     code_memory     : [u8;64*1024],
//     data_memory     : [u8;64*1024]
// }

// impl Default for Sim8051
// {
//     fn default() -> Sim8051 {
//         Sim8051 {
//             internal_memory : InternalMemory::default(),
//             PC              : 0x0000,
//             code_memory     : [0;64*1024],
//             data_memory     : [0;64*1024]
//         }
//     }
// }
pub mod Sim8051;
pub mod assembler;
pub mod lexer;

fn main() {
    println!("Hello 8051 EmuSimulator");

    let mut memory = Sim8051::InternalMemory::default();
    let z = memory.get_register_bank(1);
    for i in z.ptr.iter_mut() {
        *i = 0xFF;
    }
    for (n, i) in memory.memory.iter().enumerate() {
        print!("{:4} ", i);
        if (n + 1) % 8 == 0 {
            println!();
        }
    }
    lexer::nothing();
    lexer::string_handling();
    let mut asm = assembler::Assembler::default();
    asm.read_src(String::from("./test.asm"));

    asm.tokenizer.src.push_str("\n$");
    let z = asm.tokenizer.src;
    asm.tokenizer.src = String::from("^\n");
    asm.tokenizer.src.push_str(&z);
    println!("\nRead asm src file : \n {}",asm.tokenizer.src);

    // Now start parsing the grammar
    asm.start();
}
