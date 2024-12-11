use std::collections::HashMap;

use crate::{
    error::SyntaxError,
    instruction::{Instruction, Opcode},
    token::{RawToken, Token, TokenKind},
    utils::*,
};

enum IRCode<'a> {
    Label(&'a str),
    Symbol(&'a str),
    Opcode(&'static Opcode, &'a [Token<'a>]),
    Expression(Vec<Token<'a>>),
    Value(u32),
}

//
//
//

pub fn assemble<'a>(tokens: &'a [RawToken<'a>]) -> Result<Vec<u8>, SyntaxError> {
    let tokens = process_raw_tokens(tokens);

    let ir = assembler_pass_one(&mut &tokens[..])?;
    let bytes = assembler_pass_two(&mut &ir[..])?;
    Ok(vec![])
}

fn process_raw_tokens<'a>(raw_tokens: &'a [RawToken<'a>]) -> Vec<Token<'a>> {
    let mut out_tokens = Vec::<Token>::with_capacity(raw_tokens.len());

    for raw_token in raw_tokens {
        if let Some(token) = Token::from_raw_token(raw_token) {
            out_tokens.push(token)
        }
    }
    out_tokens
}

/// The first assembler pass which produces an IR output.
fn assembler_pass_one<'f, 'a>(
    tokens: &'f mut &'a [Token<'a>],
) -> Result<Vec<IRCode<'a>>, SyntaxError> {
    Ok(vec![])
}

/// The second assembler pass which produces the final binary output.
fn assembler_pass_two<'f, 'a>(ir: &'f mut &'a [IRCode<'a>]) -> Result<Vec<u8>, SyntaxError> {
    Ok(vec![])
}

//
//
//

/*

line    = org-directive
        | eq-directive
        | db-directive
        | instruction
        ;


org-directive   = ".org" number;
eq-directive    = symbol ".eq" number;
db-directive    = [label] (".db" | ".bytes") literal {',' literal};

instruction     = [label] mnemonic operand;

operand         =
                | '#' (number | character);
                |
                ;

value-expr      = '(' value-expr ')'
                | '-' value-expr
                | value-lit operator value-expr
                ;

value-lit       = symbol | character | number;

label   = identifier [':'];
literal = number | character | string;


mnemonic   = identifier
symbol     = identifier

identifier = <built-in>
number     = <built-in>
character  = <built-in>
string     = <built-in>
operator   = <built-in>


    lda #

*/

fn parse_line<'f, 'a>(line: &'f mut &'a [Token<'a>]) -> Result<Vec<IRCode<'a>>, SyntaxError> {
    if line.is_empty() {
        return Ok(vec![]);
    }

    let token = take_one(line).unwrap();
    match token.kind {
        TokenKind::Identifier => {
            let name = token.value();
            if let Some(instr) = Instruction::find_by_name(name) {}
        }
        _ => {}
    }

    let first = take_one(line).unwrap();
    if first.is_identifier() {}

    //
    Ok(vec![])
}

// fn parse_instruction<'f, 'a>()

// fn parse_expression<'f, 'a>(tokens: &'f mut &'a [Token<'a>]) -> Result<>

// immediate '#<number>'

//
