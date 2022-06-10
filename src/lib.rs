use std::os::raw::c_char;

pub mod Sim8051;
pub mod assembler;
pub mod lexer;
// Disable the name mangling

// Define a struct to out all the required information
// They are :
// Flag content
// 128 byte memory address, let's pass whole thing
// Scratchpad registers
// Ports
// Errors .. which we will be covering later
// Lets get done with it, there's no point on taking this assembler ahead further .. Better work on a new compiler including both complete front-end and back-end

#[repr(C)]
pub struct AsmData {
    compiled: bool,
    psw: u8,
    sfr_arr: *mut u8,
    sfr_len: u64,
    reg_arr: *mut u8,
    reg_len: u64,
    memory: *mut u8,
    memory_len: u64, // Every other thing can be inferred from here in C++ side
}

impl Default for AsmData {
    fn default() -> AsmData {
        AsmData {
            compiled: false,
            psw: 0,
            sfr_arr: std::ptr::null_mut(),
            sfr_len: 0,
            reg_arr: std::ptr::null_mut(),
            reg_len: 0,
            memory: std::ptr::null_mut(),
            memory_len: 0,
        }
    }
}

#[no_mangle]
// its return type would be a struct that pass every information of the assembler to c++
pub extern "C" fn RustAssemble(ptr: *const c_char, len: u64) -> AsmData {
    let compiled = false;
    if ptr.is_null() {
        let mut failed_data = AsmData::default();
        failed_data.compiled = false;
        return failed_data;
    }

    println!("Rust got the length : {}.", len);
    let src = unsafe {
        std::str::from_utf8_unchecked(std::slice::from_raw_parts(ptr as *const u8, len as usize))
    };

    let mut asm = assembler::Assembler::default();

    asm.read_src_from_string(String::from(src));

    asm.tokenizer.src.push_str("\n$");
    let z = asm.tokenizer.src;
    asm.tokenizer.src = String::from("^\n");
    asm.tokenizer.src.push_str(&z);

    println!("\nRead asm src file : \n {}", asm.tokenizer.src);

    for i in &mut asm.simulator.internal_memory.memory {
        *i = 0x00;
    }

    asm.start();
    println!("------------------------- Showing 8051 Flags Status -----------------------------");
    asm.simulator.show_flags();
    asm.simulator.show_scratchpad_registers();
    asm.simulator.show_sfr_registers();
    asm.show_jmptable();

    //    return AsmData::default();
    let mut ffi_data = AsmData::default();
    let mem = Box::new(asm.simulator.internal_memory.memory);

    ffi_data.memory_len = asm.simulator.internal_memory.memory.len() as u64;
    ffi_data.memory = std::boxed::Box::into_raw(mem) as *mut _;

    // Repeat
    // Take the current active register bank
    let active_bank = asm.simulator.get_active_register_bank();
    let mem = active_bank.ptr.to_vec().into_boxed_slice();
    ffi_data.reg_len = active_bank.ptr.len() as u64;
    ffi_data.reg_arr = std::boxed::Box::into_raw(mem) as *mut _;

    use Sim8051::IRegs::*;
    use Sim8051::Ports::*;
    use Sim8051::SFR::*;

    let vec = vec![
        Reg(PSW),
        Reg(ACC),
        Reg(B),
        Port(P0),
        Port(P1),
        Port(P2),
        Port(P3),
    ];
    let sfr_vec: Vec<u8> = vec
        .iter()
        .map(|reg| asm.simulator.internal_memory.memory[Sim8051::sfr_addr(&reg) as usize])
        .collect();
    ffi_data.sfr_len = sfr_vec.len() as u64;
    ffi_data.sfr_arr = std::boxed::Box::into_raw(sfr_vec.into_boxed_slice()) as *mut _;
    ffi_data.compiled = true;

    ffi_data
}
