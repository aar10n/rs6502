use crate::cpu::CPU;
use crate::microcode::Context;
use crate::registers::StatusFlags;

struct Value(u8, StatusFlags);

// pub const fn carrying_add(self, rhs: Self, carry: bool) -> (Self, bool) {
//     let (sum, carry) = (self as $UnsignedT).carrying_add(rhs as $UnsignedT, carry);
//     (sum as $SelfT, carry)
// }

// note: longer-term this should be done via an intrinsic, but this has been shown
// //   to generate optimal code for now, and LLVM doesn't have an equivalent intrinsic
// let (a, b) = self.overflowing_sub(rhs);
// let (c, d) = a.overflowing_sub(borrow as $SelfT);
// (c, b | d)

impl Value {
    fn new(value: u8, status: StatusFlags) -> Self {
        Self(value, status)
    }

    fn unwrap(self) -> (u8, StatusFlags) {
        (self.0, self.1)
    }

    fn safe_add(self, rhs: u8) -> Self {
        let Value(lhs, status) = self;

        let (result, carry) = lhs.overflowing_add(rhs);
        return Value(result, status.with_carry(carry));
    }

    fn safe_sub(self, rhs: u8) -> Self {
        let Value(lhs, status) = self;

        let (result, overflow) = (lhs as i8).overflowing_sub(rhs as i8);
        return Value(result as u8, status.with_overflow(overflow));
    }

    fn carrying_add(self, rhs: u8) -> Self {
        let Value(lhs, status) = self;
        let carry = status.get_carry() as u8;

        let (a, b) = lhs.overflowing_add(rhs);
        let (c, d) = a.overflowing_add(carry);

        return Value(c, status.with_carry(b | d));
    }

    fn borrowing_sub(self, rhs: u8) -> Self {
        let Value(lhs, status) = self;
        let borrow = !status.get_carry() as i8; // invert carry

        let (a, b) = (lhs as i8).overflowing_sub(rhs as i8);
        let (c, d) = a.overflowing_sub(borrow);

        return Value(c as u8, status.with_overflow(b | d));
    }

    fn update<F: Fn(u8, StatusFlags) -> (u8, StatusFlags)>(self, f: F) -> Self {
        let Value(value, status) = self;
        let (new_value, new_status) = f(value, status);
        return Value(new_value, new_status);
    }

    fn update_value<F: Fn(u8) -> u8>(self, f: F) -> Self {
        let Value(value, status) = self;
        return Value(f(value), status);
    }

    fn update_status<F: Fn(StatusFlags) -> StatusFlags>(self, f: F) -> Self {
        let Value(value, status) = self;
        return Value(value, f(status));
    }

    fn update_v_flag(self) -> Self {
        let Value(value, status) = self;
        return Value(value, status.with_overflow(value & 0x80 != 0));
    }

    fn update_z_flag(self) -> Self {
        let Value(value, status) = self;
        return Value(value, status.with_zero(value == 0));
    }

    fn update_zn_flags(self) -> Self {
        let Value(value, status) = self;
        return Value(
            value,
            status
                .with_zero(value == 0)
                .with_negative((value as i8) < 0),
        );
    }

    fn update_zv_flags(self) -> Self {
        let Value(value, status) = self;
        return Value(
            value,
            status
                .with_zero(value == 0)
                .with_overflow(value & 0x80 != 0),
        );
    }
}

// Addressing Modes:
//   - Accumulator (one byte, operation is on the accumulator)
//   - Immediate (two bytes, second byte is the operand)
//   - Absolute (three bytes, second byte is address lo-byte, third is hi-byte)
//   - Zero page (two bytes, second byte is lo-byte, implied zero hi-byte)
//   - Indexed zero page (two bytes, second byte is added to index register, implied zero hi-byte)
//   - Indexed absolute (three bytes, index reigster is added to second and third bytes)
//   - Implied (address containing operand is given in opcode)
//   - Relative (two bytes, second byte is added to pc register)
//   - Indexed indirect (two bytes, second byte is added to X reigster, address is at this zero-page address) [Indirect, X]
//   - Indirect indexed (two bytes, second byte contains zero-page address that is added to Y register) [Indirect], Y
//   - Absolute indirect (three bytes, second and third byte make up address which is loaded into pc register)

