mod constants;
mod error;
mod util;

use self::constants::*;
pub use self::error::*;
use self::util::*;
use super::lexer;
use crate::operation::{
  Modifier, Operation, OperationAdd, OperationCreate, OperationRemove,
  OperationShow,
};
use std::cmp::min;
use std::hint;

fn parse_add<'a, 'b>(
  op_token: lexer::Token<'a>,
  tokens: &'b [lexer::Token<'a>],
) -> Result<Operation, Error<'a>> {
  let error_handler = get_parse_list_error_handler_generator(OperationAdd::keyword(), op_token);
  let (names, i) = parse_list(tokens, &[LINKER_TO]).map_err(error_handler(&[LINKER_TO], "name"))?;
  let (departments, j) = parse_list(&tokens[i + 1..], &TERMINATORS)
    .map_err(error_handler(&TERMINATORS, "department"))?;
  handle_terminator(
    &tokens[min(i + j + 1, tokens.len())..],
    OperationAdd::new(departments, names, Modifier::None).into(),
    op_token,
  )
}

fn parse_create<'a, 'b>(
  op_token: lexer::Token<'a>,
  tokens: &'b [lexer::Token<'a>],
) -> Result<Operation, Error<'a>> {
  let error_handler = get_parse_list_error_handler_generator(OperationCreate::keyword(), op_token);
  let (departments, i) =
    parse_list(tokens, &TERMINATORS).map_err(error_handler(&TERMINATORS, "department"))?;
  handle_terminator(
    &tokens[min(i, tokens.len())..],
    OperationCreate::new(departments, Modifier::None).into(),
    op_token,
  )
}

fn parse_show<'a, 'b>(
  op_token: lexer::Token<'a>,
  tokens: &'b [lexer::Token<'a>],
) -> Result<Operation, Error<'a>> {
  let error_handler =
    get_parse_list_error_handler_generator(OperationShow::keyword(), op_token);
  let (departments, i) =
    parse_list(tokens, &TERMINATORS).map_err(error_handler(&TERMINATORS, "department"))?;
  handle_terminator(
    &tokens[min(i, tokens.len())..],
    OperationShow::new(departments, false).into(),
    op_token,
  )
}

fn parse_remove<'a, 'b>(
  op_token: lexer::Token<'a>,
  tokens: &'b [lexer::Token<'a>],
) -> Result<Operation, Error<'a>> {
  let error_handler =
    get_parse_list_error_handler_generator(OperationRemove::keyword(), op_token);
  const LIST_TERMINATORS: [lexer::TokenValue; 4] = [
    LINKER_FROM,
    SEPARATOR,
    SEPARATOR_OVERWRITE,
    SEPARATOR_FAIL_SILENTLY,
  ];
  let (first_list, i) = parse_list(tokens, &LIST_TERMINATORS)
    .map_err(error_handler(&TERMINATORS, "name or department"))?;
  let (second_list, j) = parse_list(&tokens[i + 1..], &TERMINATORS)
    .map_err(error_handler(&TERMINATORS, "department"))
    .unwrap_or_default();
  let (names, departments, j) = if second_list.is_empty() {
    (first_list, second_list, j)
  } else {
    (second_list, first_list, j + 1)
  };
  handle_terminator(
    &tokens[min(i + j, tokens.len())..],
    OperationRemove::new(names, departments, false).into(),
    op_token,
  )
}

pub fn parse(tokens: Vec<lexer::Token>) -> Result<Vec<Operation>, Error> {
  let mut res = Vec::new();

  let mut i = 0;
  while i < tokens.len() {
    let token = tokens[i];
    match token.value {
      lexer::Whitespace => {}
      KEYWORD_ADD | KEYWORD_CREATE | KEYWORD_SHOW | KEYWORD_REMOVE => {
        let op_tokens = get_operation_tokens(&tokens[i + 1..]);
        i += op_tokens.len();
        res.push(match token.value {
          KEYWORD_ADD => parse_add,
          KEYWORD_CREATE => parse_create,
          KEYWORD_SHOW => parse_show,
          KEYWORD_REMOVE => parse_remove,
          _ => unsafe { hint::unreachable_unchecked() },
        }(token, op_tokens)?);
      }
      _ => {
        return Err(Error::new(
          Operation::Unknown.keyword(),
          token,
          Some(token),
          Some({
            let k = &KEYWORDS[..];
            k.into()
          }),
          Some("You must input an operation!".into()),
        ));
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

  #[test]
  fn test_parse_add() {
    let test_sources = &[
      "Add Mihai, Andrei and Ioan to Science and Engineering!",
      "Add Mihai, Andrei and Ioan to Science and Engineering",
      "Add Mihai, Andrei and Ioan to .",
      "Add Mihai, Andrei and Ioan.",
      "Add to?",
      "Add Mihai",
      "Add!",
    ];
    type FnExpect = fn(Result<Operation, Error>) -> bool;
    let expect: &[FnExpect] = &[
      |res| {
        res
          == Ok(
            OperationAdd::new(
              util::to_string_vec(vec!["Science", "Engineering"]),
              util::to_string_vec(vec!["Mihai", "Andrei", "Ioan"]),
              Modifier::Overwrite,
            )
            .into(),
          )
      },
      |res| res.is_err(),
      |res| res.is_err(),
      |res| res.is_err(),
      |res| res.is_err(),
      |res| res.is_err(),
      |res| res.is_err(),
    ];
    test_sources
      .iter()
      .map(|s| lexer::lex(s).unwrap())
      .map(|t| parse_add(t[0], &t[1..]))
      .zip(expect.iter())
      .for_each(|(res, f)| assert!(f(res)));
  }

  #[test]
  fn test_parse_create() {
    let tokens = lexer::lex("Create Science, Engineering and Physics.").unwrap();
    let expect = OperationCreate::new(
      util::to_string_vec(vec!["Science", "Engineering", "Physics"]),
      Modifier::None,
    )
    .into();
    let got = parse(tokens).unwrap();
    assert_eq!(got[0], expect);
  }

  #[test]
  fn test_parse_show() {
    let tokens = lexer::lex("Show Science, Engineering and Physics?").unwrap();
    let expect = OperationShow::new(
      util::to_string_vec(vec!["Science", "Engineering", "Physics"]),
      true,
    )
    .into();
    let got = parse(tokens).unwrap();
    assert_eq!(got[0], expect);
  }

  #[test]
  fn test_parse_remove() {
    let source = vec!["Remove Science.", "Remove Michael from Physics?"];
    let expect = vec![
      OperationRemove::new(util::to_string_vec(vec!["Science"]), vec![], false).into(),
      OperationRemove::new(
        util::to_string_vec(vec!["Physics"]),
        util::to_string_vec(vec!["Michael"]),
        true,
      )
      .into(),
    ];
    source
      .into_iter()
      .map(|s| parse(lexer::lex(s).unwrap()).unwrap())
      .zip(expect.into_iter())
      .for_each(|(got, expected)| assert_eq!(got[0], expected));
  }
  // TODO: more tests
}
