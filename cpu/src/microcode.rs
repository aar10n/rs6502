use crate::cpu::Cpu;
use crate::registers::Register;
use crate::Bus;

#[derive(Clone, Copy)]
pub struct Context {
    pub temp: Register<u8>,
    pub stack: [u8; 4],
    pub ptr: u8,
}

impl Context {
    const SIZE: u8 = 4;

    pub fn new() -> Self {
        Self {
            temp: Register::new(0),
            stack: [0; Self::SIZE as usize],
            ptr: Self::SIZE,
        }
    }

    pub fn size(self) -> usize {
        return (Self::SIZE - self.ptr) as usize;
    }

    pub fn peek(self, rel: u8) -> u8 {
        let ptr = self.ptr + rel;
        assert!(ptr < Self::SIZE);
        return self.stack[ptr as usize];
    }

    pub fn push(&mut self, byte: u8) {
        assert!(self.ptr > 0);
        self.ptr -= 1;
        self.stack[self.ptr as usize] = byte;
    }

    pub fn pop(&mut self) -> u8 {
        assert!(self.ptr < Self::SIZE);
        let byte = self.stack[self.ptr as usize];
        self.ptr += 1;
        return byte;
    }
}

#[derive(Clone, Copy)]
pub enum MicroOp {
    Unimplemented,

    /// Empty cycle (1 cycle)
    EmptyCycle,
    /// Empty ignored cycle (0 cycles)
    EmptyNoCycle,
    /// Loads the byte at the address in the PC register then increments it by one (1 cycle)
    LoadIncrPC,
    /// Pops a value off the context and stores it at the address pointed to by SP. Then decrement SP by one (1 cycle)
    StoreDecrSP,
    /// Increments the SP by one, then loads the value at the new address and pushes it onto the context stack (1 cycle)
    IncrLoadSP,

    /// Pushes the contents of the accumulator onto the context stack (0 cycles)
    PushAcc,
    /// Pushes a zero-byte onto the context stack (0 cycles)
    PushZero,
    /// Pushes the low order byte of the PC register onto the context stack (0 cycles)
    PushPCL,
    /// Pushes the high order byte of the PC register onto the context stack (0 cycles)
    PushPCH,
    /// Pops a hi and lo byte off the context stack and loads it into PCH and PCL respectively (0 cycles)
    PopJump,

    /// Pops a hi and lo byte off the context stack, loads it, then pushes the value onto the context (1 cycle)
    PopLoadAddress,
    /// Peeks the top two bytes (hi then lo) off the context stack, loads it and pushes the value onto the stack (1 cycle)
    PeekLoadAddress,
    /// Pops a value, followed by a hi and lo byte off the context stack and stores it at the address (1 cycle)
    PopStoreAddress,

    /// Pops a byte off the context stack and moves it into the temp register (0 cycles)
    PopTemp,
    /// Pushes the contents of the temp register onto the context stack (0 cycles)
    PushTemp,
    /// Increments the temp register by one (0 cycles)
    IncrTemp,
    /// Adds the contents of the cpu X register to the temp register (0 cycles)
    AddTempX,

    Execute(fn(&mut Cpu, &mut Context)),
    Evaluate(fn(&mut Cpu, &mut Context) -> MicroOp),
}