/// ADC - Add with Carry
///
/// A,Z,C,N = A+M+C
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Immediate    | 0x69   | 2     | 2
/// Zero Page    | 0x65   | 2     | 3
/// Zero Page,X  | 0x75   | 2     | 4
/// Absolute     | 0x6D   | 3     | 4
/// Absolute,X   | 0x7D   | 3     | 4 (+1)
/// Absolute,Y   | 0x79   | 3     | 4 (+1)
/// (Indirect,X) | 0x61   | 2     | 6
/// (Indirect),Y | 0x71   | 2     | 5 (+1)
pub fn adc_impl(cpu: &mut CPU, ctx: &mut Context) {
    let acc = cpu.registers.acc.get();
    let value = ctx.pop();

    let (result, status) = Value::new(acc, cpu.status)
        .carrying_add(value)
        .update_zv_flags()
        .unwrap();

    cpu.registers.acc.set(result);
    cpu.status.replace(status);
}

/// AND - Bitwise AND with Accumulator
///
/// A,Z,N = A & M
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Immediate    | 0x29   | 2     | 2
/// Zero Page    | 0x25   | 2     | 3
/// Zero Page,X  | 0x35   | 2     | 4
/// Absolute     | 0x2D   | 3     | 4
/// Absolute,X   | 0x3D   | 3     | 4 (+1)
/// Absolute,Y   | 0x39   | 3     | 4 (+1)
/// (Indirect,X) | 0x21   | 2     | 6
/// (Indirect),Y | 0x31   | 2     | 5 (+1)
pub fn and_impl(cpu: &mut CPU, ctx: &mut Context) {
    let acc = cpu.registers.acc.get();
    let value = ctx.pop();

    let (result, status) = Value::new(acc, cpu.status)
        .update_value(|v| v & value)
        .update_zv_flags()
        .unwrap();

    cpu.registers.acc.set(result);
    cpu.status.replace(status);
}

/// ASL - Arithmetic Shift Left One Bit (Memory or Accumulator)
///
/// A,Z,C,N = M * 2 or M,Z,C,N = M * 2
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Accumulator  | 0x0A   | 1     | 2
/// Zero Page    | 0x06   | 2     | 5
/// Zero Page,X  | 0x16   | 2     | 6
/// Absolute     | 0x0E   | 3     | 6
/// Absolute,X   | 0x1E   | 3     | 7
pub fn asl_impl(cpu: &mut CPU, ctx: &mut Context) {
    let value = ctx.pop();
    let carry = (value & 0x80) != 0;

    let (result, status) = Value::new(value, cpu.status)
        .update_value(|v| v << 1)
        .update_status(|s| s.with_carry(carry))
        .update_zn_flags()
        .update_v_flag()
        .unwrap();

    cpu.status.replace(status);
    ctx.push(result);
}

/// BCC - Branch on Carry Clear
///
/// branch on C = 0
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Relative     | 0x90   | 2     | 2 (+2)
pub fn bcc_impl(cpu: &mut CPU, ctx: &mut Context) {
    ctx.push(!cpu.status.get_carry() as u8);
}

/// BCS - Branch on Carry Set
///
/// branch on C = 1
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Relative     | 0xB0   | 2     | 2 (+2)
pub fn bcs_impl(cpu: &mut CPU, ctx: &mut Context) {
    ctx.push(cpu.status.get_carry() as u8);
}

/// BEQ - Branch on Result Zero
///
/// branch on Z = 1
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Relative     | 0xF0   | 2     | 2 (+2)
pub fn beq_impl(cpu: &mut CPU, ctx: &mut Context) {
    ctx.push(cpu.status.get_zero() as u8);
}

/// BIT - Test Bits in Memory with Accumulator
///
/// A AND M, M7 -> N, M6 -> V
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Zero Page    | 0x24   | 2     | 3
/// Absolute     | 0x2C   | 3     | 4
pub fn bit_impl(cpu: &mut CPU, ctx: &mut Context) {
    let acc = cpu.registers.acc.get();
    let value = ctx.pop();

    let b7 = (value & 0x80) != 0;
    let b6 = (value & 0x40) != 0;

    let (result, status) = Value::new(acc, cpu.status)
        .update_value(|v| v & value)
        .update_status(|s| s.with_negative(b7).with_overflow(b6))
        .update_z_flag()
        .unwrap();

    cpu.status.replace(status);
    ctx.push(result);
}

