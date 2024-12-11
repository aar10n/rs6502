mod cpu;
mod instructions;
mod microcode;
mod opcode;
mod registers;
mod utility;

pub use cpu::Cpu;

pub trait Bus {
    fn read<'a>(&'a self, address: u16) -> u8;
    fn write<'a>(&'a mut self, address: u16, data: u8);
}
