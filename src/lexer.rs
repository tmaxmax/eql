use crate::util;
use bstr::{ByteSlice, B};
use std::fmt;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TokenValue<'a> {
  Whitespace,
  Word(&'a str),
  Punctuation(&'a str),
  Unknown(&'a str),
}

impl TokenValue<'_> {
  fn get_type_and_value(&self) -> (&str, &str) {
    match *self {
      Whitespace => ("whitespace", ""),
      Word(s) => ("word", s),
      Punctuation(s) => ("punctuation", s),
      Unknown(s) => ("unknown", s),
    }
  }

  pub fn get(&self) -> &str {
    self.get_type_and_value().1
  }
}

impl fmt::Display for TokenValue<'_> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let (t, v) = self.get_type_and_value();
    if v.is_empty() {
      write!(f, "any {} token", t)
    } else {
      write!(f, "{} token \"{}\"", t, v)
    }
  }
}

pub use TokenValue::*;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Token<'a> {
  pub value: TokenValue<'a>,
  pub line: &'a str,
  pub line_number: usize,
  pub column_number: usize,
}

impl<'a> Token<'a> {
  pub fn new(
    value: TokenValue<'a>,
    line: &'a str,
    line_number: usize,
    column_number: usize,
  ) -> Self {
    Token {
      value,
      line,
      line_number,
      column_number,
    }
  }
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Error<'a>(Token<'a>);

impl fmt::Display for Error<'_> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let token = &self.0;
    let (padding, pointer) = util::fmt_token_pointer(token.value.get(), token.column_number);
    write!(
      f,
      "Error on line {}, column {}: {}\n  {}\n  {}{}",
      token.line_number, token.column_number, token.value, token.line, padding, pointer
    )
  }
}

fn is_punctuation(s: &str) -> bool {
  let b = s.as_bytes();
  b.len() == 1 && matches!(b[0], b',' | b'.' | b'!' | b'?')
}

fn get_token<'a>(
  s: &'a str,
  line_number: usize,
  column_number: usize,
  line: &'a str,
) -> Result<Token<'a>, Error<'a>> {
  let token = Token {
    value: match () {
      _ if util::is_whitespace(s) => Whitespace,
      _ if is_punctuation(s) => Punctuation(s),
      _ if util::is_alphabetic(s) => Word(s),
      _ => Unknown(s),
    },
    line,
    line_number,
    column_number,
  };
  match token.value {
    Unknown(_) => Err(Error(token)),
    _ => Ok(token),
  }
}

pub fn lex(s: &str) -> Result<Vec<Token>, Error> {
  B(s.trim())
    .lines_with_terminator()
    .map(|line| unsafe { line.to_str_unchecked() })
    .enumerate()
    .flat_map(|(line_number, line)| {
      let mut column_number = 1;
      line.split_word_bounds().map(move |token| {
        let res = get_token(token, line_number + 1, column_number, line);
        column_number += util::string_length(token);
        res
      })
    })
    .collect()
}

#[cfg(test)]
mod test {
  use super::*;

  #[test]
  fn lex_empty() {
    let source = "";
    let tokens = lex(source).expect("Lex must succeed");
    assert!(tokens.is_empty());
  }

  #[test]
  fn lex_whitespace() {
    let source = " \n\r\n\t";
    let got = lex(source).expect("Lex must succeed");
    assert!(got.is_empty());
  }

  #[test]
  fn lex_words_and_whitespace() {
    let source = "Create\n孫德明";
    let expect = vec![
      Token::new(Word("Create"), "Create\n", 1, 1),
      Token::new(Whitespace, "Create\n", 1, 7),
      Token::new(Word("孫"), "孫德明", 2, 1),
      Token::new(Word("德"), "孫德明", 2, 2),
      Token::new(Word("明"), "孫德明", 2, 3),
    ];
    let got = lex(source).expect("Lex must succeed");
    assert_eq!(got, expect);
  }

  #[test]
  fn lex_punctuation() {
    let source = ".,!?";
    let expect = vec![
      Token::new(Punctuation("."), source, 1, 1),
      Token::new(Punctuation(","), source, 1, 2),
      Token::new(Punctuation("!"), source, 1, 3),
      Token::new(Punctuation("?"), source, 1, 4),
    ];
    let got = lex(source).expect("Lex must succeed");
    assert_eq!(got, expect);
  }

  #[test]
  fn lex_unknown() {
    // All Unicode Character categories that shouldn't be supported by the lexer at all or not on their own (in the case of M categories).
    // The only exceptions are ' ', '.', ',', '!', '?', their support being tested above.
    let chars = vec![
      "\u{0000}", // Cc Control
      "\u{00AD}", // Cf Format
      "\u{0903}", // Mc Spacing Mark
      "\u{0488}", // Me Enclosing Mark
      "\u{0300}", // Mn Nonspacing Mark
      "0",        // Nd Decimal Number
      "\u{16EE}", // Nl Letter Number
      "\u{00B2}", // No Other Number
      "\u{005F}", // Pc Connector Punctuation
      "-",        // Pd Dash Punctuation
      ")",        // Pe Close Punctuation
      "\u{00BB}", // Pf Final Punctuation
      "\u{00AB}", // Pi Initial Punctuation
      "\"",       // Po Other Punctuation
      "$",        // Sc Currency Symbol
      "^",        // Sk Modifier Symbol
      "+",        // Sm Math Symbol
      "\u{00A6}", // So Other Symbol
    ];
    let accepted_count = chars
      .iter()
      .enumerate()
      .filter_map::<(), _>(|(i, c)| match lex(c) {
        Ok(res) => panic!("({}, {:?})", i, res),
        Err(_) => None,
      })
      .count();
    assert_eq!(accepted_count, 0);
  }

  #[test]
  fn error_format() {
    let source = "孫德 12345";
    let expect = r#"Error on line 1, column 4: unknown token "12345"
  孫德 12345
     ^^^^^"#;
    let got = format!("{}", lex(source).expect_err("Lex must fail"));
    assert_eq!(got, expect);
  }

  // TODO: Update tests for empty value token
}
