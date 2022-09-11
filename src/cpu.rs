use crate::memory::Memory;
use crate::microcode::{ucode_reset, Context, MicroOp};
use crate::opcode;
use crate::registers::{Registers, StatusFlags};
use crate::utility;

utility::bitset! {
    #[derive(Clone, Copy)]
    pub struct Pins(u8);

    0 : irq  => IRQ;
    1 : rdy  => RDY;
    2 : ml   => ML;
    3 : nmi  => NMI;
    4 : sync => SYNC;
    5 : res  => RES;
}

impl std::fmt::Debug for Pins {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = utility::multiline! {
            "Pins:"
            "{}={}\n{}={}\n{}={}"
            "{}={}\n{}={}\n{}={}"
        };

        #[rustfmt::skip]
        return write!(
            f,
            utility::multiline! {
                "Pins:"
                "{}={}\n{}={}\n{}={}"
                "{}={}\n{}={}\n{}={}"
            },
            utility::overline!('I''R''Q'), self.get_irq(),
            "RDY", self.get_rdy(),
            "ML", self.get_ml(),
            utility::overline!('N''M''I'), self.get_nmi(),
            "SYNC", self.get_sync(),
            utility::overline!('R''E''S'), self.get_res(),
        );
    }
}

pub struct CPU<'a> {
    pub registers: Registers,
    pub status: StatusFlags,
    pub pins: Pins,
    pub memory: &'a mut Memory,

    cycle: u64,
    index: usize,
    ctx: Context,
    pipeline: Option<&'static [MicroOp]>,
}

impl<'a> CPU<'a> {
    pub const NMI_VECTOR: u16 = 0xFFFA;
    pub const RES_VECTOR: u16 = 0xFFFC;
    pub const IRQ_VECTOR: u16 = 0xFFFE;

    pub fn new(memory: &'a mut Memory) -> Self {
        Self {
            registers: Registers::new(),
            status: StatusFlags::new(),
            pins: Pins::from(Pins::IRQ | Pins::NMI | Pins::SYNC),
            memory,

            cycle: 0,
            index: 0,
            ctx: Context::new(),
            pipeline: None,
        }
    }

    pub fn reset(&mut self) {
        self.cycle = 0;
        self.index = 0;
        self.ctx = Context::new();
        self.pipeline = None;

        let mut ctx = Context::new();
        let ops = ucode_reset();
        for op in ops {
            let cycle = op.execute(self, &mut ctx);
            self.cycle += cycle as u64;
        }
    }

    pub fn step_instruction(&mut self) {
        if self.pipeline.is_none() {
            self.cycle(); // fetch next instruction
        }

        while self.pipeline.is_some() {
            self.step_cycle();
        }
    }

    pub fn step_cycle(&mut self) {
        self.cycle();
    }

    //

    fn cycle(&mut self) {
        if self.pipeline.is_none() {
            // fetch & decode next instruction
            let pc = self.registers.pc.get();
            self.registers.pc.set(pc + 1); // increment pc

            let op = self.memory.read8(pc);
            let ucode = opcode::decode_instruction(op);

            self.ctx = Context::new();
            self.index = 0;
            self.pipeline = Some(ucode);
            self.cycle += 1;
            return;
        }

        // execute next micro-op in pipeline
        let pipeline = self.pipeline.unwrap();
        loop {
            let uop = pipeline[self.index];
            self.index += 1;

            let cycle: u8;
            unsafe {
                let ctx = &mut self.ctx as *mut _;
                cycle = uop.execute(self, &mut *ctx);
            }

            if self.index >= pipeline.len() {
                // end of pipeline
                self.pipeline = None;
            }

            // continue until we run a micro-op that actually takes a cycle
            if cycle != 0 {
                self.cycle += cycle as u64;
                break;
            } else if self.pipeline.is_none() {
                break;
            }
        }
    }
}

impl<'a> std::fmt::Debug for CPU<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "{:?}\n\n{:?}",
            self.registers,
            self.status, // self.pins
        );
    }
}
