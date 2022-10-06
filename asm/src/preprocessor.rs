use std::{collections::HashMap, rc::Rc};

use crate::{
    error,
    error::SyntaxError,
    source::SourceRef,
    token::{RawToken, RawTokenKind, TokenLike},
};

const RECURSION_LIMIT: usize = 10;

pub struct Macro<'a> {
    pub name: &'a str,
    pub params: Option<Vec<&'a str>>,
    pub def: Vec<MacroToken<'a>>,
}

impl<'a> Macro<'a> {
    pub fn new(name: &'a str, params: Option<Vec<&'a str>>, def: Vec<MacroToken<'a>>) -> Self {
        Self { name, params, def }
    }

    pub fn new_constant(name: &'a str, def: Vec<&'a RawToken<'a>>) -> Self {
        let params = None;
        let def = def
            .into_iter()
            .map(|t| MacroToken::Token(t))
            .collect::<Vec<_>>();

        Macro { name, params, def }
    }

    pub fn new_function(name: &'a str, params: Vec<&'a str>, def: Vec<&'a RawToken<'a>>) -> Self {
        let def = def
            .iter()
            .map(|t| {
                if params.contains(&t.source.value()) {
                    MacroToken::Parameter(t)
                } else {
                    MacroToken::Token(t)
                }
            })
            .collect::<Vec<_>>();
        let params = Some(params);

        Macro { name, params, def }
    }
}

pub enum MacroToken<'a> {
    Parameter(&'a RawToken<'a>),
    Token(&'a RawToken<'a>),
}

impl std::fmt::Debug for MacroToken<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MacroToken::Token(t) => write!(f, "{:?}", t),
            MacroToken::Parameter(t) => write!(f, "{:?}", t),
        }
    }
}

/// A structure which holds all definitions for a single macro name.
///
/// It stores a different [`Macro`] definition for each unique parameter count. This
/// enables macro functions to be overloaded by argument count.
/// ```text
///     %define test
///     %define test(a)
///     %define test(a, b)
///     %define test(a, b, c)
///
///     test
///     test(1)
///     test(2, 3)
///     test(4, 5, 6)
/// ```
pub struct MacroSet<'a> {
    pub name: &'a str,
    constant: Option<Vec<MacroToken<'a>>>,
    overloads: Vec<(Vec<&'a str>, Vec<MacroToken<'a>>)>,
}

impl<'a> MacroSet<'a> {
    /// Returns a new empty [`MacroSet`] for `name`.
    pub fn new(name: &'a str) -> Self {
        Self {
            name,
            constant: None,
            overloads: vec![],
        }
    }

    /// Adds a new macro definition to the set.
    ///
    /// If any definition already exists with the same number of parameters it is
    /// replaced. This method will panic if the name of `macro` does not match the
    /// name of this `MacroSet`.
    pub fn add(&mut self, params: Option<Vec<&'a str>>, def: Vec<MacroToken<'a>>) {
        if let Some(p) = params {
            match self
                .overloads
                .binary_search_by_key(&p.len(), |(a, _)| a.len())
            {
                // existing definition exists, replace it
                Ok(index) => self.overloads[index] = (p, def),
                // add completely new definition
                Err(index) => self.overloads.insert(index, (p, def)),
            }
        } else {
            self.constant = Some(def);
        }
    }

    /// Returns whether the set has a macro constant definition.
    pub fn has_constant(&self) -> bool {
        self.constant.is_some()
    }

    /// Returns whether the set has any macro function overloads.
    pub fn has_overloads(&self) -> bool {
        !self.overloads.is_empty()
    }

    /// Returns the constant definition for this macro should it exist.
    pub fn get_constant<'b>(&'b self) -> Option<&'b Vec<MacroToken<'a>>> {
        self.constant.as_ref()
    }

    /// Returns the definition for the given overload form if it exists.
    pub fn get_overload<'b>(&'b self, args: usize) -> Option<&(Vec<&'a str>, Vec<MacroToken<'a>>)> {
        self.overloads
            .iter()
            .position(|(c, _)| c.len() == args)
            .map(|i| (&self.overloads[i]))
    }
}

