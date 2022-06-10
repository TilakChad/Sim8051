// Lets start our 8051 Simulator here
// First need to learn some 8051 first
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
    let mut asm = assembler::Assembler::default();
    // for i in asm.simulator.internal_memory.memory.iter_mut() {
    //     *i = 0xAB;
    // }

    asm.read_src(String::from("./test.asm"));

    asm.tokenizer.src.push_str("\n$");
    let z = asm.tokenizer.src;
    asm.tokenizer.src = String::from("^\n");
    asm.tokenizer.src.push_str(&z);
    println!("\nRead asm src file : \n {}", asm.tokenizer.src);

    // Now start parsing the grammar

    for i in &mut asm.simulator.internal_memory.memory {
        *i = 0x00;
    }

    asm.start();

    println!("\n\nAfter executing the source code : ");
    for (n, i) in asm
        .simulator
        .internal_memory
        .memory
        .iter()
        .skip(0)
        .take(16)
        .enumerate()
    {
        print!("{:>#10x} ", i);
        if (n + 1) % 8 == 0 {
            println!();
        }
    }
    println!();
    println!("------------------------- Showing 8051 Flags Status -----------------------------");
    asm.simulator.show_flags();
    asm.simulator.show_scratchpad_registers();
    asm.simulator.show_sfr_registers();
    asm.show_jmptable();
}
