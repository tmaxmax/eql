mod constants;
mod error;
mod operation;
mod util;

pub use self::error::*;
pub use self::operation::*;

use self::constants::*;
use self::util::*;
use super::lexer;
use std::cmp::min;

fn parse_add<'a, 'b>(
  op_token: lexer::Token<'a>,
  tokens: &'b [lexer::Token<'a>],
) -> Result<Operation, Error<'a>> {
  let error_handler = ListErrorHandler::new(OperationKind::Add, op_token);
  let (names, i) = parse_list(tokens, &[LINKER_TO])
    .map_err(|v| error_handler.get_handler(&[LINKER_TO], "name").handle(v))?;
  let (departments, j) = parse_list(&tokens[i + 1..], &TERMINATORS).map_err(|v| {
    error_handler
      .get_handler(&TERMINATORS, "department")
      .handle(v)
  })?;
  handle_terminator(
    &tokens[min(i + j + 1, tokens.len())..],
    Operation::add(departments, false, names, false),
    op_token,
  )
}

fn parse_create<'a, 'b>(
  op_token: lexer::Token<'a>,
  tokens: &'b [lexer::Token<'a>],
) -> Result<Operation, Error<'a>> {
  let error_handler = ListErrorHandler::new(OperationKind::Create, op_token);
  let (departments, i) = parse_list(tokens, &TERMINATORS).map_err(|v| {
    error_handler
      .get_handler(&TERMINATORS, "department")
      .handle(v)
  })?;
  handle_terminator(
    &tokens[min(i, tokens.len())..],
    Operation::create(departments, false, false),
    op_token,
  )
}

fn parse_show<'a, 'b>(
  op_token: lexer::Token<'a>,
  tokens: &'b [lexer::Token<'a>],
) -> Result<Operation, Error<'a>> {
  let error_handler = ListErrorHandler::new(OperationKind::Show, op_token);
  let (departments, i) = parse_list(tokens, &TERMINATORS).map_err(|v| {
    error_handler
      .get_handler(&TERMINATORS, "department")
      .handle(v)
  })?;
  handle_terminator(
    &tokens[min(i, tokens.len())..],
    Operation::show(departments, false),
    op_token,
  )
}

fn parse_remove<'a, 'b>(
  op_token: lexer::Token<'a>,
  tokens: &'b [lexer::Token<'a>],
) -> Result<Operation, Error<'a>> {
  let error_handler = ListErrorHandler::new(OperationKind::Remove, op_token);
  const LIST_TERMINATORS: [lexer::TokenValue; 4] = [
    LINKER_FROM,
    SEPARATOR,
    SEPARATOR_OVERWRITE,
    SEPARATOR_FAIL_SILENTLY,
  ];
  let (first_list, i) = parse_list(tokens, &LIST_TERMINATORS).map_err(|v| {
    error_handler
      .get_handler(&TERMINATORS, "name or department")
      .handle(v)
  })?;
  let (second_list, j) = parse_list(&tokens[i + 1..], &TERMINATORS)
    .map_err(|v| {
      error_handler
        .get_handler(&TERMINATORS, "department")
        .handle(v)
    })
    .unwrap_or_default();
  let (names, departments, j) = if second_list.is_empty() {
    (first_list, second_list, j)
  } else {
    (second_list, first_list, j + 1)
  };
  handle_terminator(
    &tokens[min(i + j, tokens.len())..],
    Operation::remove(names, false, departments),
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
          _ => unreachable!(),
        }(token, op_tokens)?);
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
      .map(|s| lexer::lex(s).unwrap())
      .map(|t| parse_add(t[0], &t[1..]))
      .zip(expect.iter())
      .for_each(|(res, f)| assert!(f(res)));
  }

  #[test]
  fn test_parse_create() {
    let tokens = lexer::lex("Create Science, Engineering and Physics.").unwrap();
    let expect = Operation::create(
      util::to_string_vec(vec!["Science", "Engineering", "Physics"]),
      false,
      false,
    );
    let got = parse(tokens).unwrap();
    assert_eq!(got[0], expect);
  }

  #[test]
  fn test_parse_show() {
    let tokens = lexer::lex("Show Science, Engineering and Physics?").unwrap();
    let expect = Operation::show(
      util::to_string_vec(vec!["Science", "Engineering", "Physics"]),
      true,
    );
    let got = parse(tokens).unwrap();
    assert_eq!(got[0], expect);
  }

  #[test]
  fn test_parse_remove() {
    let source = vec!["Remove Science.", "Remove Michael from Physics?"];
    let expect = vec![
      Operation::remove(util::to_string_vec(vec!["Science"]), false, vec![]),
      Operation::remove(
        util::to_string_vec(vec!["Physics"]),
        true,
        util::to_string_vec(vec!["Michael"]),
      ),
    ];
    source
      .into_iter()
      .map(|s| parse(lexer::lex(s).unwrap()).unwrap())
      .zip(expect.into_iter())
      .for_each(|(got, expected)| assert_eq!(got[0], expected));
  }
  // TODO: more tests
}