/// BMI - Branch on Result Minus
///
/// branch on N = 1
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Relative     | 0x30   | 2     | 2
pub fn bmi_impl(cpu: &mut CPU, ctx: &mut Context) {
    ctx.push(cpu.status.get_negative() as u8);
}

/// BNE - Branch on Result not Zero
///
/// branch on Z = 0
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Relative     | 0xD0   | 2     | 2
pub fn bne_impl(cpu: &mut CPU, ctx: &mut Context) {
    ctx.push(!cpu.status.get_zero() as u8);
}

/// BPL - Branch on Result Plus
///
/// branch on N = 0
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Relative     | 0x10   | 2     | 2
pub fn bpl_impl(cpu: &mut CPU, ctx: &mut Context) {
    ctx.push(!cpu.status.get_negative() as u8);
}

/// BRK - Force Break
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Implied      | 0x00   | 1     | 7
pub fn brk_impl(cpu: &mut CPU, ctx: &mut Context) {}

/// BVC - Branch on Overflow Clear
///
/// branch on V = 0
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Relative     | 0x50   | 2     | 2
pub fn bvc_impl(cpu: &mut CPU, ctx: &mut Context) {
    ctx.push(!cpu.status.get_overflow() as u8);
}

/// BVS - Branch on Overflow Set
///
/// branch on V = 1
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Relative     | 0x70   | 2     | 2
pub fn bvs_impl(cpu: &mut CPU, ctx: &mut Context) {
    ctx.push(cpu.status.get_overflow() as u8);
}

/// CLC - Clear Carry Flag
///
/// 0 -> C
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Implied      | 0x18   | 1     | 2
pub fn clc_impl(cpu: &mut CPU, _: &mut Context) {
    cpu.status.replace(cpu.status.with_carry(false));
}

/// CLD - Clear Decimal Flag
///
/// 0 -> D
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Implied      | 0xD8   | 1     | 2
pub fn cld_impl(cpu: &mut CPU, _: &mut Context) {
    cpu.status.replace(cpu.status.with_decimal_mode(false));
}

/// CLI - Clear Interrupt Disable Flag
///
/// 0 -> I
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Implied      | 0x58   | 1     | 2
pub fn cli_impl(cpu: &mut CPU, _: &mut Context) {
    cpu.status.replace(cpu.status.with_irq_disable(false));
}

/// CLV - Clear Overflow Flag
///
/// 0 -> V
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Implied      | 0xB8   | 1     | 2
pub fn clv_impl(cpu: &mut CPU, _: &mut Context) {
    cpu.status.replace(cpu.status.with_overflow(false));
}

/// CMP - Compare Memory with Accumulator
///
/// A - M
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Immediate    | 0xC9   | 2     | 2
/// Zero Page    | 0xC5   | 2     | 3
/// Zero Page,X  | 0xD5   | 2     | 4
/// Absolute     | 0xCD   | 3     | 4
/// Absolute,X   | 0xDD   | 3     | 4
/// Absolute,Y   | 0xD9   | 3     | 4
/// (Indirect,X) | 0xC1   | 2     | 6
/// (Indirect),Y | 0xD1   | 2     | 5
pub fn cmp_impl(cpu: &mut CPU, ctx: &mut Context) {
    let acc = cpu.registers.acc.get();
    let value = ctx.pop();

    let (_, status) = Value::new(acc, cpu.status)
        .safe_sub(value)
        .update_zn_flags()
        .unwrap();

    cpu.status.replace(status);
}

/// CPX - Compare Memory and Index X
///
/// X - M
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Immediate    | 0xE0   | 2     | 2
/// Zero Page    | 0xE4   | 2     | 3
/// Absolute     | 0xEC   | 3     | 4
pub fn cpx_impl(cpu: &mut CPU, ctx: &mut Context) {
    let x = cpu.registers.x.get();
    let value = ctx.pop();

    let (_, status) = Value::new(x, cpu.status)
        .safe_sub(value)
        .update_zn_flags()
        .unwrap();

    cpu.status.replace(status);
}

/// CPY - Compare Memory and Index Y
///
/// Y - M
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Immediate    | 0xC0   | 2     | 2
/// Zero Page    | 0xC4   | 2     | 3
/// Absolute     | 0xCC   | 3     | 4
pub fn cpy_impl(cpu: &mut CPU, ctx: &mut Context) {
    let y = cpu.registers.y.get();
    let value = ctx.pop();

    let (_, status) = Value::new(y, cpu.status)
        .safe_sub(value)
        .update_zn_flags()
        .unwrap();

    cpu.status.replace(status);
}

