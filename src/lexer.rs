use crate::util;
use bstr::{ByteSlice, B};
use std::{default, fmt};
use unicode_segmentation::UnicodeSegmentation;

/// **TokenValue** holds the value type of the token and a reference to the string slice that represents it.
/// The characters that make up a value type are defined by the Unicode Standard.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum TokenValue<'a> {
  /// **Whitespace** represents any kind of whitespace. It doesn't hold the actual whitespace value
  /// because EQL treats all whitespace as a Unicode Space (SP) " " character.
  Whitespace,
  /// **Word** represents a group of letters.
  Word(&'a str),
  /// **Punctuation** represents a single punctuation character.
  Punctuation(&'a str),
  /// **Unknown** represents all the Unicode characters that are not valid in EQL source code.
  /// The value held is used to display error messages.
  Unknown(&'a str),
}

impl TokenValue<'_> {
  /// **get_type_and_value** returns a string representation of the value type and a reference to the string slice
  /// that represents the token's value.
  fn get_type_and_value(&self) -> (&str, &str) {
    match *self {
      Whitespace => ("whitespace", " "),
      Word(s) => ("word", s),
      Punctuation(s) => ("punctuation", s),
      Unknown(s) => ("unknown", s),
    }
  }

  /// **get** returns a reference to the string slice that represents the token's value.
  pub fn get(&self) -> &str {
    self.get_type_and_value().1
  }
}

impl fmt::Display for TokenValue<'_> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let (t, v) = self.get_type_and_value();
    if v.is_empty() || matches!(*self, Whitespace) {
      write!(f, "any {} token", t)
    } else {
      write!(f, "{} token \"{}\"", t, v)
    }
  }
}

impl default::Default for TokenValue<'_> {
  fn default() -> Self {
    Unknown("")
  }
}

pub use TokenValue::*;

/// **Token** represents a single valid EQL lexical token. It holds its value, a reference to the
/// line it's found on, the line number it's found on, and the column it starts at.
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

/// **Error** represents an invalid token. It implements the fmt::Display trait so a useful error message
/// can be generated.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Error<'a>(Token<'a>);

impl fmt::Display for Error<'_> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let token = &self.0;
    let (padding, pointer) = util::fmt_token_pointer(token.value.get(), token.column_number);
    write!(
      f,
      "Error on line {}, column {}: {}\n  {}\n  {}{}",
      token.line_number,
      token.column_number,
      token.value,
      token.line.trim_end(),
      padding,
      pointer
    )
  }
}

impl std::error::Error for Error<'_> {}

/// **is_punctuation** checks if the slice is valid EQL punctuation.
fn is_punctuation(s: &str) -> bool {
  let b = s.as_bytes();
  b.len() == 1 && matches!(b[0], b',' | b'.' | b'!' | b'?')
}

/// **get_token_value** returns a new TokenValue from a string reference containing the token type
/// that string represents and the respective value.
fn get_token_value(s: &str) -> TokenValue {
  match () {
    _ if util::is_whitespace(s) => Whitespace,
    _ if is_punctuation(s) => Punctuation(s),
    _ if util::is_alphabetic(s) => Word(s),
    _ => Unknown(s),
  }
}

/// **get_token** creates a new Token from the given parameters and returns it, if its value is not
/// Unknown, or wraps it in an Error and returns it.
fn get_token<'a>(
  s: &'a str,
  line_number: usize,
  column_number: usize,
  line: &'a str,
) -> Result<Token<'a>, Error<'a>> {
  let token = Token {
    value: get_token_value(s),
    line,
    line_number,
    column_number,
  };
  match token.value {
    Unknown(_) => Err(Error(token)),
    _ => Ok(token),
  }
}

#[allow(clippy::tabs_in_doc_comments)]
/// **lex** lexes a string and returns a vector of tokens, if the string is valid EQL
/// source code, or the first error encountered.
///
/// # Arguments:
/// * `s` - The string representing the source code
///
/// # Examples:
///
/// ```
/// use eql::lex;
/// let source = r"
/// Add
/// 	Andrew,
/// 	立顯榮朝士,
/// 	John
/// to HR and PR.";
/// let tokens = lex(source).unwrap();
/// let source_final: String = tokens.iter().map(|t| t.value.get()).collect();
/// assert_eq!(source_final, " Add  Andrew,  立顯榮朝士,  John to HR and PR.");
/// ```
///
/// If you want to also parse the source code, there is the convenience function `lex_parse`:
///
/// ```
/// use eql::lex_parse;
/// use eql::operation::OperationShow;
/// let source = "Show HR?";
/// let ops = lex_parse(source).unwrap();
/// assert_eq!(ops[0], OperationShow::new(vec!["HR".into()], true).into());
/// ```
///
/// **Note:** Make sure that `s` is a valid UTF-8 string, otherwise calling this function
/// is undefined behavior.
pub fn lex(s: &str) -> Result<Vec<Token>, Error> {
  B(s)
    // Line terminaors are required as EQL treats newlines as whitespace
    .lines_with_terminator()
    // SAFETY: the caller should ensure that the string is valid
    .map(|line| unsafe { line.to_str_unchecked() })
    .enumerate()
    .flat_map(|(line_number, line)| {
      let mut column_number = 1;
      line.split_word_bounds().map(move |token| {
        let res = get_token(token, line_number + 1, column_number, line);
        // Can't use len() here because a column is represented by a single grapheme.
        column_number += util::string_length(token);
        res
      })
    })
    .collect()
}

/// **last_token_value** returns the last token from the string, if not empty.
pub fn last_token_value(s: &str) -> Option<TokenValue> {
  s.split_word_bounds().map(get_token_value).rev().next()
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
    let expect = vec![
      Token::new(Whitespace, " \n", 1, 1),
      Token::new(Whitespace, " \n", 1, 2),
      Token::new(Whitespace, "\r\n", 2, 1),
      Token::new(Whitespace, "\t", 3, 1),
    ];
    assert_eq!(got, expect);
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
    chars.into_iter().map(lex).for_each(|r| {
      r.unwrap_err();
    });
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

  // TODO: More tests (formatting of tokens and error messages)
}
