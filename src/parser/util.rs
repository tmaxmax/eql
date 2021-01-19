use super::constants::*;
use super::error::Error;
use crate::lexer;
use crate::operation::{self, Operation};
use std::cmp::min;
use std::hint;

pub fn handle_terminator<'a, 'b>(
  tokens: &'b [lexer::Token<'a>],
  op: Operation,
  op_token: lexer::Token<'a>,
) -> Result<Operation, Error<'a>> {
  let get_terminators = |op_kind| -> &[lexer::TokenValue] {
    match op_kind {
      operation::Unknown => {
        panic!("Unknown operations should not be passed to handle_terminator")
      }
      operation::Create | operation::Add => {
        &[SEPARATOR, SEPARATOR_OVERWRITE, SEPARATOR_FAIL_SILENTLY]
      }
      operation::Show | operation::Remove => &[SEPARATOR, SEPARATOR_FAIL_SILENTLY],
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
    _ => unsafe { hint::unreachable_unchecked() },
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

type ParseListError<'a> = (Option<lexer::Token<'a>>, bool);

// FIXME: Handle "elem, and elem" case
pub fn parse_list<'a, 'b>(
  tokens: &'b [lexer::Token<'a>],
  terminators: &[lexer::TokenValue],
) -> Result<(Vec<String>, usize), ParseListError<'a>> {
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
      _ => return Err((Some(token), elements.is_empty())),
    }
    i += 1;
  }
  let ret: Vec<String> = elements.into_iter().map(get_string_from_tokens).collect();
  if ret.is_empty() {
    Err((
      tokens
        .get(min(i, tokens.len().checked_sub(1).unwrap_or_default()))
        .cloned(),
      true,
    ))
  } else if i > tokens.len() {
    Err((tokens.last().cloned(), false))
  } else {
    Ok((ret, i))
  }
}

pub fn get_parse_list_error_handler_generator<'a>(
  op_kind: operation::OperationKind,
  op_token: lexer::Token<'a>,
) -> impl Fn(
  &'static [lexer::TokenValue<'static>],
  &'static str,
) -> Box<dyn Fn(ParseListError<'a>) -> Error<'a> + 'a> {
  move |terminators, name| {
    Box::new(move |(t, is_empty)| {
      const EXPECTED: &[lexer::TokenValue] = &[lexer::Whitespace, lexer::Word("")];
      if is_empty {
        Error::new(
          op_kind,
          op_token,
          t,
          Some(EXPECTED.into()),
          Some(
            format!(
              "You must specify at least one {} before list terminator{}",
              name,
              t.map(|v| format!(" {}", v.value.to_string()))
                .unwrap_or_default(),
            )
            .into(),
          ),
        )
      } else {
        Error::new(
          op_kind,
          op_token,
          t.filter(|v| !matches!(v.value, lexer::Word(_))),
          Some([EXPECTED, terminators].concat().into()),
          Some(
            t.map(|v| v.value)
              .filter(|v| RESERVED.contains(&v))
              .map_or_else(
                || "The list you entered is not terminated!".into(),
                |v| format!("Can't use {} in lists, it's reserved!", v).into(),
              ),
          ),
        )
      }
    })
  }
}

pub fn get_operation_tokens<'a, 'b>(tokens: &'b [lexer::Token<'a>]) -> &'b [lexer::Token<'a>] {
  for i in 0..tokens.len() {
    if TERMINATORS.contains(&tokens[i].value) {
      return &tokens[..=i];
    }
  }
  tokens
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
  // TODO: more tests
}
