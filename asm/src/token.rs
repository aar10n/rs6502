use std::ops::Deref;

use logos::{Lexer, Logos};

use crate::source::File;
use crate::source::SourceRef;

pub trait TokenLike<'source>: Sized {
    type Kind: ?Sized + Clone + PartialEq;

    fn kind<'a>(&'a self) -> &'a Self::Kind;
    fn source<'a>(&'a self) -> &'a SourceRef<'source>;
    fn file<'a>(&'a self) -> &'source File;

    fn is_newline(&'source self) -> bool;
}

macro_rules! impl_TokenLike {
    ($token:tt, $kind:ty) => {
        impl<'source> TokenLike<'source> for $token<'source> {
            type Kind = $kind;

            fn kind<'a>(&'a self) -> &'a Self::Kind {
                &self.kind
            }

            fn source<'a>(&'a self) -> &'a SourceRef<'source> {
                &self.source
            }

            fn file(&self) -> &'source File {
                &self.source.file
            }

            fn is_newline(&self) -> bool {
                matches!(self.kind(), <$kind>::Newline)
            }
        }
    };
}

//
// Token
//

/// A token produced by the preprocessor.
pub struct Token<'source> {
    pub kind: TokenKind,
    pub source: SourceRef<'source>,
}

impl_TokenLike!(Token, TokenKind);

#[derive(Clone, Debug, PartialEq)]
pub enum TokenKind {
    Directive,
    Identifier,

    Literal(LitKind),
    Operator(OpKind),

    Comma,
    Colon,
    Hash,

    LParen,
    RParen,

    Newline,
}

impl TokenKind {
    pub fn from_raw_token<'a>(token: RawTokenKind) -> Option<Self> {
        match token {
            RawTokenKind::Directive => Some(Self::Directive),
            RawTokenKind::Identifier => Some(Self::Identifier),

            RawTokenKind::Number(v) => Some(Self::Literal(LitKind::Number(v))),
            RawTokenKind::Char(v) => Some(Self::Literal(LitKind::Char(v))),
            RawTokenKind::String(v) => Some(Self::Literal(LitKind::String(v))),

            RawTokenKind::Add => Some(Self::Operator(OpKind::Add)),
            RawTokenKind::Sub => Some(Self::Operator(OpKind::Sub)),
            RawTokenKind::Mul => Some(Self::Operator(OpKind::Mul)),
            RawTokenKind::Div => Some(Self::Operator(OpKind::Div)),
            RawTokenKind::Mod => Some(Self::Operator(OpKind::Mod)),
            RawTokenKind::Not => Some(Self::Operator(OpKind::Not)),
            RawTokenKind::And => Some(Self::Operator(OpKind::And)),
            RawTokenKind::Or => Some(Self::Operator(OpKind::Or)),
            RawTokenKind::Xor => Some(Self::Operator(OpKind::Xor)),
            RawTokenKind::Shl => Some(Self::Operator(OpKind::Shl)),
            RawTokenKind::Shr => Some(Self::Operator(OpKind::Shr)),

            RawTokenKind::Comma => Some(Self::Comma),
            RawTokenKind::Colon => Some(Self::Colon),
            RawTokenKind::Hash => Some(Self::Hash),
            RawTokenKind::LParen => Some(Self::LParen),
            RawTokenKind::RParen => Some(Self::RParen),

            RawTokenKind::Newline => Some(Self::Newline),

            _ => None,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum LitKind {
    Number(u64),
    Char(char),
    String(String),
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum OpKind {
    Add, // +
    Sub, // -
    Mul, // *
    Div, // /
    Mod, // %
    Not, // ~
    And, // &
    Or,  // |
    Xor, // ^
    Shl, // <<
    Shr, // >>
}

//
// RawToken
//

/// A token produced by the lexer.
///
/// This is an intermediate representation that is consumed by the preprocessor.
/// As such, it preserves whitespace and comments, and are stripped before they
/// are converted to their `Token` representation.
#[derive(Clone)]
pub struct RawToken<'source> {
    pub kind: RawTokenKind,
    pub source: SourceRef<'source>,
}

impl_TokenLike!(RawToken, RawTokenKind);

impl Deref for RawToken<'_> {
    type Target = RawTokenKind;

    fn deref(&self) -> &Self::Target {
        &self.kind
    }
}

impl std::fmt::Debug for RawToken<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.kind {
            RawTokenKind::PreProcessor | RawTokenKind::Directive | RawTokenKind::Identifier => {
                let value = self.source.value();
                let escaped = value
                    .chars()
                    .fold(String::with_capacity(value.len()), |acc, c| match c {
                        '\n' => acc + "\\n",
                        '\t' => acc + "\\t",
                        ' ' => acc + "⎕",
                        _ => format!("{}{}", acc, c),
                    });

                write!(f, "{:?}({})", self.kind, escaped)
            }
            _ => {
                write!(f, "{:?}", self.kind)
            }
        }
    }
}

