use super::*;

use std::collections::HashMap;
use std::sync::Arc;

#[salsa::query_group(SourceQStorage)]
pub trait SourceQ: salsa::Database {
    #[salsa::input]
    fn sources(&self) -> HashMap<String, Arc<String>>;

    fn linelens(&self, package_id: PackageId) -> LineLens;
    fn pos(&self, pos: PosIdx) -> Pos;
    fn span(&self, span: SpanIdx) -> Span;
}

fn pos(db: &dyn SourceQ, pos: PosIdx) -> Pos {
    let linelens = db.linelens(pos.package());
    linelens.pos(pos)
}

fn span(db: &dyn SourceQ, span: SpanIdx) -> Span {
    let start = db.pos(span.start());
    let end = db.pos(span.end());
    Span(start, end)
}

#[derive(Eq, PartialEq, Clone, Hash)]
pub struct Span(Pos, Pos);

impl Span {
    pub fn package(&self) -> PackageId {
        self.0.package()
    }

    pub fn start(&self) -> Pos {
        self.0.clone()
    }

    pub fn end(&self) -> Pos {
        self.1.clone()
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Debug)]
pub struct Pos(PackageId, usize, usize);

impl Pos {
    pub fn package(&self) -> PackageId {
        self.0.clone()
    }

    pub fn line(&self) -> usize {
        self.1
    }

    pub fn col(&self) -> usize {
        self.2
    }
}

fn linelens(db: &dyn SourceQ, package_id: PackageId) -> LineLens {
    LineLens::new(db.sources().get(&package_id.to_string()).unwrap())
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct LineLens(Vec<usize>);

impl LineLens {
    fn new(text: &str) -> LineLens {
        let mut lens = vec![];
        for line in text.split("\n") {
            lens.push(line.len() + 1);
        }
        LineLens(lens)
    }

    fn pos(&self, pos: PosIdx) -> Pos {
        let mut line = 0;
        let mut col = pos.0;
        for line_len in &self.0 {
            if col >= *line_len {
                col -= *line_len;
                line += 1;
            } else {
                break
            }
        }
        Pos(pos.package(), line, col)
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct PosIdx(usize, PackageId);

impl PosIdx {
    pub fn package(&self) -> PackageId {
        self.1.clone()
    }
}


#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct SpanIdx(usize, usize, PackageId);

impl SpanIdx {
    pub fn new(package_id: PackageId, start_idx: usize, end_idx: usize) -> SpanIdx {
        SpanIdx(start_idx, end_idx, package_id)
    }

    pub fn start(&self) -> PosIdx {
        PosIdx(self.0, self.2.clone())
    }

    pub fn end(&self) -> PosIdx {
        PosIdx(self.1, self.2.clone())
    }

    pub fn package(&self) -> PackageId {
        self.2.clone()
    }
}

impl PosIdx {
    pub fn new(package_id: PackageId, idx: usize) -> PosIdx {
        PosIdx(idx, package_id)
    }
}

impl std::fmt::Display for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        let package = self.package().name();
        let start = self.start();
        let _end = self.end();
        write!(f, "[{}:{}:{:?}]", start.line() + 1, start.col() + 1, package)
    }
}

impl std::fmt::Debug for Span {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        write!(f, "{self}")
    }
}
