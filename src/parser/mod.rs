mod error;
mod operation;
pub use self::error::*;
pub use self::operation::*;

use super::lexer;
use std::borrow::Cow;

const KEYWORD_ADD: lexer::TokenValue = lexer::TokenValue::Word("Add");
const KEYWORD_CREATE: lexer::TokenValue = lexer::TokenValue::Word("Create");
const KEYWORD_REMOVE: lexer::TokenValue = lexer::TokenValue::Word("Remove");
const KEYWORD_SHOW: lexer::TokenValue = lexer::TokenValue::Word("Show");

const LINKER_AND: lexer::TokenValue = lexer::TokenValue::Word("and");
const LINKER_TO: lexer::TokenValue = lexer::TokenValue::Word("to");
const LINKER_FROM: lexer::TokenValue = lexer::TokenValue::Word("from");

const RESERVED: [lexer::TokenValue; 7] = [
  KEYWORD_ADD,
  KEYWORD_CREATE,
  KEYWORD_REMOVE,
  KEYWORD_SHOW,
  LINKER_AND,
  LINKER_TO,
  LINKER_FROM,
];

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
    Err(Error::new(op, op_token, None, Some(TERMINATORS), None))
  }
}

fn parse_names<'a>(
  tokens: &mut impl Iterator<Item = lexer::Token<'a>>,
) -> Result<Vec<String>, Error> {
  unimplemented!();
}

fn push_token<'a>(v: &mut Vec<lexer::TokenValue<'a>>, token: &lexer::Token<'a>) -> bool {
  match token.value {
    lexer::Word(_) => {
      v.push(token.value);
      true
    }
    lexer::Whitespace => {
      if let Some(last) = v.last() {
        match last {
          lexer::Word(_) => v.push(token.value),
          _ => {}
        }
      }
      true
    }
    _ => false,
  }
}

fn parse_add<'a>(
  op_token: lexer::Token<'a>,
  tokens: &mut (impl Iterator<Item = lexer::Token<'a>> + Clone),
) -> Result<Operation, Error<'a>> {
  const ALLOWED_TOKENS: &[lexer::TokenValue] = &[lexer::Word(""), lexer::Whitespace];

  let mut it = tokens
    .clone()
    .take_while(|t| !is_operation_separator(t.value));
  let op = Operation::add(String::new(), false, Vec::new(), false);
  let make_error = |token: lexer::Token<'a>, details: Option<Cow<'static, str>>| {
    Error::new(op, op_token, Some(token), Some(ALLOWED_TOKENS), details)
  };

  let mut names: Vec<String> = Vec::new();
  let mut name = Vec::new();
  for token in it {
    if push_token(&mut name, &token) {
      continue;
    }
    match token.value {
      SEPARATOR_VALUES | LINKER_AND => {
        if name.is_empty() {
          return Err(make_error(
            token,
            Some(format!("You didn't write any name before {}", token.value).into()),
          ));
        }
        names.push(name.into_iter().map(|t| t.get()).collect());
      }
      LINKER_TO => {
        break;
      }
      _ if RESERVED.contains(&token.value) => {
        return Err(make_error(
          token,
          Some(format!("{} is reserved", token.value).into()),
        ))
      }
    }
  }

  let mut department = Vec::new();
  for token in it {
    if push_token(&mut department, &token) {
      continue;
    }
    return Err(make_error(token, None));
  }

  handle_separator(tokens, op, op_token)
}

pub fn parse(tokens: Vec<lexer::Token>) -> Result<Vec<Operation>, Error> {
  let mut res = Vec::new();
  let mut it = tokens.into_iter();

  while let Some(token) = it.next() {
    match token.value {
      KEYWORD_ADD => res.push(parse_add(token, &mut it)?),
      lexer::Whitespace => {}
      _ => {
        return Err(Error::new(
          Operation::unknown(),
          token,
          Some(token),
          None,
          None,
        ))
      }
    }
  }

  Ok(res)
}

// TODO: Rest of operations + tests
