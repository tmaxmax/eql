use super::lexer;
use crate::operation;
use crate::util;
use std::borrow::Cow;
use std::error;
use std::fmt;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Error<'a> {
  operation_kind: operation::OperationKind,
  operation_token: lexer::Token<'a>,
  unexpected_token: Option<lexer::Token<'a>>,
  expected_tokens: Option<Cow<'static, [lexer::TokenValue<'static>]>>,
  details: Option<Cow<'static, str>>,
}

impl<'a> Error<'a> {
  pub fn new(
    operation_kind: operation::OperationKind,
    operation_token: lexer::Token<'a>,
    unexpected_token: Option<lexer::Token<'a>>,
    expected_tokens: Option<Cow<'static, [lexer::TokenValue<'static>]>>,
    details: Option<Cow<'static, str>>,
  ) -> Self {
    Error {
      operation_kind,
      operation_token,
      unexpected_token,
      expected_tokens,
      details,
    }
  }
}

impl error::Error for Error<'_> {}

fn fmt_unexpected(e: &Error) -> String {
  e.unexpected_token
    .filter(|t| *t != e.operation_token)
    .map_or("".into(), |un_token| {
      let s = format!("Unexpected {}", un_token.value);
      if un_token.line != e.operation_token.line {
        let (padding, pointer) =
          util::fmt_token_pointer(un_token.value.get(), un_token.column_number);
        format!(
          "\n{} on line {}, column {}:
  {}
  {}{}",
          s,
          un_token.line_number,
          un_token.column_number,
          un_token.line.trim_end(),
          padding,
          pointer
        )
      } else {
        let offset = un_token.column_number
          - e.operation_token.column_number
          - util::string_length(e.operation_token.value.get())
          + 1;
        let (padding, pointer) = util::fmt_token_pointer(un_token.value.get(), offset);
        format!("{}{}\n{}", padding, pointer, s)
      }
    })
}

fn fmt_expected(e: &Error) -> String {
  e.expected_tokens.as_ref().map_or("".into(), |ex_tokens| {
    let expected_list = util::fmt_list(ex_tokens, ", ", "or");
    if expected_list.is_empty() {
      "".into()
    } else {
      format!(
        "\nExpected {}{}",
        expected_list,
        e.unexpected_token.map_or("", |_| " instead")
      )
    }
  })
}

impl fmt::Display for Error<'_> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let op_token = &self.operation_token;
    let un_token = self
      .unexpected_token
      .filter(|t| t.line_number == op_token.line_number)
      .unwrap_or(*op_token);
    let (padding, pointer) = util::fmt_token_pointer(op_token.value.get(), op_token.column_number);
    write!(
      f,
      "Error on {} operation on line {}, column {}:\n  {}\n  {}{}{}{}{}",
      self.operation_kind,
      un_token.line_number,
      un_token.column_number,
      op_token.line.trim_end(),
      padding,
      pointer,
      fmt_unexpected(self),
      fmt_expected(self),
      self
        .details
        .as_ref()
        .map_or("".into(), |text| ["\n", text].join(""))
    )
  }
}

// TODO: Create tests
