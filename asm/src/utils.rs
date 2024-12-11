use crate::token::TokenLike;

pub fn take_one<'f, 'a, 'b, T>(tokens: &'f mut &'b [T]) -> Option<&'b T>
where
    T: TokenLike<'a> + 'a,
{
    if let Some(token) = tokens.first() {
        *tokens = &tokens[1..];
        Some(token)
    } else {
        None
    }
}

pub fn take_if<'f, 'a, 'b, T, F>(tokens: &'f mut &'b [T], pred: F) -> Option<&'b T>
where
    T: TokenLike<'a>,
    F: Fn(&T) -> bool,
{
    if let Some(token) = tokens.first() {
        if pred(token) {
            *tokens = &tokens[1..];
            return Some(token);
        }
    }
    None
}

pub fn take_while<'f, 'a, 'b, T, F>(tokens: &'f mut &'b [T], mut pred: F) -> &'b [T]
where
    T: TokenLike<'a>,
    F: FnMut(&'b T) -> bool,
{
    let mut index = 0;
    while let Some(token) = tokens[index..].first() {
        if !pred(&token) {
            break;
        }
        index += 1;
    }

    let res = &tokens[..index];
    *tokens = &tokens[index..];
    res
}
