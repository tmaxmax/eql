mod error;
mod operation;
pub use self::error::*;
pub use self::operation::*;

use super::lexer;

const KEYWORD_ADD: lexer::TokenValue = lexer::TokenValue::Word("Add");
const KEYWORD_CREATE: lexer::TokenValue = lexer::TokenValue::Word("Create");
const KEYWORD_REMOVE: lexer::TokenValue = lexer::TokenValue::Word("Remove");
const KEYWORD_SHOW: lexer::TokenValue = lexer::TokenValue::Word("Show");

const LINKER_AND: lexer::TokenValue = lexer::TokenValue::Word("and");
const LINKER_TO: lexer::TokenValue = lexer::TokenValue::Word("to");
const LINKER_FROM: lexer::TokenValue = lexer::TokenValue::Word("from");

const SEPARATOR: lexer::TokenValue = lexer::TokenValue::Punctuation(".");
const SEPARATOR_OVERWRITE: lexer::TokenValue = lexer::TokenValue::Punctuation("!");
const SEPARATOR_FAIL_SILENTLY: lexer::TokenValue = lexer::TokenValue::Punctuation("?");
const SEPARATOR_VALUES: lexer::TokenValue = lexer::TokenValue::Punctuation(",");
const TERMINATORS: &[lexer::TokenValue] =
  &[SEPARATOR, SEPARATOR_OVERWRITE, SEPARATOR_FAIL_SILENTLY];

fn is_operation_separator(t: lexer::TokenValue) -> bool {
  matches!(t, SEPARATOR | SEPARATOR_OVERWRITE | SEPARATOR_FAIL_SILENTLY)
}

trait TokenIterator<'a> {}

fn handle_separator<'a>(
  it: &mut (impl Iterator<Item = lexer::Token<'a>> + Clone),
  op: Operation,
  op_token: lexer::Token<'a>,
) -> Result<Operation, Error<'a>> {
  let separator = it.skip_while(|t| is_operation_separator(t.value)).next();
  if let Some(sep) = separator {
    unimplemented!()
  } else {
    Err(Error::new(op, op_token, None, Some(TERMINATORS)))
  }
}

fn parse_add<'a>(
  op_token: lexer::Token<'a>,
  tokens: &mut (impl Iterator<Item = lexer::Token<'a>> + Clone),
) -> Result<Operation, Error<'a>> {
  let mut it = tokens
    .clone()
    .take_while(|t| !is_operation_separator(t.value));
  let op = Operation::add(String::new(), false, Vec::new(), false);

  handle_separator(tokens, op, op_token)
}

pub fn parse(tokens: Vec<lexer::Token>) -> Result<Vec<Operation>, Error> {
  let mut res = Vec::new();
  let mut it = tokens.into_iter();

  while let Some(token) = it.next() {
    match token.value {
      KEYWORD_ADD => res.push(parse_add(token, &mut it)?),
      lexer::Whitespace => {}
      _ => return Err(Error::new(Operation::unknown(), token, Some(token), None)),
    }
  }

  Ok(res)
}