/// DEC - Decrement Memory by One
///
/// M - 1 -> M
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Zero Page    | 0xC6   | 2     | 5
/// Zero Page,X  | 0xD6   | 2     | 6
/// Absolute     | 0xCE   | 3     | 6
/// Absolute,X   | 0xDE   | 3     | 7
pub fn dec_impl(cpu: &mut CPU, ctx: &mut Context) {
    let value = ctx.pop();
    let overflow = cpu.status.get_overflow();

    let (result, status) = Value::new(value, cpu.status)
        .safe_sub(1)
        .update_zn_flags()
        .update_status(|sts| sts.with_overflow(overflow))
        .unwrap();

    cpu.status.replace(status);
    ctx.push(result);
}

/// DEX - Decrement Index X by One
///
/// X - 1 -> X
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Implied      | 0xCA   | 1     | 2
pub fn dex_impl(cpu: &mut CPU, _: &mut Context) {
    let x = cpu.registers.x.get();
    let overflow = cpu.status.get_overflow();

    let (result, status) = Value::new(x, cpu.status)
        .safe_sub(1)
        .update_zn_flags()
        .update_status(|sts| sts.with_overflow(overflow))
        .unwrap();

    cpu.registers.x.set(result);
    cpu.status.replace(status);
}

/// DEY - Decrement Index Y by One
///
/// Y - 1 -> Y
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Implied      | 0x88   | 1     | 2
pub fn dey_impl(cpu: &mut CPU, _: &mut Context) {
    let y = cpu.registers.y.get();
    let overflow = cpu.status.get_overflow();

    let (result, status) = Value::new(y, cpu.status)
        .safe_sub(1)
        .update_zn_flags()
        .update_status(|sts| sts.with_overflow(overflow))
        .unwrap();

    cpu.registers.y.set(result);
    cpu.status.replace(status);
}

/// EOR - "Exclusive-Or" Memory with Accumulator
///
/// A EOR M -> A
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Immediate    | 0x49   | 2     | 2
/// Zero Page    | 0x45   | 2     | 3
/// Zero Page,X  | 0x55   | 2     | 4
/// Absolute     | 0x4D   | 3     | 4
/// Absolute,X   | 0x5D   | 3     | 4
/// Absolute,Y   | 0x59   | 3     | 4
/// (Indirect,X) | 0x41   | 2     | 6
/// (Indirect),Y | 0x51   | 2     | 5
pub fn eor_impl(cpu: &mut CPU, ctx: &mut Context) {
    let acc = cpu.registers.acc.get();
    let value = ctx.pop();

    let (result, status) = Value::new(acc, cpu.status)
        .update_value(|v| v ^ value)
        .update_zn_flags()
        .unwrap();

    cpu.registers.acc.set(result);
    cpu.status.replace(status);
}

/// INC - Increment Memory by One
///
/// M + 1 -> M
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Zero Page    | 0xE6   | 2     | 5
/// Zero Page,X  | 0xF6   | 2     | 6
/// Absolute     | 0xEE   | 3     | 6
/// Absolute,X   | 0xFE   | 3     | 7
pub fn inc_impl(cpu: &mut CPU, ctx: &mut Context) {
    let value = ctx.pop();
    let carry = cpu.status.get_carry();

    let (result, status) = Value::new(value, cpu.status)
        .safe_add(1)
        .update_zn_flags()
        .update_status(|sts| sts.with_carry(carry))
        .unwrap();

    cpu.status.replace(status);
    ctx.push(result);
}

/// INX - Increment Index X by One
///
/// X + 1 -> X
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Implied      | 0xE8   | 1     | 2
pub fn inx_impl(cpu: &mut CPU, _: &mut Context) {
    let x = cpu.registers.x.get();
    let carry = cpu.status.get_carry();

    let (result, status) = Value::new(x, cpu.status)
        .safe_add(1)
        .update_zn_flags()
        .update_status(|sts| sts.with_carry(carry))
        .unwrap();

    cpu.registers.x.set(result);
    cpu.status.replace(status);
}