impl std::fmt::Debug for MacroSet<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let constant = match &self.constant {
            Some(def) => format!("  {:?}\n", def),
            None => format!(""),
        };
        let overloads = self
            .overloads
            .iter()
            .map(|(params, def)| format!("  ({}) => {:?}\n", params.join(", "), def))
            .collect::<String>();

        write!(f, "{}\n{}{}", self.name, constant, overloads)
    }
}

/// A structure which holds macro definitions.
///
/// This is a simple convinience wrapper around a [HashMap] that provides helpful
/// functions for inserting and retrieveing macro definitions.
pub struct MacroTable<'a>(HashMap<&'a str, MacroSet<'a>>);

impl<'a> MacroTable<'a> {
    /// Returns a new empty `MacroTable`.
    pub fn new() -> Self {
        Self(HashMap::<&'a str, MacroSet<'a>>::new())
    }

    /// Returns the [`MacroSet`] for `name` if it exists.
    pub fn get<'b>(&'b self, name: &str) -> Option<&'b MacroSet<'a>> {
        self.0.get(name)
    }

    /// Returns whether `name` has a corresponding [`MacroSet`] in the map.
    pub fn has_name(&self, name: &str) -> bool {
        self.0.contains_key(name)
    }

    /// Adds the given macro definition to the existing [`MacroSet`] or inserts a new one.
    pub fn add_macro(&mut self, def: Macro<'a>) {
        self.0
            .entry(def.name)
            .or_insert(MacroSet::new(def.name))
            .add(def.params, def.def)
    }
}

//
//
//

pub fn preprocess<'source>(
    tokens: &'source [RawToken<'source>],
    predefs: Vec<Macro<'source>>,
) -> Result<Vec<RawToken<'source>>, SyntaxError> {
    if tokens.is_empty() {
        return Ok(vec![]);
    }

    let mut tokens = &tokens[..];
    let mut defs = MacroTable::new();
    for def in predefs {
        defs.add_macro(def);
    }

    preprocess_tokens(&mut tokens, &mut defs)
}

fn preprocess_tokens<'f, 'a>(
    tokens: &'f mut &'a [RawToken<'a>],
    defs: &'f mut MacroTable<'a>,
) -> Result<Vec<RawToken<'a>>, SyntaxError> {
    if tokens.is_empty() {
        return Ok(vec![]);
    }

    let mut out_tokens = Vec::<RawToken<'a>>::with_capacity(tokens.len());
    while let Some(token) = take_one(tokens) {
        let kind = &token.kind;
        let range = &token.source;
        match kind {
            RawTokenKind::PreProcessor => {
                // drop the leading '%'
                let directive = &range.value()[1..];
                match directive {
                    // defines a macro
                    "define" => {
                        if let Some(def) = preprocess_define(tokens)? {
                            // TODO: check if macro is defined with and without parameters
                            defs.add_macro(def);
                        }
                        continue;
                    }
                    _ => {}
                }
            }
            RawTokenKind::Identifier => {
                let value = range.value();
                if defs.has_name(value) {
                    let expanded = expand_macro(token, tokens, defs)?;
                    out_tokens.extend(expanded.into_iter());
                } else {
                    out_tokens.push(token.clone())
                }
            }
            RawTokenKind::Comment => {
                // ignore
            }
            _ => out_tokens.push(token.clone()),
        }
    }

    Ok(out_tokens)
}

