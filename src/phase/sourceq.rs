use crate::ast::ComponentKind;
use crate::parse;
use crate::ast;
use crate::common::*;
use crate::virdant_error;
use super::*;

use std::collections::HashMap;
use std::sync::Arc;

#[salsa::query_group(SourceQStorage)]
pub trait SourceQ: salsa::Database {
    #[salsa::input]
    fn sources(&self) -> HashMap<String, Arc<String>>;

}
