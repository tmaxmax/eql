#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Operation {
  Create {
    department: String,
    overwrite: bool,
    fail_silently: bool,
  },
  Remove {
    department: String,
    fail_silently: bool,
  },
  Add {
    department: String,
    names: Vec<String>,
    overwrite: bool,
    fail_silently: bool,
  },
  Show {
    department: String,
    fail_silently: bool,
  },
}

use super::lexer::{Token, TokenValue};

const KEYWORD_ADD: TokenValue = TokenValue::Word("Add");
const KEYWORD_CREATE: TokenValue = TokenValue::Word("Create");
const KEYWORD_REMOVE: TokenValue = TokenValue::Word("Remove");
const KEYWORD_SHOW: TokenValue = TokenValue::Word("Show");

const SEPARATOR: TokenValue = TokenValue::Punctuation(".");
const SEPARATOR_OVERRIDE: TokenValue = TokenValue::Punctuation("!");
const SEPARATOR_FAIL_SILENTLY: TokenValue = TokenValue::Punctuation("?");
const SEPARATOR_VALUES: TokenValue = TokenValue::Punctuation(",");

pub fn parse(tokens: &[Token]) -> Result<Vec<Operation>, ()> {
  unimplemented!();
}