/// INY - Increment Index Y by One
///
/// Y + 1 -> Y
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Implied      | 0xC8   | 1     | 2
pub fn iny_impl(cpu: &mut CPU, _: &mut Context) {
    let y = cpu.registers.y.get();
    let carry = cpu.status.get_carry();

    let (result, status) = Value::new(y, cpu.status)
        .safe_add(1)
        .update_zn_flags()
        .update_status(|sts| sts.with_carry(carry))
        .unwrap();

    cpu.registers.y.set(result);
    cpu.status.replace(status);
}

/// JMP - Jump to New Location
///
/// (PC+1) -> PCL
/// (PC+2) -> PCH
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Absolute     | 0x4C   | 3     | 3
/// Indirect     | 0x6C   | 3     | 5
pub fn jmp_impl(_: &mut CPU, _: &mut Context) {}

/// JSR - Jump to New Location Saving Return Address
///
/// push (PC+2)
/// (PC+1) -> PCL
/// (PC+2) -> PCH
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Absolute     | 0x20   | 3     | 6
pub fn jsr_impl(_: &mut CPU, _: &mut Context) {}

/// LDA - Load Accumulator with Memory
///
/// M -> A
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Immediate    | 0xA9   | 2     | 2
/// Zero Page    | 0xA5   | 2     | 3
/// Zero Page,X  | 0xB5   | 2     | 4
/// Absolute     | 0xAD   | 3     | 4
/// Absolute,X   | 0xBD   | 3     | 4
/// Absolute,Y   | 0xB9   | 3     | 4
/// (Indirect,X) | 0xA1   | 2     | 6
/// (Indirect),Y | 0xB1   | 2     | 5
pub fn lda_impl(cpu: &mut CPU, ctx: &mut Context) {
    let data = ctx.pop();
    cpu.registers.acc.set(data);
}

/// LDX - Load Index X with Memory
///
/// M -> X
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Immediate    | 0xA2   | 2     | 2
/// Zero Page    | 0xA6   | 2     | 3
/// Zero Page,Y  | 0xB6   | 2     | 4
/// Absolute     | 0xAE   | 3     | 4
/// Absolute,Y   | 0xBE   | 3     | 4
pub fn ldx_impl(cpu: &mut CPU, ctx: &mut Context) {
    let data = ctx.pop();
    cpu.registers.x.set(data);
}

/// LDY - Load Index Y with Memory
///
/// M -> Y
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Immediate    | 0xA0   | 2     | 2
/// Zero Page    | 0xA4   | 2     | 3
/// Zero Page,X  | 0xB4   | 2     | 4
/// Absolute     | 0xAC   | 3     | 4
/// Absolute,X   | 0xBC   | 3     | 4
pub fn ldy_impl(cpu: &mut CPU, ctx: &mut Context) {
    let data = ctx.pop();
    cpu.registers.y.set(data);
}

/// LSR - Shift One Bit Right (Memory or Accumulator)
///
/// 0 -> [76543210] -> C
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Accumulator  | 0x4A   | 1     | 2
/// Zero Page    | 0x46   | 2     | 5
/// Zero Page,X  | 0x56   | 2     | 6
/// Absolute     | 0x4E   | 3     | 6
/// Absolute,X   | 0x5E   | 3     | 7
pub fn lsr_impl(cpu: &mut CPU, ctx: &mut Context) {
    let value = ctx.pop();
    let carry = (value & 0x1) != 0;

    let (result, status) = Value::new(value, cpu.status)
        .update_value(|v| v >> 1)
        .update_status(|s| s.with_carry(carry).with_negative(false))
        .update_z_flag()
        .unwrap();

    cpu.status.replace(status);
    ctx.push(result);
}

/// NOP - No Operation
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Implied      | 0xEA   | 1     | 2
pub fn nop_impl(_: &mut CPU, _: &mut Context) {}

/// ORA - "OR" Memory with Accumulator
///
/// A OR M -> A
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Immediate    | 0x09   | 2     | 2
/// Zero Page    | 0x05   | 2     | 3
/// Zero Page,X  | 0x15   | 2     | 4
/// Absolute     | 0x0D   | 3     | 4
/// Absolute,X   | 0x1D   | 3     | 4
/// Absolute,Y   | 0x19   | 3     | 4
/// (Indirect,X) | 0x01   | 2     | 6
/// (Indirect),Y | 0x11   | 2     | 5
pub fn ora_impl(cpu: &mut CPU, ctx: &mut Context) {
    let acc = cpu.registers.acc.get();
    let value = ctx.pop();

    let (result, status) = Value::new(acc, cpu.status)
        .update_value(|v| v | value)
        .update_zn_flags()
        .unwrap();

    cpu.registers.acc.set(result);
    cpu.status.replace(status);
}

