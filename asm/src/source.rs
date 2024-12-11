use std::{borrow::Cow, collections::HashMap, fs, io, io::Read, iter::Iterator, ops, ops::Deref};

use lazy_static::lazy_static;
use logos::Logos;

use crate::token::{RawToken, RawTokenKind};

lazy_static! {
    static ref EMPTY_FILE: File = File::new("<empty>".to_string(), "".to_string());
    static ref MACRO_FILE: File = File::new("<macro>".to_string(), "".to_string());
}

pub struct SourceMap {
    /// A map between file names and source file.
    files: HashMap<String, File>,
}

impl SourceMap {
    pub fn new() -> Self {
        Self {
            files: HashMap::<String, File>::new(),
        }
    }

    pub fn add_from_string(&mut self, name: &str, source: &str) -> &File {
        let file = File::new(name.to_owned(), source.to_owned());
        self.files.insert(name.to_owned(), file);
        self.files.get(name).unwrap()
    }

    pub fn add_from_path<'a>(&'a mut self, path: &'a str) -> Result<&'a File, io::Error> {
        let mut source_file = fs::File::open(path)?;
        let mut source = String::new();
        source_file.read_to_string(&mut source)?;
        let file = File::new(path.to_owned(), source);
        self.files.insert(path.to_owned(), file);
        Ok(self.files.get(path).unwrap())
    }
}

pub struct File {
    name: String,
    source: String,
    lines: Vec<Span>,
}

impl File {
    pub fn empty() -> &'static Self {
        &EMPTY_FILE
    }

    pub fn new(name: String, source: String) -> Self {
        let lines = Self::parse_to_lines(&source);
        Self {
            name,
            source,
            lines,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn source(&self) -> &str {
        &self.source
    }

    pub fn line_count(&self) -> usize {
        self.lines.len()
    }

    pub fn get_source_line(&self, line: usize) -> Option<&str> {
        if line > 0 && line < self.lines.len() + 1 {
            let range: ops::Range<usize> = self.lines[line - 1].into();
            Some(&self.source[range])
        } else {
            None
        }
    }

    pub fn get_source_str(&self, span: Span) -> Option<&str> {
        if Span::from(0..self.source.len()).contains(&span) {
            let range: ops::Range<usize> = span.into();
            Some(&self.source[range])
        } else {
            None
        }
    }

    pub fn get_source_ref<'a>(&'a self, span: Span) -> Option<SourceRef<'a>> {
        if Span::from(0..self.source.len()).contains(&span) {
            Some(SourceRef::new(self, span))
        } else {
            None
        }
    }

    pub fn get_line_span(&self, line: usize) -> Option<Span> {
        if line < self.lines.len() {
            Some(self.lines[line])
        } else {
            None
        }
    }

    pub fn lookup_by_index<'a>(&'a self, index: usize) -> Option<Loc<'a>> {
        let line = self
            .lines
            .iter()
            .position(|&line| line.contains_pos(index))?
            + 1;

        let span = &self.lines[line - 1];
        let column = (index - span.start) + 1;
        Some(Loc {
            file: self,
            loc: LineColumn { line, column },
        })
    }

    pub fn lookup_by_span<'a>(&'a self, span: Span) -> Option<SpanLoc<'a>> {
        let begin = self.lookup_by_index(span.start)?.loc;
        let end = self.lookup_by_index(span.end)?.loc;
        Some(SpanLoc {
            begin,
            end,
            file: self,
        })
    }

    pub fn lex_tokens<'a>(&'a self) -> Vec<RawToken<'a>> {
        let lexer = RawTokenKind::lexer(&self.source);
        lexer
            .spanned()
            .into_iter()
            .map(|(kind, span)| {
                let span = Span::from(span);
                let source = self.get_source_ref(span).unwrap();
                RawToken { kind, source }
            })
            .collect()
    }

    //

    fn parse_to_lines<'a>(source: &String) -> Vec<Span> {
        let source_ptr = source.as_ptr() as usize;
        source
            .split("\n")
            .map(|l| {
                // determine offset to start of line from start of file
                let line_ptr = l.as_ptr() as usize;
                let start = line_ptr.wrapping_sub(source_ptr);
                let end = start + l.len();
                Span { start, end }
            })
            .collect()
    }
}

/// A reference to a span in a source file.
#[derive(Clone)]
pub struct SourceRef<'a> {
    /// The file that contains the reference span.
    pub file: &'a File,
    /// The span in the source file.
    pub span: Span,
    /// An *optional* originating location (such as after a macro expansion).
    pub origin: Option<&'a SourceRef<'a>>,
}

impl<'a> SourceRef<'a> {
    pub fn new(file: &'a File, span: Span) -> Self {
        Self {
            file,
            span,
            origin: None,
        }
    }

    pub fn new_from_origin(file: &'a File, span: Span, origin: &'a SourceRef<'a>) -> Self {
        Self {
            file,
            span,
            origin: Some(origin),
        }
    }

    pub fn value(&self) -> &'a str {
        self.file.get_source_str(self.span).unwrap()
    }

    pub fn span_loc(&self) -> SpanLoc<'a> {
        self.file.lookup_by_span(self.span).unwrap()
    }

    pub fn start_loc(&self) -> Loc<'a> {
        self.file.lookup_by_index(self.span.start).unwrap()
    }

    pub fn end_loc(&self) -> Loc<'a> {
        self.file.lookup_by_index(self.span.end).unwrap()
    }
}