impl MicroOp {
    pub fn execute(self, cpu: &mut Cpu, ctx: &mut Context, bus: &mut dyn Bus) -> u8 {
        match self {
            MicroOp::Unimplemented => {
                panic!("instruction unimplemented");
            }
            MicroOp::EmptyCycle => {
                // single-cycle ignored micro op
                return 1;
            }
            MicroOp::EmptyNoCycle => {
                // zero-cycle ignored micro op
                return 0;
            }
            MicroOp::LoadIncrPC => {
                let pc = cpu.registers.pc.get();
                let value = bus.read(pc);
                ctx.push(value);
                cpu.registers.pc.set(pc + 1);
                return 1;
            }
            MicroOp::StoreDecrSP => {
                let sp = cpu.registers.sp.get();
                let address = u16::from_le_bytes([00, sp]);
                let value = ctx.pop();
                bus.write(address, value);
                cpu.registers.sp.set(sp - 1);
                return 1;
            }
            MicroOp::IncrLoadSP => {
                let sp = cpu.registers.sp.get() + 1;
                let address = u16::from_le_bytes([00, sp]);
                cpu.registers.sp.set(sp);
                let value = bus.read(address);
                ctx.push(value);
                return 1;
            }

            MicroOp::PushAcc => {
                let value = cpu.registers.acc.get();
                ctx.push(value);
                return 0;
            }
            MicroOp::PushZero => {
                ctx.push(0);
                return 0;
            }
            MicroOp::PushPCL => {
                let value = cpu.registers.pc.get_lo_byte();
                ctx.push(value);
                return 0;
            }
            MicroOp::PushPCH => {
                let value = cpu.registers.pc.get_hi_byte();
                ctx.push(value);
                return 0;
            }
            MicroOp::PopJump => {
                let hi = ctx.pop();
                let lo = ctx.pop();

                let pc = u16::from_le_bytes([lo, hi]);
                cpu.registers.pc.set(pc);
                return 0;
            }

            MicroOp::PopLoadAddress => {
                let hi = ctx.pop();
                let lo = ctx.pop();

                let address = u16::from_le_bytes([lo, hi]);
                let value = bus.read(address);
                ctx.push(value);
                return 1;
            }
            MicroOp::PeekLoadAddress => {
                let hi = ctx.peek(0);
                let lo = ctx.peek(1);

                let address = u16::from_le_bytes([lo, hi]);
                let value = bus.read(address);
                ctx.push(value);
                return 1;
            }
            MicroOp::PopStoreAddress => {
                let value = ctx.pop();
                let hi = ctx.pop();
                let lo = ctx.pop();

                let address = u16::from_le_bytes([lo, hi]);
                bus.write(address, value);
                return 1;
            }

            //
            MicroOp::PopTemp => {
                let value = ctx.pop();
                ctx.temp.set(value);
                return 0;
            }
            MicroOp::PushTemp => {
                let value = ctx.temp.get();
                ctx.push(value);
                return 0;
            }
            MicroOp::IncrTemp => {
                let value = ctx.temp.get();
                ctx.temp.set(value + 1);
                return 0;
            }
            MicroOp::AddTempX => {
                let (value, _) = ctx.temp.safe_add(cpu.registers.x.get());
                ctx.temp.set(value);
                return 0;
            }

            MicroOp::Execute(function) => {
                function(cpu, ctx);
                return 0;
            }
            MicroOp::Evaluate(function) => {
                let op = function(cpu, ctx);
                return op.execute(cpu, ctx, bus);
            }
        }
    }
}

//
// Core CPU Routines
//

pub fn ucode_reset() -> &'static [MicroOp] {
    return &[
        MicroOp::Execute(|cpu, _| {
            cpu.status.set(0);
            cpu.status = cpu.status.with_irq_disable(true).with_brk_command(true)
        }),
        MicroOp::Execute(|_, ctx| {
            let [lo, hi] = Cpu::RES_VECTOR.to_le_bytes();
            ctx.push(lo);
            ctx.push(hi);
        }),
        MicroOp::PopLoadAddress, // load pc low byte
        MicroOp::Execute(|_, ctx| {
            let [lo, hi] = (Cpu::RES_VECTOR + 1).to_le_bytes();
            ctx.push(lo);
            ctx.push(hi);
        }),
        MicroOp::PopLoadAddress, // load pc high byte
        MicroOp::Execute(|cpu, ctx| {
            let hi = ctx.pop();
            let lo = ctx.pop();

            let pc = u16::from_le_bytes([lo, hi]);
            cpu.registers.pc.set(pc);
        }),
    ];
}

//
// Single Byte Instructions
//

macro_rules! single_byte_implied {
    ($func: ident) => {
        &[
            MicroOp::EmptyCycle, // pause
            MicroOp::Execute($func),
        ]
    };
}
pub(crate) use single_byte_implied;

macro_rules! single_byte_accumulator {
    ($func: ident) => {
        &[
            MicroOp::EmptyCycle, // pause
            MicroOp::PushAcc,    // push acc as data
            MicroOp::Execute($func),
        ]
    };
}
pub(crate) use single_byte_accumulator;

//
// Internal Execution on Memory Data
//

macro_rules! load_immediate {
    ($func: ident) => {
        &[
            MicroOp::LoadIncrPC, // fetch data
            MicroOp::Execute($func),
        ]
    };
}
pub(crate) use load_immediate;

macro_rules! load_zero_page {
    ($func: ident) => {
        &[
            MicroOp::LoadIncrPC,     // fetch low order effective address byte
            MicroOp::PushZero,       // push implied 0 high order address byte
            MicroOp::PopLoadAddress, // fetch data
            MicroOp::Execute($func),
        ]
    };
}
pub(crate) use load_zero_page;