/// Parses a preprocessor macro definition.
///
/// A macro definition can either be a constant or function. All macro forms terminate
/// at the first newline encountered.
///
/// ### Macro Constants
/// ```text
///     %define
///     %define name
///     %define name 1234
///     %define name (1234)
/// ```
///
/// ### Macro Functions
/// ```text
///     %define name()
///     %define name(a)
///     %define name(a, b) (a + b)
/// ```
/// *note* - If a space follows the macro name it will be interpreted as a constant.
fn preprocess_define<'f, 'a>(
    tokens: &'f mut &'a [RawToken<'a>],
) -> Result<Option<Macro<'a>>, SyntaxError> {
    skip_whitespace(tokens);

    if tokens.first().map(is_eol).unwrap_or(true) {
        // empty '%define', consume optional newline and return
        skip_eol(tokens);
        return Ok(None);
    }

    let name = take_one(tokens).unwrap();
    if !name.is_identifier() {
        let loc = name.source.start_loc();
        let reason = format!("expected macro name");
        return Err(SyntaxError::new(loc, reason));
    }

    let name = name.source.value();
    if let Some(token) = tokens.first() {
        if is_eol(token) {
            // empty macro
            return Ok(Some(Macro::new(name, None, vec![])));
        } else if token.is_whitespace() {
            // macro constant
            return preprocess_define_const(name, tokens).map(|r| Some(r));
        } else if token.is_lparen() {
            // macro function
            return preprocess_define_func(name, tokens).map(|r| Some(r));
        } else {
            // unexpected token
            let loc = token.source.start_loc();
            let reason = format!("unexpected token");
            return Err(SyntaxError::new(loc, reason));
        }
    }

    Ok(Some(Macro::new(name, None, vec![])))
}

/// Parses a preprocessor macro constant definition.
fn preprocess_define_const<'f, 'a>(
    name: &'a str,
    tokens: &'f mut &'a [RawToken<'a>],
) -> Result<Macro<'a>, SyntaxError> {
    skip_whitespace(tokens);

    // parse definition
    let def = take_while(tokens, is_not_eol);
    skip_eol(tokens);

    let def = def.iter().map(|t| MacroToken::Token(t)).collect::<Vec<_>>();
    Ok(Macro::new(name, None, def))
}

/// Parses a preprocessor macro function definition.
fn preprocess_define_func<'f, 'a>(
    name: &'a str,
    tokens: &'f mut &'a [RawToken<'a>],
) -> Result<Macro<'a>, SyntaxError> {
    // skip the '(' token
    let lparen = take_one(tokens).unwrap();
    assert!(lparen.is_lparen());

    // parse params
    skip_whitespace(tokens);
    let mut params = Vec::<&str>::new();
    'outer: while let Some(param) = take_if(tokens, is_not_eol) {
        if param.is_rparen() {
            break;
        } else if !param.is_identifier() {
            let err = error::unexpected_token(param, "macro parameter list");
            return Err(err);
        }

        // replace or add the parameter name
        let param_name = param.source.value();
        if let Some(index) = params.iter().position(|p| *p == param_name) {
            params.remove(index);
        }
        params.push(param_name);

        skip_whitespace(tokens);
        match take_if(tokens, is_not_eol) {
            Some(next) => {
                if next.is_rparen() {
                    break 'outer;
                } else if next.is_comma() {
                    skip_whitespace(tokens);
                    continue 'outer;
                }

                let err = error::unexpected_token(next, "macro parameter list");
                return Err(err);
            }
            None => {
                let err = error::expected_delimiter(")", lparen, "macro parameter list");
                return Err(err);
            }
        }
    }

    // parse definition
    skip_whitespace(tokens);
    let def = take_while(tokens, is_not_eol);
    take_one(tokens); // consume ending newline

    let def = def
        .iter()
        .map(|t| {
            if params.contains(&t.source.value()) {
                MacroToken::Parameter(t)
            } else {
                MacroToken::Token(t)
            }
        })
        .collect::<Vec<_>>();

    Ok(Macro::new(name, Some(params), def))
}

