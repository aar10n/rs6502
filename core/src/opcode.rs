use crate::instructions::*;
use crate::microcode::*;

macro_rules! opcode {
    ($value: expr) => {
        Opcode {
            value: $value,
            mnemonic: "",
            mode: AddressMode::Implied,
            bytes: 0,
            cycles: 0,
            ucode: None,
        }
    };

    ($value: expr, $name: literal, $mode: expr, $bytes: literal, $cycles: literal, $ucode: expr) => {
        Opcode {
            value: $value,
            mnemonic: $name,
            mode: $mode,
            bytes: $bytes,
            cycles: $cycles,
            ucode: Some($ucode),
        }
    };
}

#[derive(Clone, Copy)]
pub enum AddressMode {
    Accumulator,
    Absolute,
    AbsoluteX,
    AbsoluteY,
    Immediate,
    Implied,
    Indirect,
    IndirectX,
    IndirectY,
    Relative,
    ZeroPage,
    ZeroPageX,
    ZeroPageY,
}

#[derive(Clone)]
pub struct Opcode {
    pub value: u8,
    pub mnemonic: &'static str,
    pub mode: AddressMode,
    pub bytes: u8,
    pub cycles: u8,
    pub ucode: Option<&'static [MicroOp]>,
}

#[allow(dead_code)]
#[rustfmt::skip]
pub const OPCODES: [Opcode; 256] = [
    // 0x00 - 0x0F
    opcode!(0x00, "BRK", AddressMode::Implied, 1, 7, break_implied!(brk_impl)),
    opcode!(0x01, "ORA", AddressMode::IndirectX, 2, 5, load_indirect_x!(ora_impl)),
    opcode!(0x02),
    opcode!(0x03),
    opcode!(0x04),
    opcode!(0x05, "ORA", AddressMode::ZeroPage, 2, 3, load_zero_page!(ora_impl)),
    opcode!(0x06, "ASL", AddressMode::ZeroPage, 2, 5, load_store_zero_page!(asl_impl)),
    opcode!(0x07),
    opcode!(0x08, "PHP", AddressMode::Implied, 1, 3, push_implied!(php_impl)),
    opcode!(0x09, "ORA", AddressMode::Immediate, 2, 2, load_immediate!(ora_impl)),
    opcode!(0x0A, "ASL", AddressMode::Accumulator, 1, 2, single_byte_accumulator!(asl_impl)),
    opcode!(0x0B),
    opcode!(0x0C),
    opcode!(0x0D, "ORA", AddressMode::Absolute, 3, 4, load_zero_page!(ora_impl)),
    opcode!(0x0E, "ASL", AddressMode::Absolute, 3, 6, load_store_absolute!(asl_impl)),
    opcode!(0x0F),
    // 0x10 - 0x1F
    opcode!(0x10, "BPL", AddressMode::Relative, 2, 2, branch_relative!(bpl_impl)),
    opcode!(0x11, "ORA", AddressMode::IndirectY, 2, 5, load_indirect_y!(ora_impl)),
    opcode!(0x12),
    opcode!(0x13),
    opcode!(0x14),
    opcode!(0x15, "ORA", AddressMode::ZeroPageX, 2, 4, load_zero_page_indexed!(ora_impl, x)),
    opcode!(0x16, "ASL", AddressMode::ZeroPageX, 2, 6, load_store_zero_page_x!(asl_impl)),
    opcode!(0x17),
    opcode!(0x18, "CLC", AddressMode::Implied, 1, 2, single_byte_implied!(clc_impl)),
    opcode!(0x19, "ORA", AddressMode::AbsoluteY, 3, 4, load_absolute_indexed!(ora_impl, y)),
    opcode!(0x1A),
    opcode!(0x1B),
    opcode!(0x1C),
    opcode!(0x1D, "ORA", AddressMode::AbsoluteX, 3, 4, load_absolute_indexed!(ora_impl, x)),
    opcode!(0x1E, "ASL", AddressMode::AbsoluteX, 3, 7, load_store_absolute_x!(asl_impl)),
    opcode!(0x1F),
    // 0x20 - 0x2F
    opcode!(0x20, "JSR", AddressMode::Absolute, 3, 6, jump_to_subroutine_absolute!(jsr_impl)),
    opcode!(0x21, "AND", AddressMode::IndirectX, 2, 6, load_indirect_x!(and_impl)),
    opcode!(0x22),
    opcode!(0x23),
    opcode!(0x24, "BIT", AddressMode::ZeroPage, 2, 3, load_zero_page!(bit_impl)),
    opcode!(0x25, "AND", AddressMode::ZeroPage, 2, 3, load_zero_page!(and_impl)),
    opcode!(0x26, "ROL", AddressMode::ZeroPage, 2, 5, load_zero_page!(rol_impl)),
    opcode!(0x27),
    opcode!(0x28, "PLP", AddressMode::Implied, 1, 4, pull_implied!(plp_impl)),
    opcode!(0x29, "AND", AddressMode::Immediate, 2, 2, load_immediate!(and_impl)),
    opcode!(0x2A, "ROL", AddressMode::Accumulator, 1, 2, single_byte_accumulator!(rol_impl)),
    opcode!(0x2B),
    opcode!(0x2C, "BIT", AddressMode::Absolute, 3, 4, load_absolute!(bit_impl)),
    opcode!(0x2D, "AND", AddressMode::Absolute, 3, 4, load_absolute!(and_impl)),
    opcode!(0x2E, "ROL", AddressMode::Absolute, 3, 6, load_store_absolute!(rol_impl)),
    opcode!(0x2F),
    // 0x30 - 0x3F
    opcode!(0x30, "BMI", AddressMode::Relative, 2, 2, branch_relative!(bmi_impl)),
    opcode!(0x31, "AND", AddressMode::IndirectY, 2, 5, load_indirect_y!(and_impl)),
    opcode!(0x32),
    opcode!(0x33),
    opcode!(0x34),
    opcode!(0x35, "AND", AddressMode::ZeroPageX, 2, 4, load_zero_page_indexed!(and_impl, x)),
    opcode!(0x36, "ROL", AddressMode::ZeroPageX, 2, 6, load_store_zero_page_x!(rol_impl)),
    opcode!(0x37),
    opcode!(0x38, "SEC", AddressMode::Implied, 1, 2, single_byte_implied!(sec_impl)),
    opcode!(0x39, "AND", AddressMode::AbsoluteY, 3, 4, load_absolute_indexed!(and_impl, y)),
    opcode!(0x3A),
    opcode!(0x3B),
    opcode!(0x3C),
    opcode!(0x3D, "AND", AddressMode::AbsoluteX, 3, 4, load_absolute_indexed!(and_impl, x)),
    opcode!(0x3E, "ROL", AddressMode::AbsoluteX, 3, 7, load_store_absolute_x!(rol_impl)),
    opcode!(0x3F),
    // 0x40 - 0x4F
    opcode!(0x40, "RTI", AddressMode::Implied, 1, 6, return_from_interrupt_implied!(rti_impl)),
    opcode!(0x41, "EOR", AddressMode::IndirectX, 2, 6, load_indirect_x!(eor_impl)),
    opcode!(0x42),
    opcode!(0x43),
    opcode!(0x44),
    opcode!(0x45, "EOR", AddressMode::ZeroPage, 2, 3, load_zero_page!(eor_impl)),
    opcode!(0x46, "LSR", AddressMode::ZeroPage, 2, 5, load_store_zero_page!(lsr_impl)),
    opcode!(0x47),
    opcode!(0x48, "PHA", AddressMode::Implied, 1, 3, push_implied!(pha_impl)),
    opcode!(0x49, "EOR", AddressMode::Immediate, 2, 2, load_immediate!(eor_impl)),
    opcode!(0x4A, "LSR", AddressMode::Accumulator, 1, 2, single_byte_accumulator!(lsr_impl)),
    opcode!(0x4B),
    opcode!(0x4C, "JMP", AddressMode::Absolute, 3, 3, jump_absolute!(jmp_impl)),
    opcode!(0x4D, "EOR", AddressMode::Absolute, 3, 4, load_absolute!(eor_impl)),
    opcode!(0x4E, "LSR", AddressMode::Absolute, 3, 6, load_store_absolute!(lsr_impl)),
    opcode!(0x4F),
    // 0x50 - 0x5F
    opcode!(0x50, "BVC", AddressMode::Relative, 2, 2, branch_relative!(bvc_impl)),
    opcode!(0x51, "EOR", AddressMode::IndirectY, 2, 5, load_indirect_y!(eor_impl)),
    opcode!(0x52),
    opcode!(0x53),
    opcode!(0x54),
    opcode!(0x55, "EOR", AddressMode::ZeroPageX, 2, 4, load_zero_page_indexed!(eor_impl, x)),
    opcode!(0x56, "LSR", AddressMode::ZeroPageX, 2, 6, load_store_zero_page_x!(lsr_impl)),
    opcode!(0x57),
    opcode!(0x58, "CLI", AddressMode::Implied, 1, 2, single_byte_implied!(cli_impl)),
    opcode!(0x59, "EOR", AddressMode::AbsoluteY, 3, 4, load_absolute_indexed!(eor_impl, y)),
    opcode!(0x5A),
    opcode!(0x5B),
    opcode!(0x5C),
    opcode!(0x5D, "EOR", AddressMode::AbsoluteX, 3, 4, load_absolute_indexed!(eor_impl, x)),
    opcode!(0x5E, "LSR", AddressMode::AbsoluteX, 3, 7, load_store_absolute_x!(lsr_impl)),
    opcode!(0x5F),
    // 0x60 - 0x6F
    opcode!(0x60, "RTS", AddressMode::Implied, 1, 6, return_from_subroutine_implied!(rts_impl)),
    opcode!(0x61, "ADC", AddressMode::IndirectX, 2, 6, load_indirect_x!(adc_impl)),
    opcode!(0x62),
    opcode!(0x63),
    opcode!(0x64),
    opcode!(0x65, "ADC", AddressMode::ZeroPage, 2, 3, load_zero_page!(adc_impl)),
    opcode!(0x66, "ROR", AddressMode::ZeroPage, 2, 5, load_store_zero_page!(ror_impl)),
    opcode!(0x67),
    opcode!(0x68, "PLA", AddressMode::Implied, 1, 4, pull_implied!(pla_impl)),
    opcode!(0x69, "ADC", AddressMode::Immediate, 2, 2, load_immediate!(adc_impl)),
    opcode!(0x6A, "ROR", AddressMode::Accumulator, 1, 2, single_byte_accumulator!(ror_impl)),
    opcode!(0x6B),
    opcode!(0x6C, "JMP", AddressMode::Indirect, 3, 5, jump_indirect!(jmp_impl)),
    opcode!(0x6D, "ADC", AddressMode::Absolute, 3, 4, load_absolute!(adc_impl)),
    opcode!(0x6E, "ROR", AddressMode::Absolute, 3, 6, load_store_absolute!(ror_impl)),
    opcode!(0x6F),
    // 0x70 - 0x7F
    opcode!(0x70, "BVS", AddressMode::Relative, 2, 2, branch_relative!(bvs_impl)),
    opcode!(0x71, "ADC", AddressMode::IndirectY, 2, 5, load_indirect_y!(adc_impl)),
    opcode!(0x72),
    opcode!(0x73),
    opcode!(0x74),
    opcode!(0x75, "ADC", AddressMode::ZeroPageX, 2, 4, load_zero_page_indexed!(adc_impl, x)),
    opcode!(0x76, "ROR", AddressMode::ZeroPageX, 2, 6, load_store_zero_page_x!(ror_impl)),
    opcode!(0x77),
    opcode!(0x78, "SEI", AddressMode::Implied, 1, 2, single_byte_implied!(sei_impl)),
    opcode!(0x79, "ADC", AddressMode::AbsoluteY, 3, 4, load_absolute_indexed!(adc_impl, y)),
    opcode!(0x7A),
    opcode!(0x7B),
    opcode!(0x7C),
    opcode!(0x7D, "ADC", AddressMode::AbsoluteX, 3, 4, load_absolute_indexed!(adc_impl, x)),
    opcode!(0x7E, "ROR", AddressMode::AbsoluteX, 3, 7, load_store_absolute_x!(ror_impl)),
    opcode!(0x7F),
    // 0x80 - 0x8F
    opcode!(0x80),
    opcode!(0x81, "STA", AddressMode::IndirectX, 2, 6, store_indirect_x!(sta_impl)),
    opcode!(0x82),
    opcode!(0x83),
    opcode!(0x84, "STY", AddressMode::ZeroPage, 2, 3, store_zero_page!(sty_impl)),
    opcode!(0x85, "STA", AddressMode::ZeroPage, 2, 3, store_zero_page!(sta_impl)),
    opcode!(0x86, "STX", AddressMode::ZeroPage, 2, 3, store_zero_page!(stx_impl)),
    opcode!(0x87),
    opcode!(0x88, "DEY", AddressMode::Implied, 1, 2, single_byte_implied!(dey_impl)),
    opcode!(0x89),
    opcode!(0x8A, "TXA", AddressMode::Implied, 1, 2, single_byte_implied!(txa_impl)),
    opcode!(0x8B),
    opcode!(0x8C, "STY", AddressMode::Absolute, 3, 4, store_absolute!(sty_impl)),
    opcode!(0x8D, "STA", AddressMode::Absolute, 3, 4, store_absolute!(sta_impl)),
    opcode!(0x8E, "STX", AddressMode::Absolute, 3, 4, store_absolute!(stx_impl)),
    opcode!(0x8F),
    // 0x90 - 0x9F
    opcode!(0x90, "BCC", AddressMode::Relative, 2, 2, branch_relative!(bcc_impl)),
    opcode!(0x91, "STA", AddressMode::IndirectY, 2, 6, store_indirect_y!(sta_impl)),
    opcode!(0x92),
    opcode!(0x93),
    opcode!(0x94, "STY", AddressMode::ZeroPageX, 2, 4, store_zero_page_indexed!(sty_impl, x)),
    opcode!(0x95, "STA", AddressMode::ZeroPageX, 2, 4, store_zero_page_indexed!(sta_impl, x)),
    opcode!(0x96, "STX", AddressMode::ZeroPageY, 2, 4, store_zero_page_indexed!(stx_impl, y)),
    opcode!(0x97),
    opcode!(0x98, "TYA", AddressMode::Implied, 1, 2, single_byte_implied!(tya_impl)),
    opcode!(0x99, "STA", AddressMode::AbsoluteY, 3, 5, store_absolute_indexed!(sta_impl, y)),
    opcode!(0x9A, "TXS", AddressMode::Implied, 1, 2, single_byte_implied!(txs_impl)),
    opcode!(0x9B),
    opcode!(0x9C),
    opcode!(0x9D, "STA", AddressMode::AbsoluteX, 3, 5, store_absolute_indexed!(sta_impl, x)),
    opcode!(0x9E),
    opcode!(0x9F),
    // 0xA0 - 0xAF
    opcode!(0xA0, "LDY", AddressMode::Immediate, 2, 2, load_immediate!(ldy_impl)),
    opcode!(0xA1, "LDA", AddressMode::IndirectX, 2, 6, load_indirect_x!(lda_impl)),
    opcode!(0xA2, "LDX", AddressMode::Immediate, 2, 2, load_immediate!(ldx_impl)),
    opcode!(0xA3),
    opcode!(0xA4, "LDY", AddressMode::ZeroPage, 2, 3, load_zero_page!(ldy_impl)),
    opcode!(0xA5, "LDA", AddressMode::ZeroPage, 2, 3, load_zero_page!(lda_impl)),
    opcode!(0xA6, "LDX", AddressMode::ZeroPage, 2, 3, load_zero_page!(ldx_impl)),
    opcode!(0xA7),
    opcode!(0xA8, "TAY", AddressMode::Implied, 1, 2, single_byte_implied!(tay_impl)),
    opcode!(0xA9, "LDA", AddressMode::Immediate, 2, 2, load_immediate!(lda_impl)),
    opcode!(0xAA, "TAX", AddressMode::Implied, 1, 2, single_byte_implied!(tax_impl)),
    opcode!(0xAB),
    opcode!(0xAC, "LDY", AddressMode::Absolute, 3, 4, load_absolute!(ldy_impl)),
    opcode!(0xAD, "LDA", AddressMode::Absolute, 3, 4, load_absolute!(lda_impl)),
    opcode!(0xAE, "LDX", AddressMode::Absolute, 3, 4, load_absolute!(ldx_impl)),
    opcode!(0xAF),
    // 0xB0 - 0xBF
    opcode!(0xB0, "BCS", AddressMode::Relative, 2, 2, branch_relative!(bcs_impl)),
    opcode!(0xB1, "LDA", AddressMode::IndirectY, 2, 5, load_indirect_y!(lda_impl)),
    opcode!(0xB2),
    opcode!(0xB3),
    opcode!(0xB4, "LDY", AddressMode::ZeroPageX, 2, 4, load_zero_page_indexed!(ldy_impl, x)),
    opcode!(0xB5, "LDA", AddressMode::ZeroPageX, 2, 4, load_zero_page_indexed!(lda_impl, x)),
    opcode!(0xB6, "LDX", AddressMode::ZeroPageY, 2, 4, load_zero_page_indexed!(ldx_impl, y)),
    opcode!(0xB7),
    opcode!(0xB8, "CLV", AddressMode::Implied, 1, 2, single_byte_implied!(clv_impl)),
    opcode!(0xB9, "LDA", AddressMode::AbsoluteY, 3, 4, load_absolute_indexed!(lda_impl, y)),
    opcode!(0xBA, "TSX", AddressMode::Implied, 1, 2, single_byte_implied!(tsx_impl)),
    opcode!(0xBB),
    opcode!(0xBC, "LDY", AddressMode::AbsoluteX, 3, 4, load_absolute_indexed!(ldy_impl, x)),
    opcode!(0xBD, "LDA", AddressMode::AbsoluteX, 3, 4, load_absolute_indexed!(lda_impl, x)),
    opcode!(0xBE, "LDX", AddressMode::AbsoluteY, 3, 4, load_absolute_indexed!(ldx_impl, y)),
    opcode!(0xBF),
    // 0xC0 - 0xCF
    opcode!(0xC0, "CPY", AddressMode::ZeroPage, 2, 3, load_zero_page!(cpy_impl)),
    opcode!(0xC1, "CMP", AddressMode::IndirectX, 2, 6, load_indirect_x!(cmp_impl)),
    opcode!(0xC2),
    opcode!(0xC3),
    opcode!(0xC4, "CPY", AddressMode::Immediate, 2, 2, load_immediate!(cpy_impl)),
    opcode!(0xC5, "CMP", AddressMode::ZeroPage, 2, 3, load_zero_page!(cmp_impl)),
    opcode!(0xC6, "DEC", AddressMode::ZeroPage, 2, 5, load_store_zero_page!(dec_impl)),
    opcode!(0xC7),
    opcode!(0xC8, "INY", AddressMode::Implied, 1, 2, single_byte_implied!(iny_impl)),
    opcode!(0xC9, "CMP", AddressMode::Immediate, 2, 2, load_immediate!(cmp_impl)),
    opcode!(0xCA, "DEX", AddressMode::Implied, 1, 2, single_byte_implied!(dex_impl)),
    opcode!(0xCB),
    opcode!(0xCC, "CPY", AddressMode::Absolute, 3, 4, load_absolute!(cpy_impl)),
    opcode!(0xCD, "CMP", AddressMode::Absolute, 3, 4, load_absolute!(cmp_impl)),
    opcode!(0xCE, "DEC", AddressMode::Absolute, 3, 6, load_store_absolute!(dec_impl)),
    opcode!(0xCF),
    // 0xD0 - 0xDF
    opcode!(0xD0, "BNE", AddressMode::Relative, 2, 2, branch_relative!(bne_impl)),
    opcode!(0xD1, "CMP", AddressMode::IndirectY, 2, 5, load_indirect_y!(cmp_impl)),
    opcode!(0xD2),
    opcode!(0xD3),
    opcode!(0xD4),
    opcode!(0xD5, "CMP", AddressMode::ZeroPageX, 2, 4, load_zero_page_indexed!(cmp_impl, x)),
    opcode!(0xD6, "DEC", AddressMode::ZeroPageX, 2, 6, load_store_zero_page_x!(dec_impl)),
    opcode!(0xD7),
    opcode!(0xD8, "CLD", AddressMode::Implied, 1, 2, single_byte_implied!(cld_impl)),
    opcode!(0xD9, "CMP", AddressMode::AbsoluteY, 3, 4, load_absolute_indexed!(cmp_impl, y)),
    opcode!(0xDA),
    opcode!(0xDB),
    opcode!(0xDC),
    opcode!(0xDD, "CMP", AddressMode::AbsoluteX, 3, 4, load_absolute_indexed!(cmp_impl, x)),
    opcode!(0xDE, "DEC", AddressMode::AbsoluteX, 3, 7, load_store_absolute_x!(dec_impl)),
    opcode!(0xDF),
    // 0xE0 - 0xEF
    opcode!(0xE0, "CPX", AddressMode::Immediate, 2, 2, load_immediate!(cpx_impl)),
    opcode!(0xE1, "SBC", AddressMode::IndirectX, 2, 6, load_indirect_x!(sbc_impl)),
    opcode!(0xE2),
    opcode!(0xE3),
    opcode!(0xE4, "CPX", AddressMode::ZeroPage, 2, 3, load_zero_page!(cpx_impl)),
    opcode!(0xE5, "SBC", AddressMode::ZeroPage, 2, 3, load_zero_page!(sbc_impl)),
    opcode!(0xE6, "INC", AddressMode::ZeroPage, 2, 5, load_store_zero_page!(inc_impl)),
    opcode!(0xE7),
    opcode!(0xE8, "INX", AddressMode::Implied, 1, 2, single_byte_implied!(inx_impl)),
    opcode!(0xE9, "SBC", AddressMode::Immediate, 2, 2, load_immediate!(sbc_impl)),
    opcode!(0xEA, "NOP", AddressMode::Implied, 1, 2, single_byte_implied!(nop_impl)),
    opcode!(0xEB),
    opcode!(0xEC, "CPX", AddressMode::Absolute, 3, 4, load_absolute!(cpx_impl)),
    opcode!(0xED, "SBC", AddressMode::Absolute, 3, 4, load_absolute!(sbc_impl)),
    opcode!(0xEE, "INC", AddressMode::Absolute, 3, 6, load_store_absolute!(inc_impl)),
    opcode!(0xEF),
    // 0xF0 - 0xFF
    opcode!(0xF0, "BEQ", AddressMode::Relative, 2, 2, branch_relative!(beq_impl)),
    opcode!(0xF1, "SBC", AddressMode::IndirectY, 2, 5, load_indirect_y!(sbc_impl)),
    opcode!(0xF2),
    opcode!(0xF3),
    opcode!(0xF4),
    opcode!(0xF5, "SBC", AddressMode::ZeroPageX, 2, 4, load_zero_page_indexed!(sbc_impl, x)),
    opcode!(0xF6, "INC", AddressMode::ZeroPageX, 2, 6, load_store_zero_page_x!(inc_impl)),
    opcode!(0xF7),
    opcode!(0xF8, "SED", AddressMode::Implied, 1, 2, single_byte_implied!(sed_impl)),
    opcode!(0xF9, "SBC", AddressMode::AbsoluteY, 3, 4, load_absolute_indexed!(sbc_impl, y)),
    opcode!(0xFA),
    opcode!(0xFB),
    opcode!(0xFC),
    opcode!(0xFD, "SBC", AddressMode::AbsoluteX, 3, 4, load_absolute_indexed!(sbc_impl, x)),
    opcode!(0xFE, "INC", AddressMode::AbsoluteX, 3, 7, load_store_absolute_x!(inc_impl)),
    opcode!(0xFF),
];

pub fn decode_instruction(opcode: u8) -> &'static [MicroOp] {
    let decoded = &OPCODES[opcode as usize];
    if decoded.ucode.is_none() {
        // invalid instruction
        panic!("invalid opcode")
    }

    return &OPCODES[opcode as usize].ucode.unwrap();
}

pub fn decode_instruction_to_string(opcode: u8) -> &'static str {
    let decoded = &OPCODES[opcode as usize];
    if decoded.ucode.is_none() {
        // invalid instruction
        panic!("invalid opcode")
    }

    return &OPCODES[opcode as usize].mnemonic;
}
