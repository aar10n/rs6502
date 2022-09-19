use std::error::Error;
use std::fs;

use core::CPU;
use system::{device::StdoutDevice, Bus, Memory};

fn run() -> Result<(), Box<dyn Error>> {
    let mut stdout = StdoutDevice::new();
    let mut mem = Memory::new();
    mem.register_device(&mut stdout);

    let mut rom = fs::File::open("example/hello.o")?;
    // let mut rom = fs::File::open("example/fib.o")?;
    mem.load_rom(0x1000, &mut rom)?;
    mem.write(CPU::RES_VECTOR, 0x00);
    mem.write(CPU::RES_VECTOR + 1, 0x10);

    // // set N for fibonacci subroutine
    // let n = 11;
    // mem.write(0x99, n);

    let mut cpu = CPU::new();
    cpu.reset(&mut mem);

    use std::time::Instant;
    let start = Instant::now();

    for _ in 0..1000 {
        cpu.step_instruction(&mut mem);
        if cpu.status.get_decimal_mode() {
            break;
        }
    }

    let end = Instant::now();
    let elapsed = end - start;

    println!("{:?}\n", cpu);
    // println!("fib({}) = {}", n, mem.read(0x104));
    println!("took {} us", elapsed.as_micros());
    return Ok(());
}

fn main() {
    match run() {
        Ok(_) => {}
        Err(err) => panic!("error: {}", err.to_string()),
    }
}