impl std::fmt::Debug for SourceRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.span_loc())
    }
}

/// A span within a source.
#[derive(Debug, Clone, Copy, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    pub fn new(start: usize, end: usize) -> Self {
        assert!(start <= end);
        Self { start, end }
    }

    pub fn len(&self) -> usize {
        self.end - self.start
    }

    pub fn contains(&self, other: &Span) -> bool {
        self.start <= other.start && self.end >= other.end
    }

    pub fn contains_pos(&self, pos: usize) -> bool {
        self.start <= pos && self.end >= pos
    }

    pub fn subspan(&self, range: ops::Range<usize>) -> Span {
        assert!(range.start <= range.end);
        assert!(range.start + range.end <= self.len());
        let start = self.start + range.start;
        let end = start + range.end;
        Self { start, end }
    }
}

impl Into<ops::Range<usize>> for Span {
    fn into(self) -> ops::Range<usize> {
        ops::Range {
            start: self.start,
            end: self.end,
        }
    }
}

impl<T> From<ops::Range<T>> for Span
where
    T: Into<usize>,
{
    fn from(range: ops::Range<T>) -> Self {
        Self {
            start: range.start.into(),
            end: range.end.into(),
        }
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}..{}", self.start, self.end)
    }
}

/// A file, and line/column location within it.
#[derive(Clone)]
pub struct Loc<'a> {
    pub file: &'a File,
    pub loc: LineColumn,
}

impl std::fmt::Display for Loc<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.file.name(), self.loc)
    }
}

impl std::fmt::Debug for Loc<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}: {}", self.file.name(), self.loc)
    }
}

/// A file, and line/column range within it.
#[derive(Clone)]
pub struct SpanLoc<'a> {
    pub file: &'a File,
    pub begin: LineColumn,
    pub end: LineColumn,
}

impl std::fmt::Display for SpanLoc<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.begin == self.end {
            write!(f, "{}: {}", self.file.name(), self.begin)
        } else {
            write!(
                f,
                "{}:{} {}-{}",
                self.file.name(),
                self.begin.line,
                self.begin.column,
                self.end.column
            )
        }
    }
}

/// A line and column
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LineColumn {
    pub line: usize,
    pub column: usize,
}

impl std::fmt::Display for LineColumn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

//
//
//

/// A copy-on-write string.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct StrRef<'a>(Cow<'a, str>);

impl<'a> Deref for StrRef<'a> {
    type Target = Cow<'a, str>;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, S> From<S> for StrRef<'a>
where
    S: Into<&'a str>,
{
    fn from(string: S) -> Self {
        Self {
            0: Cow::Borrowed(string.into()),
        }
    }
}

impl<'a> PartialEq<str> for StrRef<'a> {
    fn eq(&self, other: &str) -> bool {
        self.deref() == other
    }
}

impl std::fmt::Display for StrRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.deref())
    }
}

impl std::fmt::Debug for StrRef<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let string = self.deref();
        let escaped = string
            .chars()
            .fold(String::with_capacity(string.len()), |acc, c| match c {
                '\n' => acc + "\\n",
                '\t' => acc + "\\t",
                ' ' => acc + "⎕",
                _ => format!("{}{}", acc, c),
            });
        write!(f, "{}", escaped)
    }
}

//

pub struct SliceView<'a, T> {
    slice: &'a [T],
}

impl<'a, T> SliceView<'a, T> {
    pub fn peek(&self, n: usize) -> Option<&'a T> {
        self.slice.get(n)
    }

    pub fn first_if<F>(&self, pred: F) -> Option<&'a T>
    where
        F: FnOnce(&T) -> bool,
    {
        let value = self.slice.first()?;
        if pred(value) {
            Some(value)
        } else {
            None
        }
    }

    pub fn drop_first<'b>(&'b mut self) -> Option<&'a T> {
        let item = self.slice.first()?;
        self.slice = &self.slice[1..];
        Some(item)
    }

    pub fn drop_first_if<'b, F>(&'b mut self, pred: F) -> Option<&'a T>
    where
        F: FnOnce(&T) -> bool,
    {
        let item = self.slice.first()?;
        if pred(item) {
            self.slice = &self.slice[1..];
            Some(item)
        } else {
            None
        }
    }

    pub fn drop_while<'b, F>(&'b mut self, pred: F) -> &'b [T]
    where
        F: Fn(&T) -> bool,
    {
        let slice = self.slice;
        let mut index = 0;
        while let Some(value) = self.first() {
            if !pred(value) {
                break;
            }

            self.drop_first();
            index += 1;
        }

        &slice[0..index + 1]
    }
}

impl<'a, T> Deref for SliceView<'a, T> {
    type Target = &'a [T];
    fn deref(&self) -> &Self::Target {
        &self.slice
    }
}

impl<'a, T> From<&'a [T]> for SliceView<'a, T> {
    fn from(slice: &'a [T]) -> Self {
        Self { slice }
    }
}

impl<'a, T> Into<&'a [T]> for SliceView<'a, T> {
    fn into(self) -> &'a [T] {
        self.slice
    }
}
