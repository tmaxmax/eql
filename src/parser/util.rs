use super::constants::*;
use super::error::Error;
use crate::lexer;
use crate::operation::{Modifier, Operation};
use std::cmp::min;

pub fn handle_terminator<'a, 'b>(
  tokens: &'b [lexer::Token<'a>],
  op: Operation,
  op_token: lexer::Token<'a>,
) -> Result<Operation, Error<'a>> {
  let get_terminators = |o: &Operation| -> &[lexer::TokenValue] {
    match o {
      Operation::Unknown => {
        panic!("Unknown operations should not be passed to handle_terminator")
      }
      Operation::Create(_) | Operation::Add(_) => {
        &[SEPARATOR, SEPARATOR_OVERWRITE, SEPARATOR_FAIL_SILENTLY]
      }
      Operation::Show(_) | Operation::Remove(_) => &[SEPARATOR, SEPARATOR_FAIL_SILENTLY],
    }
  };
  match tokens.len() {
    0 => Err(Error::new(
      op.keyword(),
      op_token,
      None,
      Some(get_terminators(&op).into()),
      Some("You didn't terminate your operation!".into()),
    )),
    1 => {
      let value = tokens[0].value;
      let op_keyword = op.keyword();
      let terminators = get_terminators(&op);
      match value {
        SEPARATOR_OVERWRITE => op.set_modifier(Modifier::Overwrite),
        SEPARATOR_FAIL_SILENTLY => op.set_modifier(Modifier::FailSilently),
        _ => Some(op),
      }
      .ok_or_else(|| {
        Error::new(
          op_keyword,
          op_token,
          Some(tokens[0]),
          Some(terminators.into()),
          Some(format!("{} is not a valid modifier for the operation", value).into()),
        )
      })
    }
    _ => unreachable!(),
  }
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct ListElement<'a, 'b> {
  tokens: &'b [lexer::Token<'a>],
  consumed_count: usize,
  is_last_consumed_terminator: bool,
}

fn get_list_element_tokens<'a, 'b>(
  tokens: &'b [lexer::Token<'a>],
  terminators: &[lexer::TokenValue],
) -> Result<ListElement<'a, 'b>, usize> {
  let mut last_word_index = 0;
  let mut first_word_index = None;

  for i in 0..tokens.len() {
    let token = tokens[i];

    match token.value {
      SEPARATOR_VALUES | LINKER_AND => {
        return Ok(ListElement {
          tokens: &tokens[first_word_index.unwrap_or_default()..=last_word_index],
          consumed_count: i,
          is_last_consumed_terminator: false,
        });
      }
      _ if terminators.contains(&token.value) => {
        return Ok(ListElement {
          tokens: &tokens[first_word_index.unwrap_or_default()..=last_word_index],
          consumed_count: i,
          is_last_consumed_terminator: true,
        });
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
  Ok(ListElement {
    tokens: &tokens[first_word_index.unwrap_or_default()..=last_word_index],
    consumed_count: tokens.len(),
    is_last_consumed_terminator: false,
  })
}

fn get_string_from_tokens(tokens: &[lexer::Token]) -> String {
  let mut v = tokens.to_vec();
  v.dedup();
  v.into_iter().map(|t| t.value.get().to_string()).collect()
}

#[derive(Clone, Debug)]
pub struct ParseListError<'a> {
  unexpected_token: Option<lexer::Token<'a>>,
  has_parsed_elements: bool,
}

// FIXME: Handle "elem, and elem" case
pub fn parse_list<'a, 'b>(
  tokens: &'b [lexer::Token<'a>],
  terminators: &[lexer::TokenValue],
) -> Result<(Vec<String>, usize), ParseListError<'a>> {
  let mut elements = Vec::new();
  let mut last_linker = None;

  let mut i = 0;
  while i < tokens.len() {
    let token = tokens[i];

    match token.value {
      lexer::Whitespace => {}
      _ if terminators.contains(&token.value) => break,
      LINKER_AND => {
        if elements.is_empty() {
          break;
        }
        match last_linker {
          Some(linker) => match linker {
            SEPARATOR_VALUES => last_linker = Some(linker),
            _ => {
              return Err(ParseListError {
                unexpected_token: Some(token),
                has_parsed_elements: true,
              })
            }
          },
          _ => {
            return Err(ParseListError {
              unexpected_token: Some(token),
              has_parsed_elements: true,
            })
          }
        }
      }
      lexer::Word(_) => match get_list_element_tokens(&tokens[i..], terminators) {
        Ok(elem) => {
          i += elem.consumed_count;
          elements.push(elem.tokens);
          if elem.is_last_consumed_terminator {
            continue;
          }
          last_linker = tokens.get(i).map(|t| t.value)
        }
        Err(incr) => {
          i += incr;
          continue;
        }
      },
      _ => {
        return Err(ParseListError {
          unexpected_token: Some(token),
          has_parsed_elements: !elements.is_empty(),
        })
      }
    }
    i += 1;
  }
  let ret: Vec<String> = elements.into_iter().map(get_string_from_tokens).collect();
  if ret.is_empty() {
    Err(ParseListError {
      unexpected_token: tokens
        .get(min(i, tokens.len().checked_sub(1).unwrap_or_default()))
        .cloned(),
      has_parsed_elements: false,
    })
  } else if i > tokens.len() {
    Err(ParseListError {
      unexpected_token: tokens.last().cloned(),
      has_parsed_elements: true,
    })
  } else {
    Ok((ret, i))
  }
}

