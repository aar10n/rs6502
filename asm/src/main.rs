mod assembler;
mod error;
mod instruction;
mod preprocessor;
mod source;
mod token;
mod utils;

use colored::*;
use indoc::indoc;

use crate::preprocessor::preprocess;
use crate::source::{File, SourceMap};
use crate::token::tokens;

static SOURCE: &str = indoc! {"
%define STACK $0100
%define add(a)    ((a) + STACK)
%define add(a, b) (add(a) + b)

    add((1), 2)
    sta STACK
    hello 123 ; comment
    
LABEL: .db 0xa
    sta STACK
"};

fn run(file: &File) -> Result<(), String> {
    let raw_tokens = file.lex_tokens();
    println!("{}", "original".green());
    for line in 1..file.line_count() + 1 {
        println!("{:3} │ {}", line, file.get_source_line(line).unwrap());
    }

    // println!("{}", "tokens:".blue());
    // for token in tokens.iter() {
    //     if let Some(orig) = token.source.origin {
    //         let span = orig.span_loc();
    //         println!("{:?}: {:?} <{}>", token.kind(), token.source.value(), span);
    //     } else {
    //         println!("{:?}: {:?}", token.kind(), token.source.value());
    //     }
    // }

    let out_tokens = preprocess(&raw_tokens, vec![]).map_err(|err| format!("{:?}", err))?;
    let result = tokens::to_string(&out_tokens);
    println!("{}", "preprocessed:".green());
    for (index, line) in result.split("\n").enumerate() {
        println!("{:3} │ {}", index + 1, line);
    }

    println!();
    Ok(())
}

fn main() {
    let mut source_map = SourceMap::new();
    let file = source_map.add_from_string("<source>", SOURCE);
    if let Err(error) = run(&file) {
        println!("{}", error);
        std::process::exit(1);
    }
    std::process::exit(0);
}