/// Fully expands a preprocessor macro into its final replacement.
///
/// This function handles expansion of both macro constants and functions. If the name
/// refers to a macro function, this will select the correct overload to use based on
/// the number of arguments provided. If no such expansion exists, it returns an error.
///
/// This function recursively expands until it cannot be expanded further.
fn expand_macro<'f, 'a, 'b>(
    token: &'a RawToken<'a>,
    tokens: &'f mut &'b [RawToken<'a>],
    defs: &'f MacroTable<'a>,
) -> Result<Vec<RawToken<'a>>, SyntaxError> {
    assert!(token.is_identifier());
    let name = token.source.value().to_owned();
    let macroset = defs.get(&name).unwrap();

    let expanded = expand_macro_once(token, tokens, macroset)?;
    if expanded.is_none() {
        return Ok(vec![token.clone()]);
    }

    let mut out_tokens = Vec::<RawToken<'a>>::new();
    let mut working = vec![Rc::new(expanded.unwrap())];
    'outer: while let Some(temp) = working.last().map(|v| Rc::clone(v)) {
        if working.len() > RECURSION_LIMIT {
            let loc = token.source.start_loc();
            println!("loc -> {:?}", loc);
            let reason = format!(
                "recursion limit reached during expansion of macro '{}'",
                name
            );
            let err = error::syntax_error(loc, reason);
            return Err(err);
        }

        let tokens = &mut &temp[..];
        while let Some(token) = take_one(tokens) {
            let value = token.source.value();
            if token.is_identifier() && defs.has_name(value) {
                let macroset = defs.get(value).unwrap();
                if let Some(expanded) = expand_macro_once(token, tokens, macroset)? {
                    let index = working.len() - 1;
                    working[index] = Rc::new(tokens.to_vec());
                    working.push(Rc::new(expanded));
                    continue 'outer;
                }
            }

            out_tokens.push(token.clone());
        }

        working.pop();
    }

    Ok(out_tokens)
}

/// Expands a preprocessor macro once.
fn expand_macro_once<'f, 'a, 'b>(
    token: &'b RawToken<'a>,
    tokens: &'f mut &'b [RawToken<'a>],
    defs: &'f MacroSet<'a>,
) -> Result<Option<Vec<RawToken<'a>>>, SyntaxError> {
    assert!(token.is_identifier());

    // check to see if this could be a macro function
    if matches!(tokens.first(), Some(t) if t.is_lparen()) && defs.has_overloads() {
        // skip the '(' token
        let lparen = take_one(tokens).unwrap();

        // this might be a function call
        let args = collect_macro_args(lparen, tokens)?;
        if let Some((params, def)) = defs.get_overload(args.len()) {
            Ok(Some(expand_macro_func(token, args, params, def)))
        } else {
            // no matching overload
            // TODO: print warning?
            panic!("invalid macro call")
        }
    } else if let Some(def) = defs.get_constant() {
        Ok(Some(expand_macro_const(token, def)))
    } else {
        Ok(None)
    }
}

/// Expands a constant macro definition.
fn expand_macro_const<'f, 'a, 'b>(
    token: &'b RawToken<'a>,
    def: &'b Vec<MacroToken<'a>>,
) -> Vec<RawToken<'a>> {
    assert!(token.is_identifier());

    let tokens = def
        .iter()
        .map(|t| match t {
            MacroToken::Parameter(_) => panic!("unexpected parameter in macro constant"),
            MacroToken::Token(t) => {
                let kind = t.kind.clone();
                let source = SourceRef::new(t.file(), t.source.span);
                RawToken { kind, source }
            }
        })
        .collect::<Vec<_>>();

    tokens
}

/// Expands a function macro definition.
fn expand_macro_func<'f, 'a, 'b>(
    token: &'b RawToken<'a>,
    args: Vec<&'b [RawToken<'a>]>,
    params: &'f Vec<&'a str>,
    def: &'b Vec<MacroToken<'a>>,
) -> Vec<RawToken<'a>> {
    assert!(token.is_identifier());
    assert!(args.len() == params.len());

    let mut tokens = Vec::<RawToken<'a>>::with_capacity(def.len());
    for t in def {
        match t {
            MacroToken::Parameter(def_tok) => {
                let index = params
                    .iter()
                    .position(|p| *p == def_tok.source.value())
                    .unwrap();

                for arg_tok in args[index] {
                    let kind = arg_tok.kind.clone();
                    let file = arg_tok.file();
                    let source = SourceRef::new(file, arg_tok.source.span);
                    tokens.push(RawToken { kind, source })
                }
            }
            MacroToken::Token(def_tok) => {
                let kind = def_tok.kind.clone();
                let file = def_tok.file();
                let source = SourceRef::new(file, def_tok.source.span);
                tokens.push(RawToken { kind, source })
            }
        }
    }

    tokens
}

