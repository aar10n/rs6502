use core::ops::Range;
use std::sync::Arc;
use std::{cell::RefCell, iter::FromIterator, rc::Rc, sync::Mutex};

use intervaltree::{Element, IntervalTree};

use crate::memory::Memory;

type RcRefBox<T> = Rc<RefCell<Box<T>>>;

pub trait Device {
    fn read(&self, address: u16) -> u8;
    fn write(&mut self, address: u16, data: u8);
}

pub struct Bus<'a> {
    memory: Arc<Mutex<&'a mut Memory>>,
    devices: Vec<(Range<u16>, RcRefBox<dyn Device + 'a>)>,
    mapped: IntervalTree<u16, RcRefBox<dyn Device + 'a>>,
}

impl<'a> Bus<'a> {
    pub fn new(memory: &'a mut Memory) -> Self {
        let iter = std::iter::empty::<Element<u16, RcRefBox<dyn Device>>>();
        let a = Arc::new(Mutex::new(memory));
        Bus {
            memory: a,
            devices: vec![],
            mapped: IntervalTree::from_iter(iter),
        }
    }

    pub fn register_device(&'a mut self, device: impl Device + 'a, range: Range<u16>) {
        let iter = self.mapped.query(range.clone());
        if iter.peekable().peek().is_some() {
            panic!("requested range overlaps with an existing device");
        }

        let boxed: RcRefBox<dyn Device + 'a> = Rc::new(RefCell::new(Box::new(device)));
        self.devices.push((range, boxed));
        self.mapped = IntervalTree::from_iter(self.devices.iter_mut().map(|t| Element {
            range: t.0.clone(),
            value: Rc::clone(&t.1),
        }));
    }

    pub fn read(&self, address: u16) -> u8 {
        if let Some(device) = self.get_device_or_none(address) {
            return device.borrow().read(address);
        }
        return self.read_mem(address);
    }

    pub fn write(&mut self, address: u16, data: u8) {
        if let Some(device) = self.get_device_or_none(address) {
            (*device).borrow_mut().write(address, data);
        }
        self.write_mem(address, data);
    }

    //

    fn read_mem(&self, address: u16) -> u8 {
        if let Ok(memory) = self.memory.lock() {
            return memory.read8(address);
        }
        unreachable!("uh oh");
    }

    fn write_mem(&mut self, address: u16, data: u8) {
        if let Ok(mut memory) = self.memory.lock() {
            memory.write8(address, data);
            return;
        }
        unreachable!("uh oh");
    }

    fn get_device_or_none(&self, address: u16) -> Option<RcRefBox<dyn Device + 'a>> {
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
}
