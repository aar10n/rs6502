use crate::device::Device;

use crate::Range;

pub struct StdoutDevice {
    range: Range,
}

impl StdoutDevice {
    const MMIO_RANGE: Range = Range {
        start: 0xA000,
        end: 0xA001,
    };

    pub fn new() -> Self {
        Self {
            range: Self::MMIO_RANGE,
        }
    }
}

impl Device for StdoutDevice {
    fn get_range(&self) -> Range {
        return self.range;
    }

    fn set_range(&mut self, range: Range) -> bool {
        self.range = range;
        return true;
    }

    fn read(&self, _: u16) -> u8 {
        // reads not supported
        return 0;
    }

    fn write(&mut self, address: u16, data: u8) {
        assert!(address == self.range.start);
        print!("{}", data as char);
    }
}
