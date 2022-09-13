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
    let mut rom = fs::File::open("example/fib.o")?;
    mem.load_rom(0x1000, &mut rom)?;
    mem.write16(CPU::RES_VECTOR, 0x1000);

    // set N for fibonacci subroutine
    let n = 11;
    mem.write8(0x99, n);

    let mut cpu = CPU::new(&mut mem);
    cpu.reset();
    for _ in 0..1000 {
        cpu.step_instruction();
        if cpu.status.get_decimal_mode() {
            break;
        }
    }

    println!("{:?}\n", cpu);
    println!("fib({}) = {}", n, mem.read8(0x104));

    return Ok(());
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(err) => panic!("error: {}", err.to_string()),
    }
}
