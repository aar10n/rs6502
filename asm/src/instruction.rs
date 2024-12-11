use phf::phf_map;

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
    pub mode: AddressMode,
    pub bytes: u8,
    pub cycles: u8,
}

pub struct Instruction {
    pub name: &'static str,
    pub opcodes: &'static [Opcode],
}

impl Instruction {
    pub fn find_by_name(name: &str) -> Option<&'static Instruction> {
        INSTRUCTIONS.get(name)
    }
}

static INSTRUCTIONS: phf::Map<&'static str, Instruction> = phf_map! {
    "adc" => Instruction {
        name: "adc",
        opcodes: &[
            Opcode { value: 0x69, mode: AddressMode::Immediate, bytes: 2, cycles: 2 },
            Opcode { value: 0x65, mode: AddressMode::ZeroPage, bytes: 2, cycles: 3 },
            Opcode { value: 0x75, mode: AddressMode::ZeroPageX, bytes: 2, cycles: 4 },
            Opcode { value: 0x6D, mode: AddressMode::Absolute, bytes: 3, cycles: 4 },
            Opcode { value: 0x7D, mode: AddressMode::AbsoluteX, bytes: 3, cycles: 4 },
            Opcode { value: 0x79, mode: AddressMode::AbsoluteY, bytes: 3, cycles: 4 },
            Opcode { value: 0x61, mode: AddressMode::IndirectX, bytes: 2, cycles: 6 },
            Opcode { value: 0x71, mode: AddressMode::IndirectY, bytes: 2, cycles: 5 },
        ],
    },
    "and" => Instruction {
        name: "and",
        opcodes: &[
            Opcode { value: 0x29, mode: AddressMode::Immediate, bytes: 2, cycles: 2 },
            Opcode { value: 0x25, mode: AddressMode::ZeroPage, bytes: 2, cycles: 3 },
            Opcode { value: 0x35, mode: AddressMode::ZeroPageX, bytes: 2, cycles: 4 },
            Opcode { value: 0x2D, mode: AddressMode::Absolute, bytes: 3, cycles: 4 },
            Opcode { value: 0x3D, mode: AddressMode::AbsoluteX, bytes: 3, cycles: 4 },
            Opcode { value: 0x39, mode: AddressMode::AbsoluteY, bytes: 3, cycles: 4 },
            Opcode { value: 0x21, mode: AddressMode::IndirectX, bytes: 2, cycles: 6 },
            Opcode { value: 0x31, mode: AddressMode::IndirectY, bytes: 2, cycles: 5 },
        ],
    },
    "asl" => Instruction {
        name: "asl",
        opcodes: &[
            Opcode { value: 0x0A, mode: AddressMode::Accumulator, bytes: 1, cycles: 2 },
            Opcode { value: 0x06, mode: AddressMode::ZeroPage, bytes: 2, cycles: 5 },
            Opcode { value: 0x16, mode: AddressMode::ZeroPageX, bytes: 2, cycles: 6 },
            Opcode { value: 0x0E, mode: AddressMode::Absolute, bytes: 3, cycles: 6 },
            Opcode { value: 0x1E, mode: AddressMode::AbsoluteX, bytes: 3, cycles: 7 },
        ],
    },
    "bcc" => Instruction {
        name: "bcc",
        opcodes: &[
            Opcode { value: 0x90, mode: AddressMode::Relative, bytes: 2, cycles: 2 },
        ],
    },
    "bcs" => Instruction {
        name: "bcs",
        opcodes: &[
            Opcode { value: 0xB0, mode: AddressMode::Relative, bytes: 2, cycles: 2 },
        ],
    },
    "beq" => Instruction {
        name: "beq",
        opcodes: &[
            Opcode { value: 0xF0, mode: AddressMode::Relative, bytes: 2, cycles: 2 },
        ],
    },
    "bit" => Instruction {
        name: "bit",
        opcodes: &[
            Opcode { value: 0x24, mode: AddressMode::ZeroPage, bytes: 2, cycles: 3 },
            Opcode { value: 0x2C, mode: AddressMode::Absolute, bytes: 3, cycles: 4 },
        ],
    },
    "bmi" => Instruction {
        name: "bmi",
        opcodes: &[
            Opcode { value: 0x30, mode: AddressMode::Relative, bytes: 2, cycles: 2 },
        ],
    },
    "bne" => Instruction {
        name: "bne",
        opcodes: &[
            Opcode { value: 0xD0, mode: AddressMode::Relative, bytes: 2, cycles: 2 },
        ],
    },
    "bpl" => Instruction {
        name: "bpl",
        opcodes: &[
            Opcode { value: 0x10, mode: AddressMode::Relative, bytes: 2, cycles: 2 },
        ],
    },
    "brk" => Instruction {
        name: "brk",
        opcodes: &[
            Opcode { value: 0x00, mode: AddressMode::Implied, bytes: 1, cycles: 7 },
        ],
    },
    "bvc" => Instruction {
        name: "bvc",
        opcodes: &[
            Opcode { value: 0x50, mode: AddressMode::Relative, bytes: 2, cycles: 2 },
        ],
    },
    "bvs" => Instruction {
        name: "bvs",
        opcodes: &[
            Opcode { value: 0x70, mode: AddressMode::Relative, bytes: 2, cycles: 2 },
        ],
    },
    "clc" => Instruction {
        name: "clc",
        opcodes: &[
            Opcode { value: 0x18, mode: AddressMode::Implied, bytes: 1, cycles: 2 },
        ],
    },
    "cld" => Instruction {
        name: "cld",
        opcodes: &[
            Opcode { value: 0xD8, mode: AddressMode::Implied, bytes: 1, cycles: 2 },
        ],
    },
    "cli" => Instruction {
        name: "cli",
        opcodes: &[
            Opcode { value: 0x58, mode: AddressMode::Implied, bytes: 1, cycles: 2 },
        ],
    },
    "clv" => Instruction {
        name: "clv",
        opcodes: &[
            Opcode { value: 0xB8, mode: AddressMode::Implied, bytes: 1, cycles: 2 },
        ],
    },
    "cmp" => Instruction {
        name: "cmp",
        opcodes: &[
            Opcode { value: 0xC9, mode: AddressMode::Immediate, bytes: 2, cycles: 2 },
            Opcode { value: 0xC5, mode: AddressMode::ZeroPage, bytes: 2, cycles: 3 },
            Opcode { value: 0xD5, mode: AddressMode::ZeroPageX, bytes: 2, cycles: 4 },
            Opcode { value: 0xCD, mode: AddressMode::Absolute, bytes: 3, cycles: 4 },
            Opcode { value: 0xDD, mode: AddressMode::AbsoluteX, bytes: 3, cycles: 4 },
            Opcode { value: 0xD9, mode: AddressMode::AbsoluteY, bytes: 3, cycles: 4 },
            Opcode { value: 0xC1, mode: AddressMode::IndirectX, bytes: 2, cycles: 6 },
            Opcode { value: 0xD1, mode: AddressMode::IndirectY, bytes: 2, cycles: 5 },
        ],
    },
    "cpx" => Instruction {
        name: "cpx",
        opcodes: &[
            Opcode { value: 0xE0, mode: AddressMode::Immediate, bytes: 2, cycles: 2 },
            Opcode { value: 0xE4, mode: AddressMode::ZeroPage, bytes: 2, cycles: 3 },
            Opcode { value: 0xEC, mode: AddressMode::Absolute, bytes: 3, cycles: 4 },
        ],
    },
    "cpy" => Instruction {
        name: "cpy",
        opcodes: &[
            Opcode { value: 0xC0, mode: AddressMode::Immediate, bytes: 2, cycles: 2 },
            Opcode { value: 0xC4, mode: AddressMode::ZeroPage, bytes: 2, cycles: 3 },
            Opcode { value: 0xCC, mode: AddressMode::Absolute, bytes: 3, cycles: 4 },
        ],
    },
    "dec" => Instruction {
        name: "dec",
        opcodes: &[
            Opcode { value: 0xC6, mode: AddressMode::ZeroPage, bytes: 2, cycles: 5 },
            Opcode { value: 0xD6, mode: AddressMode::ZeroPageX, bytes: 2, cycles: 6 },
            Opcode { value: 0xCE, mode: AddressMode::Absolute, bytes: 3, cycles: 6 },
            Opcode { value: 0xDE, mode: AddressMode::AbsoluteX, bytes: 3, cycles: 7 },
        ],
    },
    "dex" => Instruction {
        name: "dex",
        opcodes: &[
            Opcode { value: 0xCA, mode: AddressMode::Implied, bytes: 1, cycles: 2 },
        ],
    },
    "dey" => Instruction {
        name: "dey",
        opcodes: &[
            Opcode { value: 0x88, mode: AddressMode::Implied, bytes: 1, cycles: 2 },
        ],
    },
    "eor" => Instruction {
        name: "eor",
        opcodes: &[
            Opcode { value: 0x49, mode: AddressMode::Immediate, bytes: 2, cycles: 2 },
            Opcode { value: 0x45, mode: AddressMode::ZeroPage, bytes: 2, cycles: 3 },
            Opcode { value: 0x55, mode: AddressMode::ZeroPageX, bytes: 2, cycles: 4 },
            Opcode { value: 0x4D, mode: AddressMode::Absolute, bytes: 3, cycles: 4 },
            Opcode { value: 0x5D, mode: AddressMode::AbsoluteX, bytes: 3, cycles: 4 },
            Opcode { value: 0x59, mode: AddressMode::AbsoluteY, bytes: 3, cycles: 4 },
            Opcode { value: 0x41, mode: AddressMode::IndirectX, bytes: 2, cycles: 6 },
            Opcode { value: 0x51, mode: AddressMode::IndirectY, bytes: 2, cycles: 5 },
        ],
    },
    "inc" => Instruction {
        name: "inc",
        opcodes: &[
            Opcode { value: 0xE6, mode: AddressMode::ZeroPage, bytes: 2, cycles: 5 },
            Opcode { value: 0xF6, mode: AddressMode::ZeroPageX, bytes: 2, cycles: 6 },
            Opcode { value: 0xEE, mode: AddressMode::Absolute, bytes: 3, cycles: 6 },
            Opcode { value: 0xFE, mode: AddressMode::AbsoluteX, bytes: 3, cycles: 7 },
        ],
    },
    "inx" => Instruction {
        name: "inx",
        opcodes: &[
            Opcode { value: 0xE8, mode: AddressMode::Implied, bytes: 1, cycles: 2 },
        ],
    },
    "iny" => Instruction {
        name: "iny",
        opcodes: &[
            Opcode { value: 0xC8, mode: AddressMode::Implied, bytes: 1, cycles: 2 },
        ],
    },
    "jmp" => Instruction {
        name: "jmp",
        opcodes: &[
            Opcode { value: 0x4C, mode: AddressMode::Absolute, bytes: 3, cycles: 3 },
            Opcode { value: 0x6C, mode: AddressMode::Indirect, bytes: 3, cycles: 5 },
        ],
    },
    "jsr" => Instruction {
        name: "jsr",
        opcodes: &[
            Opcode { value: 0x20, mode: AddressMode::Absolute, bytes: 3, cycles: 6 },
        ],
    },
    "lda" => Instruction {
        name: "lda",
        opcodes: &[
            Opcode { value: 0xA9, mode: AddressMode::Immediate, bytes: 2, cycles: 2 },
            Opcode { value: 0xA5, mode: AddressMode::ZeroPage, bytes: 2, cycles: 3 },
            Opcode { value: 0xB5, mode: AddressMode::ZeroPageX, bytes: 2, cycles: 4 },
            Opcode { value: 0xAD, mode: AddressMode::Absolute, bytes: 3, cycles: 4 },
            Opcode { value: 0xBD, mode: AddressMode::AbsoluteX, bytes: 3, cycles: 4 },
            Opcode { value: 0xB9, mode: AddressMode::AbsoluteY, bytes: 3, cycles: 4 },
            Opcode { value: 0xA1, mode: AddressMode::IndirectX, bytes: 2, cycles: 6 },
            Opcode { value: 0xB1, mode: AddressMode::IndirectY, bytes: 2, cycles: 5 },
        ],
    },
    "ldx" => Instruction {
        name: "ldx",
        opcodes: &[
            Opcode { value: 0xA2, mode: AddressMode::Immediate, bytes: 2, cycles: 2 },
            Opcode { value: 0xA6, mode: AddressMode::ZeroPage, bytes: 2, cycles: 3 },
            Opcode { value: 0xB6, mode: AddressMode::ZeroPageY, bytes: 2, cycles: 4 },
            Opcode { value: 0xAE, mode: AddressMode::Absolute, bytes: 3, cycles: 4 },
            Opcode { value: 0xBE, mode: AddressMode::AbsoluteY, bytes: 3, cycles: 4 },
        ],
    },
    "ldy" => Instruction {
        name: "ldy",
        opcodes: &[
            Opcode { value: 0xA0, mode: AddressMode::Immediate, bytes: 2, cycles: 2 },
            Opcode { value: 0xA4, mode: AddressMode::ZeroPage, bytes: 2, cycles: 3 },
            Opcode { value: 0xB4, mode: AddressMode::ZeroPageX, bytes: 2, cycles: 4 },
            Opcode { value: 0xAC, mode: AddressMode::Absolute, bytes: 3, cycles: 4 },
            Opcode { value: 0xBC, mode: AddressMode::AbsoluteX, bytes: 3, cycles: 4 },
        ],
    },
    "lsr" => Instruction {
        name: "lsr",
        opcodes: &[
            Opcode { value: 0x4A, mode: AddressMode::Accumulator, bytes: 1, cycles: 2 },
            Opcode { value: 0x46, mode: AddressMode::ZeroPage, bytes: 2, cycles: 5 },
            Opcode { value: 0x56, mode: AddressMode::ZeroPageX, bytes: 2, cycles: 6 },
            Opcode { value: 0x4E, mode: AddressMode::Absolute, bytes: 3, cycles: 6 },
            Opcode { value: 0x5E, mode: AddressMode::AbsoluteX, bytes: 3, cycles: 7 },
        ],
    },
    "nop" => Instruction {
        name: "nop",
        opcodes: &[
            Opcode { value: 0xEA, mode: AddressMode::Implied, bytes: 1, cycles: 2 },
        ],
    },
    "ora" => Instruction {
        name: "ora",
        opcodes: &[
            Opcode { value: 0x09, mode: AddressMode::Immediate, bytes: 2, cycles: 2 },
            Opcode { value: 0x05, mode: AddressMode::ZeroPage, bytes: 2, cycles: 3 },
            Opcode { value: 0x15, mode: AddressMode::ZeroPageX, bytes: 2, cycles: 4 },
            Opcode { value: 0x0D, mode: AddressMode::Absolute, bytes: 3, cycles: 4 },
            Opcode { value: 0x1D, mode: AddressMode::AbsoluteX, bytes: 3, cycles: 4 },
            Opcode { value: 0x19, mode: AddressMode::AbsoluteY, bytes: 3, cycles: 4 },
            Opcode { value: 0x01, mode: AddressMode::IndirectX, bytes: 2, cycles: 6 },
            Opcode { value: 0x11, mode: AddressMode::IndirectY, bytes: 2, cycles: 5 },
        ],
    },
    "pha" => Instruction {
        name: "pha",
        opcodes: &[
            Opcode { value: 0x48, mode: AddressMode::Implied, bytes: 1, cycles: 3 },
        ],
    },
    "php" => Instruction {
        name: "php",
        opcodes: &[
            Opcode { value: 0x08, mode: AddressMode::Implied, bytes: 1, cycles: 3 },
        ],
    },
    "pla" => Instruction {
        name: "pla",
        opcodes: &[
            Opcode { value: 0x68, mode: AddressMode::Implied, bytes: 1, cycles: 4 },
        ],
    },
    "plp" => Instruction {
        name: "plp",
        opcodes: &[
            Opcode { value: 0x28, mode: AddressMode::Implied, bytes: 1, cycles: 4 },
        ],
    },
    "rol" => Instruction {
        name: "rol",
        opcodes: &[
            Opcode { value: 0x2A, mode: AddressMode::Accumulator, bytes: 1, cycles: 2 },
            Opcode { value: 0x26, mode: AddressMode::ZeroPage, bytes: 2, cycles: 5 },
            Opcode { value: 0x36, mode: AddressMode::ZeroPageX, bytes: 2, cycles: 6 },
            Opcode { value: 0x2E, mode: AddressMode::Absolute, bytes: 3, cycles: 6 },
            Opcode { value: 0x3E, mode: AddressMode::AbsoluteX, bytes: 3, cycles: 7 },
        ],
    },
    "ror" => Instruction {
        name: "ror",
        opcodes: &[
            Opcode { value: 0x6A, mode: AddressMode::Accumulator, bytes: 1, cycles: 2 },
            Opcode { value: 0x66, mode: AddressMode::ZeroPage, bytes: 2, cycles: 5 },
            Opcode { value: 0x76, mode: AddressMode::ZeroPageX, bytes: 2, cycles: 6 },
            Opcode { value: 0x6E, mode: AddressMode::Absolute, bytes: 3, cycles: 6 },
            Opcode { value: 0x7E, mode: AddressMode::AbsoluteX, bytes: 3, cycles: 7 },
        ],
    },
    "rti" => Instruction {
        name: "rti",
        opcodes: &[
            Opcode { value: 0x40, mode: AddressMode::Implied, bytes: 1, cycles: 6 },
        ],
    },
    "rts" => Instruction {
        name: "rts",
        opcodes: &[
            Opcode { value: 0x60, mode: AddressMode::Implied, bytes: 1, cycles: 6 },
        ],
    },
    "sbc" => Instruction {
        name: "sbc",
        opcodes: &[
            Opcode { value: 0xE9, mode: AddressMode::Immediate, bytes: 2, cycles: 2 },
            Opcode { value: 0xE5, mode: AddressMode::ZeroPage, bytes: 2, cycles: 3 },
            Opcode { value: 0xF5, mode: AddressMode::ZeroPageX, bytes: 2, cycles: 4 },
            Opcode { value: 0xED, mode: AddressMode::Absolute, bytes: 3, cycles: 4 },
            Opcode { value: 0xFD, mode: AddressMode::AbsoluteX, bytes: 3, cycles: 4 },
            Opcode { value: 0xF9, mode: AddressMode::AbsoluteY, bytes: 3, cycles: 4 },
            Opcode { value: 0xE1, mode: AddressMode::IndirectX, bytes: 2, cycles: 6 },
            Opcode { value: 0xF1, mode: AddressMode::IndirectY, bytes: 2, cycles: 5 },
        ],
    },
    "sec" => Instruction {
        name: "sec",
        opcodes: &[
            Opcode { value: 0x38, mode: AddressMode::Implied, bytes: 1, cycles: 2 },
        ],
    },
    "sed" => Instruction {
        name: "sed",
        opcodes: &[
            Opcode { value: 0xF8, mode: AddressMode::Implied, bytes: 1, cycles: 2 },
        ],
    },
    "sei" => Instruction {
        name: "sei",
        opcodes: &[
            Opcode { value: 0x78, mode: AddressMode::Implied, bytes: 1, cycles: 2 },
        ],
    },
    "sta" => Instruction {
        name: "sta",
        opcodes: &[
            Opcode { value: 0x85, mode: AddressMode::ZeroPage, bytes: 2, cycles: 3 },
            Opcode { value: 0x95, mode: AddressMode::ZeroPageX, bytes: 2, cycles: 4 },
            Opcode { value: 0x8D, mode: AddressMode::Absolute, bytes: 3, cycles: 4 },
            Opcode { value: 0x9D, mode: AddressMode::AbsoluteX, bytes: 3, cycles: 5 },
            Opcode { value: 0x99, mode: AddressMode::AbsoluteY, bytes: 3, cycles: 5 },
            Opcode { value: 0x81, mode: AddressMode::IndirectX, bytes: 2, cycles: 6 },
            Opcode { value: 0x91, mode: AddressMode::IndirectY, bytes: 2, cycles: 6 },
        ],
    },
    "stx" => Instruction {
        name: "stx",
        opcodes: &[
            Opcode { value: 0x86, mode: AddressMode::ZeroPage, bytes: 2, cycles: 3 },
            Opcode { value: 0x96, mode: AddressMode::ZeroPageY, bytes: 2, cycles: 4 },
            Opcode { value: 0x8E, mode: AddressMode::Absolute, bytes: 3, cycles: 4 },
        ],
    },
    "sty" => Instruction {
        name: "sty",
        opcodes: &[
            Opcode { value: 0x84, mode: AddressMode::ZeroPage, bytes: 2, cycles: 3 },
            Opcode { value: 0x94, mode: AddressMode::ZeroPageX, bytes: 2, cycles: 4 },
            Opcode { value: 0x8C, mode: AddressMode::Absolute, bytes: 3, cycles: 4 },
        ],
    },
    "tax" => Instruction {
        name: "tax",
        opcodes: &[
            Opcode { value: 0xAA, mode: AddressMode::Implied, bytes: 1, cycles: 2 },
        ],
    },
    "tay" => Instruction {
        name: "tay",
        opcodes: &[
            Opcode { value: 0xA8, mode: AddressMode::Implied, bytes: 1, cycles: 2 },
        ],
    },
    "tsx" => Instruction {
        name: "tsx",
        opcodes: &[
            Opcode { value: 0xBA, mode: AddressMode::Implied, bytes: 1, cycles: 2 },
        ],
    },
    "txa" => Instruction {
        name: "txa",
        opcodes: &[
            Opcode { value: 0x8A, mode: AddressMode::Implied, bytes: 1, cycles: 2 },
        ],
    },
    "txs" => Instruction {
        name: "txs",
        opcodes: &[
            Opcode { value: 0x9A, mode: AddressMode::Implied, bytes: 1, cycles: 2 },
        ],
    },
    "tya" => Instruction {
        name: "tya",
        opcodes: &[
            Opcode { value: 0x98, mode: AddressMode::Implied, bytes: 1, cycles: 2 },
        ],
    },
};
