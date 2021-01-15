mod error;
mod operation;
pub use self::error::*;
pub use self::operation::*;

use super::lexer;
use crate::util;
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

fn handle_terminator<'a, 'b>(
  tokens: &'b [lexer::Token<'a>],
  op: Operation,
  op_token: lexer::Token<'a>,
) -> Result<Operation, Error<'a>> {
  match tokens.len() {
    0 => Err(Error::new(
      op.kind(),
      op_token,
      None,
      Some(TERMINATORS),
      Some("You didn't terminate your operation!".into()),
    )),
    1 => {
      let value = tokens[0].value;
      let op_kind = op.kind();
      match value {
        SEPARATOR_OVERWRITE => op.set_overwrite(true),
        SEPARATOR_FAIL_SILENTLY => op.set_fail_silently(true),
        _ => Some(op),
      }
      .ok_or_else(|| {
        Error::new(
          op_kind,
          op_token,
          Some(tokens[0]),
          Some(match op_kind {
            OperationKind::Unknown => {
              panic!("Unknown operations should not be passed to handle_terminator")
            }
            OperationKind::Create | OperationKind::Add => {
              &[SEPARATOR_OVERWRITE, SEPARATOR_FAIL_SILENTLY]
            }
            OperationKind::Show | OperationKind::Remove => &[SEPARATOR_FAIL_SILENTLY],
          }),
          Some(format!("{} is not a valid terminator for the operation", value).into()),
        )
      })
    }
    _ => panic!(
      "Invalid input for handle_terminator, expected 1 or no tokens, got {}: {:#?}",
      tokens.len(),
      tokens
    ),
  }
}

fn get_list_element_tokens<'a, 'b>(
  tokens: &'b [lexer::Token<'a>],
) -> Result<(&'b [lexer::Token<'a>], usize, bool), usize> {
  let mut last_word_index = 0;
  let mut first_word_index = None;

  for i in 0..tokens.len() {
    let token = tokens[i];
    match token.value {
      SEPARATOR_VALUES | LINKER_AND => {
        return Ok((
          &tokens[first_word_index.unwrap_or_default()..=last_word_index],
          i,
          false,
        ))
      }
      LINKER_TO | LINKER_FROM => {
        return Ok((
          &tokens[first_word_index.unwrap_or_default()..=last_word_index],
          i,
          true,
        ))
      }
      lexer::Whitespace => {}
      lexer::Word(_) => {
        if let Some(_) = first_word_index {
          last_word_index = i;
        } else {
          first_word_index = Some(i);
        }
      }
      _ => return Err(i),
    }
  }
  Ok((
    &tokens[first_word_index.unwrap_or_default()..=last_word_index],
    tokens.len(),
    false,
  ))
}

fn get_string_from_tokens<'a, 'b>(tokens: &'b [lexer::Token<'a>]) -> String {
  let mut v = tokens.to_vec();
  v.dedup();
  v.into_iter().map(|t| t.value.get().to_string()).collect()
}

fn parse_list<'a, 'b>(
  tokens: &'b [lexer::Token<'a>],
  terminators: &[lexer::TokenValue],
) -> Result<(Vec<String>, usize), Option<lexer::Token<'a>>> {
  let mut elements = Vec::new();

  let mut i = 0;
  while i < tokens.len() {
    let token = tokens[i];

    match token.value {
      lexer::Whitespace => {}
      _ if terminators.contains(&token.value) => break,
      lexer::Word(_) => match get_list_element_tokens(&tokens[i..]) {
        Ok((elem_tokens, incr, is_terminator)) => {
          i += incr;
          elements.push(elem_tokens);
          if is_terminator {
            continue;
          }
        }
        Err(incr) => {
          i += incr;
          continue;
        }
      },
      _ => return Err(Some(token)),
    }
    i += 1;
  }
  let ret: Vec<String> = elements.into_iter().map(get_string_from_tokens).collect();
  if ret.is_empty() {
    Err(None)
  } else {
    Ok((ret, i))
  }
}

fn parse_add<'a, 'b>(
  op_token: lexer::Token<'a>,
  tokens: &'b [lexer::Token<'a>],
) -> Result<Operation, Error<'a>> {
  let mut i = 0;
  let mut lists = Vec::new();
  for terminator in &[&[LINKER_TO], TERMINATORS] {
    let (list, incr) = parse_list(&tokens[i..], terminator).or_else(|t| {
      const EXPECTED: &[lexer::TokenValue] = &[lexer::Whitespace, lexer::Word("")];
      Err(t.map_or_else(
        || {
          Error::new(
            OperationKind::Add,
            op_token,
            None,
            Some(EXPECTED),
            Some("List is empty!".into()),
          )
        },
        |t| {
          Error::new(
            OperationKind::Add,
            op_token,
            Some(t),
            Some(EXPECTED),
            match () {
              _ if RESERVED.contains(&t.value) => {
                Some(format!("Can't use {} in lists, it's reserved!", t.value).into())
              }
              _ => None,
            },
          )
        },
      ))
    })?;
    lists.push(list);
    i += incr + 1;
  }
  let (names, departments) = (lists.swap_remove(0), lists.swap_remove(0));
  handle_terminator(
    &tokens[i - 1..],
    Operation::add(departments, false, names, false),
    op_token,
  )
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

#[cfg(test)]
mod tests {
  use super::*;
  fn tv<'a>(tokens: Vec<lexer::Token<'a>>) -> Vec<lexer::TokenValue<'a>> {
    tokens.into_iter().map(|t| t.value).collect()
  }

  // TODO: fix list parser
  #[test]
  fn test_get_list_element_tokens() {
    let tokens = lexer::lex("Moraru    Mihaela  , Mircea Ioan and Amalia Brad").unwrap();
    let (got, ..) = get_list_element_tokens(&tokens).unwrap();
    let expect = lexer::lex("Moraru Mihaela").unwrap();
    assert_eq!(tv(got.into()), tv(expect));
  }
  // TODO: more tests
}
