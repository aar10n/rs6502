use std::error::Error;
use std::fs;
use std::io::Read;

pub struct Memory {
    size: usize,
    data: Vec<u8>,
}

impl Memory {
    pub fn new() -> Self {
        let size = usize::from(u16::MAX);
        Self {
            size,
            data: vec![0; size],
        }
    }

    pub fn load_rom(&mut self, at_address: u16, rom: &mut fs::File) -> Result<(), Box<dyn Error>> {
        let addr = at_address as usize;
        if addr > self.size {
            return Err(format!("cannot load rom at address {:#04x}", at_address).into());
        }

        let metadata = rom.metadata()?;
        if metadata.len() > ((self.size - addr) as u64) {
            return Err("rom size exceeds available memory".into());
        }

        let mut buffer = Vec::new();
        rom.read_to_end(&mut buffer)?;
        (&mut self.data[addr..(addr + buffer.len())]).copy_from_slice(&buffer);
        println!("loaded {} bytes at address ${:04x}", buffer.len(), addr);
        return Ok(());
    }

    pub fn read8(&self, address: u16) -> u8 {
        let index = usize::from(address);
        assert!(index <= self.size - 1);
        return self.data[index];
    }

    pub fn write8(&mut self, address: u16, value: u8) {
        let index = usize::from(address);
        assert!(index <= self.size - 1);
        self.data[index] = value;
    }

    pub fn read16(&self, address: u16) -> u16 {
        let index = usize::from(address);
        assert!(index < self.size - 1);

        let bytes = [self.data[index], self.data[index + 1]];
        return u16::from_le_bytes(bytes);
    }

    pub fn write16(&mut self, address: u16, value: u16) {
        let index = usize::from(address);
        assert!(index < self.size - 1);

        let bytes = value.to_le_bytes();
        self.data[index] = bytes[0]; // lo-byte
        self.data[index + 1] = bytes[1]; // hi-byte
    }
}
