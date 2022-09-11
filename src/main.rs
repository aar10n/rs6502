mod cpu;
mod instructions;
mod memory;
mod microcode;
mod opcode;
mod registers;
mod utility;

use std::error::Error;
use std::fs;

use cpu::CPU;
use memory::Memory;

fn run() -> Result<(), Box<dyn Error>> {
    let mut mem = Memory::new();
    let mut rom = fs::File::open("example/hello.o")?;
    mem.load_rom(0x1000, &mut rom)?;
    mem.write16(CPU::RES_VECTOR, 0x1000);

    let mut cpu = CPU::new(&mut mem);
    cpu.reset();
    cpu.step_instruction();
    cpu.step_instruction();
    cpu.step_instruction();
    cpu.step_instruction();
    cpu.step_instruction();
    cpu.step_instruction();

    println!("{:?}", cpu);

    println!("0x200 -> {:02x}", mem.read8(0x200));
    println!("0x201 -> {:02x}", mem.read8(0x201));
    println!("0x202 -> {:02x}", mem.read8(0x202));

    return Ok(());
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(err) => panic!("error: {}", err.to_string()),
    }
}