macro_rules! load_absolute {
    ($func: ident) => {
        &[
            MicroOp::LoadIncrPC,     // fetch low order address byte
            MicroOp::LoadIncrPC,     // fetch high order address byte
            MicroOp::PopLoadAddress, // fetch data
            MicroOp::Execute($func),
        ]
    };
}
pub(crate) use load_absolute;

macro_rules! load_indirect_x {
    ($func: ident) => {
        &[
            MicroOp::LoadIncrPC, // fetch page zero base address
            MicroOp::PopTemp,    // temp = bal
            MicroOp::EmptyCycle, // pause for one cycle
            //
            MicroOp::AddTempX,       // temp = bal + x
            MicroOp::PushTemp,       // push temp onto stack
            MicroOp::PushZero,       // push hi zero byte
            MicroOp::PopLoadAddress, // fetch low order address byte
            //
            MicroOp::IncrTemp,       // temp = bal + x + 1
            MicroOp::PushTemp,       // push temp onto stack
            MicroOp::PushZero,       // push hi zero byte
            MicroOp::PopLoadAddress, // fetch high order address byte
            //
            MicroOp::PopLoadAddress, // fetch data
            MicroOp::Execute($func),
        ]
    };
}
pub(crate) use load_indirect_x;

macro_rules! load_indirect_y {
    ($func: ident) => {
        &[
            MicroOp::LoadIncrPC, // fetch page zero indirect address
            MicroOp::PopTemp,    // temp = ial
            //
            MicroOp::PushTemp,       // push temp onto stack
            MicroOp::PushZero,       // push hi zero byte
            MicroOp::PopLoadAddress, // fetch low order address byte of base address
            //
            MicroOp::IncrTemp,       // temp = ial + 1
            MicroOp::PushTemp,       // push temp onto stack
            MicroOp::PushZero,       // push hi zero byte
            MicroOp::PopLoadAddress, // fetch high order address byte of base address
            //
            MicroOp::Evaluate(|cpu, ctx| {
                let bal = ctx.pop();
                let bah = ctx.pop();

                let (lo, carry) = cpu.registers.y.safe_add(bal);
                let hi = bah + (carry as u8);

                if hi != bah {
                    // crosses page boundary, we must spend one more cycle
                    // to fetch the data from the next page
                    ctx.push(lo);
                    ctx.push(hi);
                    return MicroOp::EmptyCycle;
                }

                // doesn't cross page boundary so we can shorten this
                // instruction sequence by one cycle
                return MicroOp::PopLoadAddress; // fetch data
            }),
            MicroOp::Evaluate(|_, ctx| {
                if ctx.size() == 2 {
                    // the previous addition caused a page boundary to be
                    // crossed so we load the data from the next page now
                    return MicroOp::PopLoadAddress;
                }

                // the data has already been loaded so we can skip this cycle
                return MicroOp::EmptyNoCycle;
            }),
            //
            MicroOp::Execute($func),
        ]
    };
}
pub(crate) use load_indirect_y;

macro_rules! load_absolute_indexed {
    ($func: ident, $register: ident) => {
        &[
            MicroOp::LoadIncrPC, // fetch low order byte of base address
            MicroOp::LoadIncrPC, // fetch high order byte of base address
            MicroOp::Evaluate(|cpu, ctx| {
                let bah = ctx.pop();
                let bal = ctx.pop();

                let (lo, carry) = cpu.registers.$register.safe_add(bal);
                let hi = bah + (carry as u8);

                if hi != bah {
                    // crosses page boundary, we must spend one more cycle
                    // to fetch the data from the next page
                    ctx.push(lo);
                    ctx.push(hi);
                    return MicroOp::EmptyCycle;
                }

                // doesn't cross page boundary so we can shorten this
                // instruction sequence by one cycle
                return MicroOp::PopLoadAddress; // fetch data
            }),
            MicroOp::Evaluate(|_, ctx| {
                if ctx.size() == 2 {
                    // the previous addition caused a page boundary to be
                    // crossed so we load the data from the next page now
                    return MicroOp::PopLoadAddress;
                }

                // the data has already been loaded so we can skip this cycle
                return MicroOp::EmptyNoCycle;
            }),
            //
            MicroOp::Execute($func),
        ]
    };
}
pub(crate) use load_absolute_indexed;