/// PHA - Push Accumulator on Stack
///
/// A -> stack
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Implied      | 0x48   | 1     | 3
pub fn pha_impl(cpu: &mut CPU, ctx: &mut Context) {
    let acc = cpu.registers.acc.get();
    ctx.push(acc);
}

/// PHP - Push Processor Status on Stack
///
/// SR -> stack
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Implied      | 0x08   | 1     | 3
pub fn php_impl(cpu: &mut CPU, ctx: &mut Context) {
    let status = cpu.status.get_raw();
    ctx.push(status);
}

/// PLA - Pull Accumulator from Stack
///
/// stack -> A
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Implied      | 0x68   | 1     | 4
pub fn pla_impl(cpu: &mut CPU, ctx: &mut Context) {
    let acc = ctx.pop();
    cpu.registers.acc.set(acc);
}

/// PLP - Pull Processor Status from Stack
///
/// stack -> SR
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Implied      | 0x28   | 1     | 4
pub fn plp_impl(cpu: &mut CPU, ctx: &mut Context) {
    let status = ctx.pop();
    cpu.status.set_raw(status);
}

/// ROL - Rotate One Bit Left (Memory or Accumulator)
///
/// C <- [76543210] <- C
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Accumulator  | 0x2A   | 1     | 2
/// Zero Page    | 0x26   | 2     | 5
/// Zero Page,X  | 0x36   | 2     | 6
/// Absolute     | 0x2E   | 3     | 6
/// Absolute,X   | 0x3E   | 3     | 7
pub fn rol_impl(cpu: &mut CPU, ctx: &mut Context) {
    let value = ctx.pop();
    let carry = (value & 0x80) != 0;

    let (result, status) = Value::new(value, cpu.status)
        .update_value(|v| v.rotate_left(1))
        .update_status(|s| s.with_carry(carry))
        .update_zn_flags()
        .unwrap();

    cpu.status.replace(status);
    ctx.push(result);
}

/// ROR - Rotate One Bit Right (Memory or Accumulator)
///
/// C -> [76543210] -> C
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Accumulator  | 0x6A   | 1     | 2
/// Zero Page    | 0x66   | 2     | 5
/// Zero Page,X  | 0x76   | 2     | 6
/// Absolute     | 0x6E   | 3     | 6
/// Absolute,X   | 0x7E   | 3     | 7
pub fn ror_impl(cpu: &mut CPU, ctx: &mut Context) {
    let value = ctx.pop();
    let carry = (value & 0x1) != 0;

    let (result, status) = Value::new(value, cpu.status)
        .update_value(|v| v.rotate_right(1))
        .update_status(|s| s.with_carry(carry))
        .update_zn_flags()
        .unwrap();

    cpu.status.replace(status);
    ctx.push(result);
}

/// RTI - Return from Interrupt
///
/// stack -> SR
/// stack -> PC
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Implied      | 0x40   | 1     | 6
pub fn rti_impl(cpu: &mut CPU, ctx: &mut Context) {
    let pch = ctx.pop();
    let pcl = ctx.pop();

    let status = ctx.pop();
    cpu.status.set_raw(status);

    ctx.push(pcl);
    ctx.push(pch);
}

/// RTS - Return from Subroutine
///
/// stack -> PC
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Implied      | 0x60   | 1     | 6
pub fn rts_impl(_: &mut CPU, _: &mut Context) {
    // nothing to do
}

/// SBC - Subtract Memory from Accumulator with Borrow
///
/// A - M - C -> A
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Immediate    | 0xE9   | 2     | 2
/// Zero Page    | 0xE5   | 2     | 3
/// Zero Page,X  | 0xF5   | 2     | 4
/// Absolute     | 0xED   | 3     | 4
/// Absolute,X   | 0xFD   | 3     | 4
/// Absolute,Y   | 0xF9   | 3     | 4
/// (Indirect,X) | 0xE1   | 2     | 6
/// (Indirect),Y | 0xF1   | 2     | 5
pub fn sbc_impl(cpu: &mut CPU, ctx: &mut Context) {
    let acc = cpu.registers.acc.get();
    let value = ctx.pop();

    let (result, status) = Value::new(acc, cpu.status)
        .borrowing_sub(value)
        .update_zn_flags()
        .update_v_flag()
        .unwrap();

    cpu.registers.acc.set(result);
    cpu.status.replace(status);
}

