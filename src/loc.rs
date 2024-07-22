use std::sync::Arc;

/// A [`SourceInfo`] maintains location data for parsed objects.
/// Maintains the filename (if from a file) or the originating string (if from a string).
/// Helps with the conversion from byte-position in the source to a [`Pos`].
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct SourceInfo {
    source: Source,
    linelens: LineLens,
}

/// A [`Pos`] is a container for a line and column.
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct Pos(usize, usize);

/// A [`Span`] tracks the span of an object parsed from a source.
#[derive(Clone, PartialEq, Eq, Hash)]
pub struct Span {
    start_idx: usize,
    end_idx: usize,
    source_info: SourceInfo,
}

impl SourceInfo {
    pub fn unknown() -> SourceInfo {
        SourceInfo {
            source: Source::Unknown,
            linelens: LineLens::from(""),
        }
    }

    pub fn source(&self) -> &Source {
        &self.source
    }

    pub fn from_file(filepath: &std::path::Path, contents: &str) -> SourceInfo {
        SourceInfo {
            source: Source::File(Arc::new(filepath.to_owned())),
            linelens: LineLens::from(contents),
        }
    }

    pub fn from_string(contents: &str) -> SourceInfo {
        SourceInfo {
            source: Source::String(Arc::new(contents.to_owned())),
            linelens: LineLens::from(contents),
        }
    }

    pub fn start(&self, item: &dyn HasSpan) -> Pos {
        self.linelens.linecol(item.span().start_idx)
    }

    pub fn end(&self, item: &dyn HasSpan) -> Pos {
        self.linelens.linecol(item.span().end_idx)
    }

    pub fn linecol_from(&self, pos: usize) -> Pos {
        self.linelens.linecol(pos)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Source {
    File(Arc<std::path::PathBuf>),
    String(Arc<String>),
    Unknown,
}

impl Pos {
    pub fn from(line: usize, col: usize) -> Pos {
        Pos(line, col)
    }

    /// The line number. Starts with line 0.
    pub fn line(&self) -> usize {
        self.0
    }

    /// The column. Starts with column 0.
    pub fn col(&self) -> usize {
        self.1
    }
}

impl std::fmt::Display for Pos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{}:{}", self.line() + 1, self.col() + 1)
    }
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match &self.source_info.source {
            Source::File(path) => write!(f, "[{}-{}:{:?}]", self.start(), self.end(), path),
            Source::String(s) => write!(f, "[{}-{}:{:?}]", self.start(), self.end(), String::from_utf8_lossy(&s.as_bytes()[self.start_idx..self.end_idx])),
            Source::Unknown => write!(f, "[{}-{}]", self.start(), self.end()),
        }
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        match &self.source_info.source {
            Source::File(_path) => write!(f, "[{}-{}]", self.start(), self.end()),
            Source::String(_s) => write!(f, "[{}-{}]", self.start(), self.end()),
            Source::Unknown => write!(f, "[{}-{}]", self.start(), self.end()),
        }
    }
}

impl Span {
    /// When the location of something is unknown, you can use this.
    pub fn unknown() -> Span {
        Span {
            start_idx: 0,
            end_idx: 0,
            source_info: SourceInfo::unknown(),
        }
    }

    pub fn from(source_info: &SourceInfo, start: usize, end: usize) -> Span {
        Span {
            start_idx: start,
            end_idx: end,
            source_info: source_info.clone(),
        }
    }

    /// The start of the span.
    pub fn start(&self) -> Pos {
        self.source_info.linelens.linecol(self.start_idx)
    }

    /// The end of the span.
    pub fn end(&self) -> Pos {
        self.source_info.linelens.linecol(self.end_idx)
    }

    pub fn source(&self) -> &str {
        if let Source::String(source) = &self.source_info.source {
            &source[self.start_idx..self.end_idx]
        } else {
            ""
        }
    }

    pub fn contains(&self, linecol: &Pos) -> bool {
        &self.start() <= linecol && linecol <= &self.end()
    }
}

/// Many objects have location information.
/// [`HasSpan`] allows you to call [`HasLoc::loc`] to get the span information.
pub trait HasSpan {
    fn span(&self) -> Span;
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
struct LineLens(Vec<usize>);

impl LineLens {
    fn from(text: &str) -> LineLens {
        let mut lens = vec![];
        for line in text.split("\n") {
            lens.push(line.len() + 1);
        }
        LineLens(lens)
    }

    fn linecol(&self, pos: usize) -> Pos {
        let mut line = 0;
        let mut col = pos;
        for line_len in &self.0 {
            if col >= *line_len {
                col -= *line_len;
                line += 1;
            } else {
                break
            }
        }
        Pos(line, col)
    }
}

#[test]
fn linelens() {
    // TODO Move this to tests.
    let text = "Hello,
world!
How are you?";

    let linelens = LineLens::from(text);
    assert_eq!(linelens.linecol(0).to_string(), "1:1".to_string());
    assert_eq!(linelens.linecol(5).to_string(), "1:6".to_string());
    assert_eq!(linelens.linecol(6).to_string(), "1:7".to_string());
    assert_eq!(linelens.linecol(7).to_string(), "2:1".to_string());
    assert_eq!(linelens.linecol(7).to_string(), "2:1".to_string());
    assert_eq!(linelens.linecol(12).to_string(), "2:6".to_string());
    assert_eq!(linelens.linecol(13).to_string(), "2:7".to_string());
    assert_eq!(linelens.linecol(14).to_string(), "3:1".to_string());
}