macro_rules! load_zero_page_indexed {
    ($func: ident, $register: ident) => {
        &[
            MicroOp::LoadIncrPC, // fetch page zero base address
            MicroOp::EmptyCycle, // pause for one cycle
            MicroOp::Evaluate(|cpu, ctx| {
                let bal = ctx.pop();

                let (lo, _) = cpu.registers.$register.safe_add(bal);
                let hi = 0;

                ctx.push(lo);
                ctx.push(hi);
                return MicroOp::PopLoadAddress; // fetch data
            }),
            //
            MicroOp::Execute($func),
        ]
    };
}
pub(crate) use load_zero_page_indexed;

//
// Store Operations
//

macro_rules! store_zero_page {
    ($func: ident) => {
        &[
            MicroOp::LoadIncrPC, // fetch page zero page address
            MicroOp::PushZero,   // push implied hi zero byte
            MicroOp::Execute($func),
            MicroOp::PopStoreAddress, // store data
        ]
    };
}
pub(crate) use store_zero_page;

macro_rules! store_absolute {
    ($func: ident) => {
        &[
            MicroOp::LoadIncrPC, // fetch low order address byte
            MicroOp::LoadIncrPC, // fetch high order address byte
            MicroOp::Execute($func),
            MicroOp::PopStoreAddress, // store data
        ]
    };
}
pub(crate) use store_absolute;

macro_rules! store_indirect_x {
    ($func: ident) => {
        &[
            MicroOp::LoadIncrPC, // fetch page zero base address
            MicroOp::PopTemp,    // temp = bal
            MicroOp::EmptyCycle, // pause for one cycle
            //
            MicroOp::AddTempX,       // temp = bal + x
            MicroOp::PushTemp,       // push temp onto stack
            MicroOp::PushZero,       // push hi zero byte
            MicroOp::PopLoadAddress, // fetch low order address byte
            //
            MicroOp::IncrTemp,       // temp = bal + x + 1
            MicroOp::PushTemp,       // push temp onto stack
            MicroOp::PushZero,       // push hi zero byte
            MicroOp::PopLoadAddress, // fetch high order address byte
            //
            MicroOp::Execute($func),
            MicroOp::PopStoreAddress, // store data
        ]
    };
}
pub(crate) use store_indirect_x;

macro_rules! store_indirect_y {
    ($func: ident) => {
        &[
            MicroOp::LoadIncrPC, // fetch page zero indirect address
            MicroOp::PopTemp,    // temp = ial
            //
            MicroOp::PushTemp,       // push temp onto stack
            MicroOp::PushZero,       // push hi zero byte
            MicroOp::PopLoadAddress, // fetch low order address byte of base address
            //
            MicroOp::IncrTemp,       // temp = ial + 1
            MicroOp::PushTemp,       // push temp onto stack
            MicroOp::PushZero,       // push hi zero byte
            MicroOp::PopLoadAddress, // fetch high order address byte of base address
            //
            MicroOp::Evaluate(|cpu, ctx| {
                let bal = ctx.pop();
                let bah = ctx.pop();

                let (lo, carry) = cpu.registers.y.safe_add(bal);
                let hi = bah + (carry as u8);

                ctx.push(lo);
                ctx.push(hi);
                return MicroOp::EmptyCycle; // pause one cycle
            }),
            //
            MicroOp::Execute($func),
            MicroOp::PopStoreAddress, // store data
        ]
    };
}
pub(crate) use store_indirect_y;

macro_rules! store_absolute_indexed {
    ($func: ident, $register: ident) => {
        &[
            MicroOp::LoadIncrPC, // fetch low order base address byte
            MicroOp::LoadIncrPC, // fetch high order base address byte
            //
            MicroOp::Evaluate(|cpu, ctx| {
                let bah = ctx.pop();
                let bal = ctx.pop();

                let (lo, carry) = cpu.registers.$register.safe_add(bal);
                let hi = bah + (carry as u8);

                ctx.push(lo);
                ctx.push(hi);
                return MicroOp::EmptyCycle; // pause for one cycle
            }),
            //
            MicroOp::Execute($func),
            MicroOp::PopStoreAddress, // store data
        ]
    };
}
pub(crate) use store_absolute_indexed;