/// Collects the arguments that will be used in a macro expansion.
fn collect_macro_args<'f, 'a, 'b>(
    lparen: &'b RawToken<'a>,
    tokens: &'f mut &'b [RawToken<'a>],
) -> Result<Vec<&'b [RawToken<'a>]>, SyntaxError> {
    assert!(lparen.is_lparen());

    let mut args = Vec::<&[RawToken]>::new();
    loop {
        let arg = take_macro_arg(tokens)?;
        args.push(arg);

        if let Some(next) = take_one(tokens) {
            if next.is_comma() {
                // keep collecting arguments
                continue;
            } else if next.is_rparen() {
                // final outer ')' to end macro call
                break;
            } else if is_eol(next) {
                let err = error::expected_delimiter(")", lparen, "macro call");
                return Err(err);
            } else {
                let err = error::unexpected_token(next, "macro call");
                return Err(err);
            }
        }
    }

    Ok(args)
}

fn take_macro_arg<'f, 'a, 'b>(
    tokens: &'f mut &'b [RawToken<'a>],
) -> Result<&'b [RawToken<'a>], SyntaxError> {
    skip_whitespace(tokens);

    let mut parens = Vec::<&RawToken>::new();
    let arg = take_while(tokens, |t| {
        if t.is_lparen() {
            parens.push(t);
        } else if t.is_rparen() {
            if parens.pop().is_none() {
                // outer unmatched ')'
                return false;
            }
            return true;
        }

        is_not_eol(t) && !t.is_comma()
    });

    if let Some(lparen) = parens.last() {
        let err = error::expected_delimiter(")", *lparen, "macro arg");
        return Err(err);
    }
    Ok(arg)
}

//
//
//

fn is_eol<'f, 'a>(token: &'f RawToken<'a>) -> bool {
    token.is_comment() || token.is_newline()
}

fn is_not_eol<'f, 'a>(token: &'f RawToken<'a>) -> bool {
    !(token.is_comment() || token.is_newline())
}

fn take_one<'f, 'a, 'b>(tokens: &'f mut &'b [RawToken<'a>]) -> Option<&'b RawToken<'a>> {
    if let Some(token) = tokens.first() {
        *tokens = &tokens[1..];
        Some(token)
    } else {
        None
    }
}

fn take_if<'f, 'a, 'b, F>(tokens: &'f mut &'b [RawToken<'a>], pred: F) -> Option<&'b RawToken<'a>>
where
    F: Fn(&RawToken<'a>) -> bool,
{
    if let Some(token) = tokens.first() {
        if pred(token) {
            *tokens = &tokens[1..];
            return Some(token);
        }
    }
    None
}

fn take_while<'f, 'a, 'b, F>(tokens: &'f mut &'b [RawToken<'a>], mut pred: F) -> &'b [RawToken<'a>]
where
    F: FnMut(&'b RawToken<'a>) -> bool,
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

fn skip_whitespace<'f, 'a, 'b>(rest: &'f mut &'b [RawToken<'a>]) -> Option<&'b RawToken<'a>> {
    let mut prev = rest.first();
    while let Some(token) = rest.first() {
        if !token.is_whitespace() {
            break;
        }
        prev = Some(&rest[0]);
        *rest = &rest[1..];
    }
    prev
}

fn skip_eol<'f, 'a>(tokens: &'f mut &'a [RawToken]) {
    if let Some(_) = take_if(tokens, |t| t.is_comment()) {
        take_if(tokens, |t| t.is_newline());
    } else {
        take_if(tokens, |t| t.is_newline());
    }
}