#[derive(Logos, Clone, Debug, PartialEq, Eq)]
pub enum RawTokenKind {
    #[regex(r"%[a-z]+")]
    PreProcessor,

    #[regex(r"\.[a-z]+")]
    Directive,

    #[regex(r"[a-zA-Z]+")]
    Identifier,

    /* literals */
    #[regex(r"0b[01]+", conv_bin)] // binary
    #[regex(r"0o[0-7]+", conv_oct)] // octal
    #[regex(r"[0-9]+", conv_dec)] // decimal
    #[regex(r"(\$|0x)[a-fA-F0-9]+", conv_hex)] // hex
    Number(u64),
    #[regex(r"'([[:print:]]|\\[0ntfr])'", conv_char)]
    Char(char),
    #[regex(r#""[^"]*""#, conv_string)]
    String(String),

    /* operators */
    #[token("+")]
    Add,
    #[token("-")]
    Sub,
    #[token("*")]
    Mul,
    #[token("/")]
    Div,
    #[token("%")]
    Mod,
    #[token("~")]
    Not,
    #[token("&")]
    And,
    #[token("|")]
    Or,
    #[token("^")]
    Xor,
    #[token("<<")]
    Shl,
    #[token(">>")]
    Shr,

    /* punctuation */
    #[token(",")]
    Comma,
    #[token(":")]
    Colon,
    #[token("#")]
    Hash,

    /* delimiters */
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,

    #[token("\n")]
    Newline,
    #[regex(r"[ \t]+")]
    Whitespace,
    #[regex(";[^\n]*")]
    Comment,

    #[error]
    #[regex(r"\\\n", logos::skip)] // escaped newlines
    Error,
}

impl RawTokenKind {
    pub fn is_preprocessor(&self) -> bool {
        matches!(self, RawTokenKind::PreProcessor)
    }

    pub fn is_directive(&self) -> bool {
        matches!(self, RawTokenKind::Directive)
    }

    pub fn is_identifier(&self) -> bool {
        matches!(self, RawTokenKind::Identifier)
    }

    pub fn is_literal(&self) -> bool {
        matches!(
            self,
            RawTokenKind::Number(_) | RawTokenKind::Char(_) | RawTokenKind::String(_)
        )
    }

    pub fn is_operator(&self) -> bool {
        matches!(
            self,
            RawTokenKind::Add
                | RawTokenKind::Sub
                | RawTokenKind::Mul
                | RawTokenKind::Div
                | RawTokenKind::Mod
                | RawTokenKind::Not
                | RawTokenKind::And
                | RawTokenKind::Or
                | RawTokenKind::Xor
                | RawTokenKind::Shl
                | RawTokenKind::Shr
        )
    }

    pub fn is_comma(&self) -> bool {
        matches!(self, RawTokenKind::Comma)
    }

    pub fn is_lparen(&self) -> bool {
        matches!(self, RawTokenKind::LParen)
    }

    pub fn is_rparen(&self) -> bool {
        matches!(self, RawTokenKind::RParen)
    }

    pub fn is_newline(&self) -> bool {
        matches!(self, RawTokenKind::Newline)
    }

    pub fn is_whitespace(&self) -> bool {
        matches!(self, RawTokenKind::Whitespace)
    }

    pub fn is_comment(&self) -> bool {
        matches!(self, RawTokenKind::Comment)
    }
}

//

fn conv_bin(lex: &mut Lexer<RawTokenKind>) -> Option<u64> {
    // ex. 0b111
    let slice = lex.slice();
    return u64::from_str_radix(&slice[2..slice.len()], 2).ok();
}

fn conv_oct(lex: &mut Lexer<RawTokenKind>) -> Option<u64> {
    // ex. 0o123
    let slice = lex.slice();
    return u64::from_str_radix(&slice[2..slice.len()], 8).ok();
}

fn conv_dec(lex: &mut Lexer<RawTokenKind>) -> Option<u64> {
    // ex. 123
    let slice = lex.slice();
    return u64::from_str_radix(slice, 10).ok();
}

fn conv_hex(lex: &mut Lexer<RawTokenKind>) -> Option<u64> {
    // ex. 0xABC or $ABC
    let slice = lex.slice();
    let start_char = if slice.starts_with("$") { 1 } else { 2 };
    return u64::from_str_radix(&slice[start_char..], 16).ok();
}

fn conv_char(lex: &mut Lexer<RawTokenKind>) -> Option<char> {
    // ex. 'c'
    let slice = lex.slice();
    slice.chars().nth(1)
}

fn conv_string(lex: &mut Lexer<RawTokenKind>) -> Option<String> {
    // ex. "hello"
    let slice = lex.slice();
    Some(slice[1..slice.len() - 1].to_owned())
}