macro_rules! store_zero_page_indexed {
    ($func: ident, $register: ident) => {
        &[
            MicroOp::LoadIncrPC, // fetch page zero base address
            MicroOp::EmptyCycle, // pause for one cycle
            //
            MicroOp::Evaluate(|cpu, ctx| {
                let bal = ctx.pop();

                let (lo, _) = cpu.registers.$register.safe_add(bal);
                let hi = 0;

                ctx.push(lo);
                ctx.push(hi);

                $func(cpu, ctx);
                return MicroOp::PopStoreAddress;
            }),
        ]
    };
}
pub(crate) use store_zero_page_indexed;

//
// Read-Modify-Write Operations
//

macro_rules! load_store_zero_page {
    ($func: ident) => {
        &[
            MicroOp::LoadIncrPC, // fetch page zero page address
            MicroOp::PushTemp,   // temp = adl
            //
            MicroOp::PopTemp,         // push address lo byte
            MicroOp::PushZero,        // push implied hi zero byte
            MicroOp::PeekLoadAddress, // fetch data
            MicroOp::Execute($func),
            MicroOp::PopStoreAddress, // store data
        ]
    };
}
pub(crate) use load_store_zero_page;

macro_rules! load_store_absolute {
    ($func: ident) => {
        &[
            MicroOp::LoadIncrPC,      // fetch low order address byte
            MicroOp::LoadIncrPC,      // fetch high order address byte
            MicroOp::PeekLoadAddress, // fetch data
            MicroOp::Execute($func),  //
            MicroOp::PopStoreAddress, // store data
        ]
    };
}
pub(crate) use load_store_absolute;

macro_rules! load_store_zero_page_x {
    ($func: ident) => {
        &[
            MicroOp::LoadIncrPC,      // fetch page zero base address
            MicroOp::EmptyCycle,      // pause
            MicroOp::PopTemp,         // temp = bal
            MicroOp::AddTempX,        // temp = bal + x
            MicroOp::PushTemp,        // push lo address byte to stack
            MicroOp::PushZero,        // push hi zero address byte
            MicroOp::EmptyCycle,      // pause
            MicroOp::PeekLoadAddress, // fetch data
            MicroOp::Execute($func),  //
            MicroOp::PopStoreAddress, // store data
        ]
    };
}
pub(crate) use load_store_zero_page_x;

macro_rules! load_store_absolute_x {
    ($func: ident) => {
        &[
            MicroOp::LoadIncrPC, // fetch low order address byte
            MicroOp::LoadIncrPC, // fetch high order address byte
            MicroOp::EmptyCycle, // pause
            MicroOp::Evaluate(|cpu, ctx| {
                let bah = ctx.pop();
                let bal = ctx.pop();

                let (lo, carry) = cpu.registers.x.safe_add(bal);
                let hi = bah + (carry as u8);

                ctx.push(lo);
                ctx.push(hi);
                return MicroOp::PeekLoadAddress; // fetch data
            }),
            MicroOp::EmptyCycle, // pause
            MicroOp::PopStoreAddress,
            //
            MicroOp::PopTemp,         // temp = bal
            MicroOp::AddTempX,        // temp = bal + x
            MicroOp::PushTemp,        // push lo address byte to stack
            MicroOp::PushZero,        // push hi zero address byte
            MicroOp::EmptyCycle,      // pause
            MicroOp::PeekLoadAddress, // fetch data
            MicroOp::Execute($func),  //
            MicroOp::PopLoadAddress,  // store data
        ]
    };
}
pub(crate) use load_store_absolute_x;

//
// Miscellaneous Operations
//

macro_rules! push_implied {
    ($func: ident) => {
        &[
            MicroOp::EmptyCycle,     // pause
            MicroOp::Execute($func), //
            MicroOp::StoreDecrSP,    // store data
        ]
    };
}
pub(crate) use push_implied;

macro_rules! pull_implied {
    ($func: ident) => {
        &[
            MicroOp::EmptyCycle,     // pause
            MicroOp::EmptyCycle,     // pause
            MicroOp::IncrLoadSP,     // fetch data from stack
            MicroOp::Execute($func), //
        ]
    };
}
pub(crate) use pull_implied;

macro_rules! break_implied {
    ($func: ident) => {
        &[MicroOp::Unimplemented]
    };
}
pub(crate) use break_implied;

macro_rules! jump_to_subroutine_absolute {
    ($func: ident) => {
        &[
            MicroOp::LoadIncrPC,     // fetch low order byte of subroutine address
            MicroOp::EmptyCycle,     // pause
            MicroOp::PushPCH,        // push PC hi byte onto context
            MicroOp::StoreDecrSP,    // store data on cpu stack
            MicroOp::PushPCL,        // push PC lo byte onto context
            MicroOp::StoreDecrSP,    // store data on cpu stack
            MicroOp::LoadIncrPC,     // fetch high order byte of subroutine address
            MicroOp::Execute($func), //
            MicroOp::PopJump,        // jump to address
        ]
    };
}
pub(crate) use jump_to_subroutine_absolute;

