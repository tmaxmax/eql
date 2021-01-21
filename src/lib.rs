pub mod lexer;
pub mod operation;
pub mod parser;
mod util;

pub use lexer::lex;
pub use operation::Operation;
pub use parser::parse;

pub fn lex_parse<'a>(s: &'a str) -> Result<Vec<Operation>, Box<dyn std::error::Error + 'a>> {
  Ok(parse(lex(s)?)?)
}

// #[cfg(test)]
// mod tests {}
