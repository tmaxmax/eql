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

// TODO: Implement handle_separator
fn handle_separator<'a>(
  it: &mut (impl Iterator<Item = lexer::Token<'a>> + Clone),
  op: Operation,
  op_token: lexer::Token<'a>,
) -> Result<Operation, Error<'a>> {
  let separator = it.skip_while(|t| is_operation_separator(t.value)).next();
  if let Some(sep) = separator {
    unimplemented!()
  } else {
    Err(Error::new(
      op.kind(),
      op_token,
      None,
      Some(TERMINATORS),
      None,
    ))
  }
}

fn get_name_tokens<'a, 'b>(
  tokens: &'b [lexer::Token<'a>],
) -> Result<(&'b [lexer::Token<'a>], bool), usize> {
  for i in 0..tokens.len() {
    let token = tokens[i];
    match token.value {
      SEPARATOR_VALUES | LINKER_AND => return Ok((&tokens[..i], false)),
      LINKER_TO | LINKER_FROM => return Ok((&tokens[..i], true)),
      lexer::Whitespace | lexer::Word(_) => {}
      _ => return Err(i),
    }
  }
  Ok((tokens, false))
}

fn get_department_tokens<'a, 'b>(
  tokens: &'b [lexer::Token<'a>],
) -> Result<&'b [lexer::Token<'a>], usize> {
  for i in 0..tokens.len() {
    let token = tokens[i];
    match token.value {
      lexer::Whitespace | lexer::Word(_) => {}
      _ if TERMINATORS.contains(&token.value) => return Ok(&tokens[..i]),
      _ => return Err(i),
    }
  }
  Ok(tokens)
}

fn parse_add<'a, 'b>(
  op_token: lexer::Token<'a>,
  tokens: &'b [lexer::Token<'a>],
) -> Result<Operation, Error<'a>> {
  let mut names = Vec::new();
  // let department_tokens;

  let mut i = 0;
  while i < tokens.len() {
    let token = tokens[i];

    match token.value {
      lexer::Whitespace => {}
      LINKER_TO => {
        // TODO: Implement linker handler
        unimplemented!();
      }
      lexer::Word(_) => match get_name_tokens(&tokens[i..]) {
        Ok((name_tokens, is_linker)) => {
          i += name_tokens.len();
          names.push(name_tokens);
          if is_linker {
            continue;
          }
        }
        Err(increment) => {
          i += increment;
          continue;
        }
      },
      _ => {
        let details: Option<Cow<str>> = match () {
          _ if RESERVED.contains(&token.value) => {
            Some(format!("Can't use {} in names, it's reserved!", token.value).into())
          }
          _ => None,
        };
        return Err(Error::new(
          OperationKind::Add,
          op_token,
          Some(token),
          Some(&[lexer::Whitespace, lexer::Word(""), LINKER_TO]),
          details,
        ));
      }
    }
    i += 1;
  }
  // TODO: implement rest of parse_add
  Ok(Operation::unknown())
}

fn get_statement_slice<'a, 'b>(tokens: &'b [lexer::Token<'a>]) -> &'b [lexer::Token<'a>] {
  for i in 0..tokens.len() {
    if TERMINATORS.contains(&tokens[i].value) {
      return &tokens[..=i];
    }
  }
  return tokens;
}

pub fn parse(tokens: Vec<lexer::Token>) -> Result<Vec<Operation>, Error> {
  let mut res = Vec::new();

  let mut i = 0;
  while i < tokens.len() {
    let token = tokens[i];
    match token.value {
      lexer::Whitespace => {}
      KEYWORD_ADD => {
        let add_tokens = get_statement_slice(&tokens[i + 1..]);
        i += add_tokens.len();
        res.push(parse_add(token, add_tokens)?)
      }
      _ => {
        return Err(Error::new(
          OperationKind::Unknown,
          token,
          Some(token),
          Some(&[KEYWORD_ADD, KEYWORD_CREATE, KEYWORD_REMOVE, KEYWORD_SHOW]),
          Some(format!("You must input an operation!").into()),
        ))
      }
    }
    i += 1;
  }

  Ok(res)
}

// TODO: Rest of operations + tests