/// SEC - Set Carry Flag
///
/// 1 -> C
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Implied      | 0x38   | 1     | 2
pub fn sec_impl(cpu: &mut CPU, _: &mut Context) {
    cpu.status.replace(cpu.status.with_carry(true));
}

/// SED - Set Decimal Flag
///
/// 1 -> D
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Implied      | 0xF8   | 1     | 2
pub fn sed_impl(cpu: &mut CPU, _: &mut Context) {
    cpu.status.replace(cpu.status.with_decimal_mode(true));
}

/// SEI - Set Interrupt Disable Status
///
/// 1 -> I
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Implied      | 0x78   | 1     | 2
pub fn sei_impl(cpu: &mut CPU, _: &mut Context) {
    cpu.status.replace(cpu.status.with_irq_disable(true));
}

/// STA - Store Accumulator in Memory
///
/// A -> M
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Zero Page    | 0x85   | 2     | 3
/// Zero Page,X  | 0x95   | 2     | 4
/// Absolute     | 0x8D   | 3     | 4
/// Absolute,X   | 0x9D   | 3     | 5
/// Absolute,Y   | 0x99   | 3     | 5
/// (Indirect,X) | 0x81   | 2     | 6
/// (Indirect),Y | 0x91   | 2     | 6
pub fn sta_impl(cpu: &mut CPU, ctx: &mut Context) {
    let acc = cpu.registers.acc.get();
    ctx.push(acc);
}

/// STX - Store Index X in Memory
///
/// X -> M
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Zero Page    | 0x86   | 2     | 3
/// Zero Page,Y  | 0x96   | 2     | 4
/// Absolute     | 0x8E   | 3     | 4
pub fn stx_impl(cpu: &mut CPU, ctx: &mut Context) {
    let x = cpu.registers.x.get();
    ctx.push(x);
}

/// STY - Store Index Y in Memory
///
/// Y -> M
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Zero Page    | 0x84   | 2     | 3
/// Zero Page,X  | 0x94   | 2     | 4
/// Absolute     | 0x8C   | 3     | 4
pub fn sty_impl(cpu: &mut CPU, ctx: &mut Context) {
    let y = cpu.registers.y.get();
    ctx.push(y);
}

/// TAX - Transfer Accumulator to Index X
///
/// A -> X
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Implied      | 0xAA   | 1     | 2
pub fn tax_impl(cpu: &mut CPU, _: &mut Context) {
    let acc = cpu.registers.acc.get();
    cpu.registers.x.set(acc);
}

/// TAY - Transfer Accumulator to Index Y
///
/// A -> Y
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Implied      | 0xA8   | 1     | 2
pub fn tay_impl(cpu: &mut CPU, _: &mut Context) {
    let acc = cpu.registers.acc.get();
    cpu.registers.y.set(acc);
}

/// TSX - Transfer Stack Pointer to Index X
///
/// SP -> X
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Implied      | 0xBA   | 1     | 2
pub fn tsx_impl(cpu: &mut CPU, _: &mut Context) {
    let sp = cpu.registers.sp.get();
    cpu.registers.x.set(sp);
}

/// TXA - Transfer Index X to Accumulator
///
/// X -> A
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Implied      | 0x8A   | 1     | 2
pub fn txa_impl(cpu: &mut CPU, _: &mut Context) {
    let sp = cpu.registers.sp.get();
    cpu.registers.acc.set(sp);
}

/// TXS - Transfer Index X to Stack Register
///
/// X -> SP
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Implied      | 0x9A   | 1     | 2
pub fn txs_impl(cpu: &mut CPU, _: &mut Context) {
    let x = cpu.registers.x.get();
    cpu.registers.sp.set(x);
}

/// TYA - Transfer Index Y to Accumulator
///
/// Y -> A
///
/// address mode | opcode | bytes | cycles
/// -------------+--------+-------+-------
/// Implied      | 0x98   | 1     | 2
pub fn tya_impl(cpu: &mut CPU, _: &mut Context) {
    let y = cpu.registers.y.get();
    cpu.registers.sp.set(y);
}
