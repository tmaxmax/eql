use super::lexer;
use super::operation::Operation;
use crate::util;
use std::borrow::Cow;
use std::fmt;

pub struct Error<'a> {
  operation: Operation,
  operation_token: lexer::Token<'a>,
  unexpected_token: Option<lexer::Token<'a>>,
  expected_token: Option<&'static [lexer::TokenValue<'static>]>,
  details: Option<Cow<'static, str>>,
}

impl<'a> Error<'a> {
  pub fn new(
    operation: Operation,
    operation_token: lexer::Token<'a>,
    unexpected_token: Option<lexer::Token<'a>>,
    expected_token: Option<&'static [lexer::TokenValue<'static>]>,
    details: Option<Cow<'static, str>>,
  ) -> Self {
    Error {
      operation,
      operation_token,
      unexpected_token,
      expected_token,
      details,
    }
  }
}

fn fmt_unexpected<'a>(e: &Error<'a>) -> String {
  if let Some(un_token) = e.unexpected_token {
    let s = format!("Unexpected {}", un_token.value);
    if un_token != e.operation_token && un_token.line != e.operation_token.line {
      let (padding, pointer) =
        util::fmt_token_pointer(un_token.value.get(), un_token.column_number);
      format!(
        "\n{} on line {}, column {}:
  {}
  {}{}",
        s, un_token.line_number, un_token.column_number, un_token.line, padding, pointer
      )
    } else {
      let offset = un_token.column_number
        - e.operation_token.column_number
        - util::string_length(e.operation_token.value.get());
      let (padding, pointer) = util::fmt_token_pointer(un_token.value.get(), offset);
      format!("{}{}\n{}", padding, pointer, s)
    }
  } else {
    "".into()
  }
}

fn fmt_expected<'a>(e: &Error<'a>) -> String {
  if let Some(ex_tokens) = e.expected_token {
    let expected_list = util::fmt_list(ex_tokens, ", ", "or");
    if expected_list.is_empty() {
      "".into()
    } else {
      format!("\nExpected {} instead", expected_list)
    }
  } else {
    "".into()
  }
}

impl fmt::Display for Error<'_> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let op_token = &self.operation_token;
    let (padding, pointer) = util::fmt_token_pointer(op_token.value.get(), op_token.column_number);
    write!(
      f,
      "Error on operation \"{}\" on line {}, column {}:\n  {}\n  {}{}{}{}{}",
      self.operation,
      op_token.line_number,
      op_token.column_number,
      op_token.line,
      padding,
      pointer,
      fmt_unexpected(self),
      fmt_expected(self),
      if let Some(text) = self.details {
        &["\n", &text].join("")
      } else {
        ""
      }
    )
  }
}

// TODO: Create tests
