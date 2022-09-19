use crate::utility::bitset;

use num_traits;

#[derive(Clone, Copy)]
pub struct Register<T>(T);

impl<T> Register<T>
where
    T: Copy,
{
    pub fn new(value: T) -> Self {
        Self { 0: value }
    }

    pub fn get(self) -> T {
        self.0
    }

    pub fn set(&mut self, value: T) {
        self.0 = value;
    }

    pub fn update(&mut self, updater: fn(T) -> T) -> T {
        self.0 = updater(self.0);
        return self.0;
    }
}

impl<T> Register<T>
where
    T: num_traits::CheckedAdd + num_traits::WrappingAdd,
{
    /// Adds a value to the register safely. It returns a tuple containing
    /// the result of the addition, as well as a boolean indicating the value
    /// of the `carry` flag.
    pub fn safe_add(self, value: T) -> (T, bool) {
        self.0
            .checked_add(&value)
            .map_or_else(|| (self.0.wrapping_add(&value), true), |v| (v, false))
    }
}

impl Register<u16> {
    pub fn get_bytes(self) -> [u8; 2] {
        return self.0.to_le_bytes();
    }

    pub fn get_lo_byte(self) -> u8 {
        let [lo, _] = self.0.to_le_bytes();
        return lo;
    }

    pub fn get_hi_byte(self) -> u8 {
        let [_, hi] = self.0.to_le_bytes();
        return hi;
    }
}

impl<T> std::fmt::Display for Register<T>
where
    T: ToString,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{}", self.0.to_string());
    }
}

// General Registers
// A  - Accumulator     (8-bit)
// Y  - Index Register  (8-bit)
// X  - Index Register  (8-bit)
// SP - Stack Pointer   (8-bit)
// PC - Program Counter (16-bit)

#[derive(Clone, Copy)]
pub struct Registers {
    pub acc: Register<u8>,
    pub y: Register<u8>,
    pub x: Register<u8>,
    pub sp: Register<u8>,
    pub pc: Register<u16>,
}

impl Registers {
    pub fn new() -> Self {
        Self {
            acc: Register::new(0),
            y: Register::new(0),
            x: Register::new(0),
            sp: Register::new(0),
            pc: Register::new(0),
        }
    }
}

impl std::fmt::Debug for Registers {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "Registers:\nA={} X={} Y={}\nSP={} PC={}",
            self.acc, self.x, self.y, self.sp, self.pc
        );
    }
}

// Status Register
// |N|V| |B|D|I|Z|C|
//  7 6 5 4 3 2 1 0
//
// N = Negative
// V = Overflow
// B = BRK Command
// D = Decimal Mode
// I = IRQ Disable
// Z = Zero
// C = Carry
bitset! {
    #[derive(Clone, Copy)]
    pub struct StatusFlags(u8);

    0 : carry => CARRY;
    1 : zero => ZERO;
    2 : irq_disable => INTERRUPT;
    3 : decimal_mode => DECIMAL;
    4 : brk_command => BREAK;
    6 : overflow => OVERFLOW;
    7 : negative => NEGATIVE;
}

impl std::fmt::Debug for StatusFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(
            f,
            "Status Flags:\nC Z I D B - V N\n{} {} {} {} {}   {} {}",
            self.get_carry() as u8,
            self.get_zero() as u8,
            self.get_irq_disable() as u8,
            self.get_decimal_mode() as u8,
            self.get_brk_command() as u8,
            self.get_overflow() as u8,
            self.get_negative() as u8
        );
    }
}

// Stack range = 0x0100 - 0x01FF
// System Vectors:
//   0xFFFA, 0xFFFB = NMI (Non-maskable Interrupt) vector (16-bit)
//   0xFFFC, 0xFFFD = RES (Reset) vector (16-bit)
//   0xFFFE, 0xFFFF = IRQ (Interrupt Request)  vector (16-bit)
//
// Startup sequence is 7 cycles, read 16-bit address in the reset vector  (0xFFFC, LB-HB)
// into the PC register. On 8th cycle, transfer control by performing JMP to the address.
