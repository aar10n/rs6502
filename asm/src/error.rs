use std::error::Error;

use ansi_term::Color;

use crate::source::Loc;
use crate::token::TokenLike;

pub struct SyntaxError {
    loc_reason: String,
    context: String,
    marker: String,
}

impl SyntaxError {
    pub fn new(location: Loc, reason: String) -> Self {
        let file = &location.file;
        let line = location.loc.line;
        let col = location.loc.column;
        let line_no_str = line.to_string();
        let line_str = file.get_source_line(line).unwrap();

        let offset = &line_str[..col - 1]
            .chars()
            .map(|c| match c {
                c if c.is_ascii_graphic() => ' ',
                _ => c,
            })
            .collect::<String>();

        let loc_reason = format!("{}: {}", location, reason);
        let context = format!("{} | {}", line_no_str, Color::White.bold().paint(line_str));
        let marker = format!(
            "{}{}{}",
            " ".repeat(line_no_str.len() + 3),
            offset,
            Color::Blue.paint("^")
        );

        Self {
            loc_reason,
            context,
            marker,
        }
    }
}

impl std::fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}\n{}", self.loc_reason, self.context, self.marker)
    }
}

impl std::fmt::Debug for SyntaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}\n{}\n{}", self.loc_reason, self.context, self.marker)
    }
}

impl Error for SyntaxError {}

//

pub fn syntax_error<'a>(loc: Loc<'a>, reason: String) -> SyntaxError {
    SyntaxError::new(loc, reason)
}

pub fn unexpected_token<'a, T>(token: &'a T, context: &str) -> SyntaxError
where
    T: TokenLike<'a>,
{
    let loc = token.source().start_loc();
    let reason = if context.len() > 0 {
        format!(
            "unexpected token '{}' in {}",
            token.source().value(),
            context,
        )
    } else {
        format!("unexpected token '{}'", token.source().value())
    };

    SyntaxError::new(loc, reason)
}

pub fn expected_delimiter<'a, 'b, T>(closing: &str, opening: &'a T, context: &str) -> SyntaxError
where
    T: TokenLike<'a>,
{
    let loc = opening.source().start_loc();
    let reason = if context.len() > 0 {
        format!(
            "expected '{}' to end opening '{}' in {}",
            closing,
            opening.source().value(),
            context
        )
    } else {
        format!(
            "expected '{}' to end opening '{}'",
            closing,
            opening.source().value()
        )
    };

    SyntaxError::new(loc, reason)
}

// macro_rules! unexpected_token {
//     ($token:expr, $context:literal) => {

//     };
// }
// pub(crate) use unexpected_token;
