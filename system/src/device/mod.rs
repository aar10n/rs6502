mod stdout;

pub use crate::Range;
pub use stdout::StdoutDevice;

pub trait Device {
    fn get_range(&self) -> Range;
    fn set_range(&mut self, range: Range) -> bool;

    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, data: u8);
}