pub fn get_parse_list_error_handler_generator<'a>(
  op_keyword: &'static str,
  op_token: lexer::Token<'a>,
) -> impl Fn(
  &'static [lexer::TokenValue<'static>],
  &'static str,
) -> Box<dyn Fn(ParseListError<'a>) -> Error<'a> + 'a> {
  move |terminators, name| {
    Box::new(
      move |ParseListError {
              unexpected_token,
              has_parsed_elements,
            }| {
        const EXPECTED: &[lexer::TokenValue] = &[lexer::Whitespace, lexer::Word("")];
        if has_parsed_elements {
          Error::new(
            op_keyword,
            op_token,
            unexpected_token,
            Some([EXPECTED, terminators].concat().into()),
            Some(
              unexpected_token
                .map(|v| v.value)
                .filter(|v| RESERVED.contains(&v))
                .map_or_else(
                  || format!("The {} list you entered is invalid!", name).into(),
                  |v| format!("Can't use {} in lists, it's reserved!", v).into(),
                ),
            ),
          )
        } else {
          Error::new(
            op_keyword,
            op_token,
            unexpected_token,
            Some(EXPECTED.into()),
            Some(
              format!(
                "You must specify at least one {}{}",
                name,
                unexpected_token
                  .map(|v| v.value)
                  .map(|v| format!(
                    " before {} {}",
                    if terminators.contains(&v) {
                      "list terminator"
                    } else if TERMINATORS.contains(&v) {
                      "operation terminator"
                    } else {
                      "list element separator"
                    },
                    v.to_string()
                  ))
                  .unwrap_or_default()
              )
              .into(),
            ),
          )
        }
      },
    )
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

  #[test]
  fn parse_list_usual() {
    let tokens = lexer::lex("Moraru   Mihaela  , Mircea Ioan and Amalia Brad.").unwrap();
    let (got, ..) = parse_list(&tokens, &[SEPARATOR]).unwrap();
    let expect = util::to_string_vec(vec!["Moraru Mihaela", "Mircea Ioan", "Amalia Brad"]);
    assert_eq!(got, expect);
  }

  #[test]
  fn parse_list_multiple_consecutive_separators() {
    vec!["Mama, and, Tata.", "Mama,, Tata.", "Mama and and Tata."]
      .into_iter()
      .map(lexer::lex)
      .map(Result::unwrap)
      .map(|tokens| parse_list(&tokens, &[SEPARATOR]).map(|res| res.0))
      .for_each(|res| assert!(res.is_err()));
  }
  // TODO: more tests
}
