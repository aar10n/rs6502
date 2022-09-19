mod memory;

pub use crate::memory::Memory;
pub use core::Bus;

pub trait Device {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, data: u8);
}
