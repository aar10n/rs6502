use std::error::Error;
use std::fs;
use std::io::Read;
use std::ops::Range;
use std::{cell::RefCell, iter::FromIterator, rc::Rc};

use core::Bus;
use intervaltree::{Element, IntervalTree};

use crate::device::Device;

type RcRefBox<T> = Rc<RefCell<Box<T>>>;

pub struct Memory<'a> {
    size: usize,
    data: Vec<u8>,
    devices: Vec<(Range<u16>, RcRefBox<&'a mut (dyn Device + 'a)>)>,
    mapped: IntervalTree<u16, RcRefBox<&'a mut (dyn Device + 'a)>>,
}

impl<'a> Memory<'a> {
    pub fn new() -> Self {
        let iter = std::iter::empty::<Element<u16, RcRefBox<&mut dyn Device>>>();
        let size = usize::from(u16::MAX);
        Self {
            size,
            data: vec![0; size],
            devices: vec![],
            mapped: IntervalTree::from_iter(iter),
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

    pub fn register_device(&mut self, device: &'a mut (impl Device + 'a)) {
        // device.get_range()

        let iter = self.mapped.query(device.get_range().into());
        if iter.peekable().peek().is_some() {
            panic!("requested range overlaps with an existing device");
        }

        let range = device.get_range();
        self.devices
            .push((range.into(), Rc::new(RefCell::new(Box::new(device)))));
        self.mapped = IntervalTree::from_iter(self.devices.iter_mut().map(|t| Element {
            range: t.0.clone(),
            value: Rc::clone(&t.1),
        }));
    }

    //

    fn get_device_or_none(&self, address: u16) -> Option<RcRefBox<&'a mut (dyn Device + 'a)>> {
        let range = Range {
            start: address,
            end: address + 1,
        };

        let devices = self
            .mapped
            .query(range)
            .map(|v| Rc::clone(&v.value))
            .collect::<Vec<_>>();
        assert!(devices.len() <= 1);
        return devices.first().map(|v| Rc::clone(v));
    }

    fn read_mem(&self, address: u16) -> u8 {
        let index = usize::from(address);
        assert!(index <= self.size - 1);
        return self.data[index];
    }

    fn write_mem(&mut self, address: u16, data: u8) {
        let index = usize::from(address);
        assert!(index <= self.size - 1);
        self.data[index] = data;
    }
}

impl<'a> Bus for Memory<'a> {
    fn read(&self, address: u16) -> u8 {
        if let Some(device) = self.get_device_or_none(address) {
            return device.borrow().read(address);
        }
        return self.read_mem(address);
    }

    fn write(&mut self, address: u16, data: u8) {
        if let Some(device) = self.get_device_or_none(address) {
            (*device).borrow_mut().write(address, data);
        }
        self.write_mem(address, data);
    }
}