macro_rules! return_from_subroutine_implied {
    ($func: ident) => {
        &[
            MicroOp::EmptyCycle,     // pause
            MicroOp::EmptyCycle,     // pause
            MicroOp::IncrLoadSP,     // pull PCL from stack
            MicroOp::PopTemp,        // temp = PCL
            MicroOp::IncrTemp,       // temp = PCL +1
            MicroOp::PushTemp,       //
            MicroOp::IncrLoadSP,     // pull PCH from stack
            MicroOp::Execute($func), //
            MicroOp::EmptyCycle,     // pause
            MicroOp::PopJump,        // jump to return address
        ]
    };
}
pub(crate) use return_from_subroutine_implied;

macro_rules! return_from_interrupt_implied {
    ($func: ident) => {
        &[
            MicroOp::EmptyCycle,     // pause
            MicroOp::EmptyCycle,     // pause
            MicroOp::IncrLoadSP,     // pull P from stack
            MicroOp::IncrLoadSP,     // pull PCL from stack
            MicroOp::IncrLoadSP,     // pull PCH from stack
            MicroOp::Execute($func), //
            MicroOp::PopJump,        // jump to return address
        ]
    };
}
pub(crate) use return_from_interrupt_implied;

macro_rules! jump_absolute {
    ($func: ident) => {
        &[
            MicroOp::LoadIncrPC,     // fetch low order byte of jump address
            MicroOp::LoadIncrPC,     // fetch high order byte of jump address
            MicroOp::Execute($func), //
            MicroOp::PopJump,        // jump to address
        ]
    };
}
pub(crate) use jump_absolute;

macro_rules! jump_indirect {
    ($func: ident) => {
        &[
            MicroOp::LoadIncrPC,      // fetch low order byte of indirect address
            MicroOp::LoadIncrPC,      // fetch high order byte of indirect address
            MicroOp::PeekLoadAddress, // fetch low order byte of jump address
            MicroOp::Evaluate(|_, ctx| {
                let lo = ctx.pop();

                let iah = ctx.pop();
                let ial = ctx.pop();

                ctx.push(lo);
                ctx.push(ial + 1);
                ctx.push(iah);

                return MicroOp::PopLoadAddress; // fetch high order byte of jump address
            }),
            MicroOp::Execute($func), //
            MicroOp::PopJump,        // jump to address
        ]
    };
}
pub(crate) use jump_indirect;

macro_rules! branch_relative {
    ($func: ident) => {
        &[
            MicroOp::LoadIncrPC,     // fetch branch offset
            MicroOp::Execute($func), //
            MicroOp::Evaluate(|cpu, ctx| {
                let result = ctx.pop();
                let offset = ctx.pop() as i8;
                ctx.temp.set(result);

                if result == 0 {
                    // skip if branch not taken
                    return MicroOp::EmptyNoCycle;
                }

                let pcl = cpu.registers.pc.get_lo_byte();
                let pch = cpu.registers.pc.get_hi_byte();

                let lo: u8;
                let overflow: bool;
                if offset >= 0 {
                    (lo, overflow) = pcl.overflowing_add(offset as u8);
                } else {
                    (lo, overflow) = pcl.overflowing_sub(offset.unsigned_abs());
                }

                if overflow {
                    // branch crosses page boundary
                    let hi = pch.wrapping_add(overflow as u8);
                    ctx.push(lo);
                    ctx.push(hi);
                    return MicroOp::EmptyCycle; // pause
                }

                // branch doesnt cross page boundary
                ctx.push(lo);
                ctx.push(pch);
                return MicroOp::PopJump;
            }),
            MicroOp::Evaluate(|_, ctx| {
                if ctx.temp.get() == 0 {
                    // branch was skipped
                    return MicroOp::EmptyNoCycle;
                } else if ctx.size() == 0 {
                    // branch was taken but since it didn't cross a page
                    // boundary the previous micro-op already jumped to
                    // the offset
                    return MicroOp::EmptyNoCycle;
                }

                // the branch was taken and it crossed a page boundary
                return MicroOp::PopJump;
            }),
        ]
    };
}
pub(crate) use branch_relative;
