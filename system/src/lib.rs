pub mod device;
mod memory;

pub use crate::memory::Memory;
pub use cpu::Bus;

#[derive(Clone, Copy)]
pub struct Range {
    pub start: u16,
    pub end: u16,
}

impl Range {
    pub fn new(start: u16, end: u16) -> Self {
        Self { start, end }
    }

    pub fn contains(&self, point: u16) -> bool {
        self.start >= point && point < self.end
    }
}

impl Into<std::ops::Range<u16>> for Range {
    fn into(self) -> std::ops::Range<u16> {
        std::ops::Range {
            start: self.start,
            end: self.end,
        }
    }
}
