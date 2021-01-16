use crate::lexer;

pub const KEYWORD_ADD: lexer::TokenValue = lexer::TokenValue::Word("Add");
pub const KEYWORD_CREATE: lexer::TokenValue = lexer::TokenValue::Word("Create");
pub const KEYWORD_REMOVE: lexer::TokenValue = lexer::TokenValue::Word("Remove");
pub const KEYWORD_SHOW: lexer::TokenValue = lexer::TokenValue::Word("Show");
pub const KEYWORDS: [lexer::TokenValue; 4] =
  [KEYWORD_ADD, KEYWORD_CREATE, KEYWORD_REMOVE, KEYWORD_SHOW];

pub const LINKER_AND: lexer::TokenValue = lexer::TokenValue::Word("and");
pub const LINKER_TO: lexer::TokenValue = lexer::TokenValue::Word("to");
pub const LINKER_FROM: lexer::TokenValue = lexer::TokenValue::Word("from");

pub const RESERVED: [lexer::TokenValue; 7] = [
  KEYWORD_ADD,
  KEYWORD_CREATE,
  KEYWORD_REMOVE,
  KEYWORD_SHOW,
  LINKER_AND,
  LINKER_TO,
  LINKER_FROM,
];

pub const SEPARATOR: lexer::TokenValue = lexer::TokenValue::Punctuation(".");
pub const SEPARATOR_OVERWRITE: lexer::TokenValue = lexer::TokenValue::Punctuation("!");
pub const SEPARATOR_FAIL_SILENTLY: lexer::TokenValue = lexer::TokenValue::Punctuation("?");
pub const SEPARATOR_VALUES: lexer::TokenValue = lexer::TokenValue::Punctuation(",");
pub const TERMINATORS: [lexer::TokenValue; 3] =
  [SEPARATOR, SEPARATOR_OVERWRITE, SEPARATOR_FAIL_SILENTLY];
