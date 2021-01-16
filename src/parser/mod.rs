mod error;
mod operation;
pub use self::error::*;
pub use self::operation::*;

use super::lexer;
use std::cmp::min;

const KEYWORD_ADD: lexer::TokenValue = lexer::TokenValue::Word("Add");
const KEYWORD_CREATE: lexer::TokenValue = lexer::TokenValue::Word("Create");
const KEYWORD_REMOVE: lexer::TokenValue = lexer::TokenValue::Word("Remove");
const KEYWORD_SHOW: lexer::TokenValue = lexer::TokenValue::Word("Show");
const KEYWORDS: [lexer::TokenValue; 4] =
  [KEYWORD_ADD, KEYWORD_CREATE, KEYWORD_REMOVE, KEYWORD_SHOW];

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
const TERMINATORS: [lexer::TokenValue; 3] =
  [SEPARATOR, SEPARATOR_OVERWRITE, SEPARATOR_FAIL_SILENTLY];

fn handle_terminator<'a, 'b>(
  tokens: &'b [lexer::Token<'a>],
  op: Operation,
  op_token: lexer::Token<'a>,
) -> Result<Operation, Error<'a>> {
  let get_terminators = |op_kind| -> &[lexer::TokenValue] {
    match op_kind {
      OperationKind::Unknown => {
        panic!("Unknown operations should not be passed to handle_terminator")
      }
      OperationKind::Create | OperationKind::Add => &[SEPARATOR_OVERWRITE, SEPARATOR_FAIL_SILENTLY],
      OperationKind::Show | OperationKind::Remove => &[SEPARATOR_FAIL_SILENTLY],
    }
  };
  match tokens.len() {
    0 => Err(Error::new(
      op.kind(),
      op_token,
      None,
      Some(get_terminators(op.kind()).into()),
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
          Some(get_terminators(op_kind).into()),
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
  terminators: &[lexer::TokenValue],
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
      _ if terminators.contains(&token.value) => {
        return Ok((
          &tokens[first_word_index.unwrap_or_default()..=last_word_index],
          i,
          true,
        ))
      }
      lexer::Whitespace => {}
      lexer::Word(_) => {
        if first_word_index.is_some() {
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

fn get_string_from_tokens(tokens: &[lexer::Token]) -> String {
  let mut v = tokens.to_vec();
  v.dedup();
  v.into_iter().map(|t| t.value.get().to_string()).collect()
}

fn parse_list<'a, 'b>(
  tokens: &'b [lexer::Token<'a>],
  terminators: &[lexer::TokenValue],
) -> Result<(Vec<String>, usize), (lexer::Token<'a>, bool)> {
  let mut elements = Vec::new();

  let mut i = 0;
  while i < tokens.len() {
    let token = tokens[i];

    match token.value {
      lexer::Whitespace => {}
      _ if terminators.contains(&token.value) => break,
      lexer::Word(_) => match get_list_element_tokens(&tokens[i..], terminators) {
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
      _ => return Err((token, elements.is_empty())),
    }
    i += 1;
  }
  let ret: Vec<String> = elements.into_iter().map(get_string_from_tokens).collect();
  if ret.is_empty() {
    Err((tokens[min(i, tokens.len() - 1)], true))
  } else {
    Ok((ret, i))
  }
}

fn get_list_error_handler<'a>(
  op_kind: OperationKind,
  op_token: lexer::Token<'a>,
  terminators: &'static [lexer::TokenValue],
  list_elements_name: &'static str,
) -> impl FnOnce((lexer::Token<'a>, bool)) -> Error<'a> {
  move |(t, is_empty)| {
    const EXPECTED: &[lexer::TokenValue] = &[lexer::Whitespace, lexer::Word("")];
    if is_empty {
      Error::new(
        op_kind,
        op_token,
        Some(t),
        Some(EXPECTED.into()),
        Some(
          format!(
            "You must specify at least one {} before list terminator {}",
            list_elements_name, t.value
          )
          .into(),
        ),
      )
    } else {
      Error::new(
        op_kind,
        op_token,
        Some(t),
        Some([EXPECTED, terminators].concat().into()),
        match () {
          _ if RESERVED.contains(&t.value) => {
            Some(format!("Can't use {} in lists, it's reserved!", t.value).into())
          }
          _ => None,
        },
      )
    }
  }
}

fn parse_add<'a, 'b>(
  op_token: lexer::Token<'a>,
  tokens: &'b [lexer::Token<'a>],
) -> Result<Operation, Error<'a>> {
  let error_handler = |terminators: &'static [lexer::TokenValue], list_elements_name| {
    get_list_error_handler(
      OperationKind::Add,
      op_token,
      terminators,
      list_elements_name,
    )
  };
  let (names, i) = parse_list(tokens, &[LINKER_TO]).map_err(error_handler(&[LINKER_TO], "name"))?;
  let (departments, j) = parse_list(&tokens[i + 1..], &TERMINATORS)
    .map_err(error_handler(&TERMINATORS, "department"))?;
  handle_terminator(
    &tokens[min(i + j + 1, tokens.len())..],
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
  tokens
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
          Some({
            let k = &KEYWORDS[..];
            k.into()
          }),
          Some("You must input an operation!".into()),
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
  use crate::util;

  fn tv(tokens: Vec<lexer::Token>) -> Vec<lexer::TokenValue> {
    tokens.into_iter().map(|t| t.value).collect()
  }

  #[test]
  fn test_get_list_element_tokens() {
    let tokens = lexer::lex("Moraru    Mihaela  , Mircea Ioan and Amalia Brad").unwrap();
    let (got, ..) = get_list_element_tokens(&tokens, &[]).unwrap();
    let expect = lexer::lex("Moraru Mihaela").unwrap();
    assert_eq!(tv(got.into()), tv(expect));
  }
  #[test]
  fn test_parse_list() {
    let tokens = lexer::lex("Moraru   Mihaela  , Mircea Ioan and Amalia Brad.").unwrap();
    let (got, ..) = parse_list(&tokens, &[SEPARATOR]).unwrap();
    let expect = util::to_string_vec(vec!["Moraru Mihaela", "Mircea Ioan", "Amalia Brad"]);
    assert_eq!(got, expect);
  }
  #[test]
  fn test_parse_add() {
    let test_sources = &[
      "Add Mihai, Andrei and Ioan to Science and Engineering!",
      "Add Mihai, Andrei and Ioan to Science and Engineering",
      "Add Mihai, Andrei and Ioan to .",
      "Add Mihai, Andrei and Ioan.",
      "Add to?",
      "Add!",
    ];
    type FnExpect = fn(Result<Operation, Error>) -> bool;
    let expect: &[FnExpect] = &[
      |res| {
        res
          == Ok(Operation::add(
            util::to_string_vec(vec!["Science", "Engineering"]),
            false,
            util::to_string_vec(vec!["Mihai", "Andrei", "Ioan"]),
            true,
          ))
      },
      |res| res.is_err(),
      |res| res.is_err(),
      |res| res.is_err(),
      |res| res.is_err(),
      |res| res.is_err(),
    ];
    test_sources
      .iter()
      .map(|s| lexer::lex(s))
      .map(|t| t.unwrap())
      .map(|t| parse_add(t[0], &t[1..]))
      .zip(expect.iter())
      .map(|(res, f)| {
        res.clone().err().iter().for_each(|e| println!("{}", e));
        f(res)
      })
      .for_each(|v| assert!(v));
  }
  // TODO: more tests
}
