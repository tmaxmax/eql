use crate::lexer::*;

pub const KEYWORD_ADD: TokenValue = Word("Add");
pub const KEYWORD_CREATE: TokenValue = Word("Create");
pub const KEYWORD_REMOVE: TokenValue = Word("Remove");
pub const KEYWORD_SHOW: TokenValue = Word("Show");
pub const KEYWORDS: [TokenValue; 4] = [KEYWORD_ADD, KEYWORD_CREATE, KEYWORD_REMOVE, KEYWORD_SHOW];

pub const LINKER_AND: TokenValue = Word("and");
pub const LINKER_TO: TokenValue = Word("to");
pub const LINKER_FROM: TokenValue = Word("from");

pub const RESERVED: [TokenValue; 7] = [
  KEYWORD_ADD,
  KEYWORD_CREATE,
  KEYWORD_REMOVE,
  KEYWORD_SHOW,
  LINKER_AND,
  LINKER_TO,
  LINKER_FROM,
];

pub const SEPARATOR: TokenValue = Punctuation(".");
pub const SEPARATOR_OVERWRITE: TokenValue = Punctuation("!");
pub const SEPARATOR_FAIL_SILENTLY: TokenValue = Punctuation("?");
pub const SEPARATOR_VALUES: TokenValue = Punctuation(",");
pub const TERMINATORS: [TokenValue; 3] = [SEPARATOR, SEPARATOR_OVERWRITE, SEPARATOR_FAIL_SILENTLY];
