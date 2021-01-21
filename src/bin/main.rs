extern crate eql;

use eql::lexer::last_token_value;
use std::io::{self, BufRead};

fn get_input(mut handle: impl BufRead, buf: &mut String) -> io::Result<()> {
  let mut total_read = 0;
  buf.clear();

  loop {
    let read = handle.read_line(buf)?;
    total_read += read;
    if let Some(tail) = last_token_value(&buf[total_read - read..].trim_end()) {
      // FIXME: Break on actual terminators
      if !matches!(tail, eql::lexer::Word(_)) {
        break;
      }
    }
  }

  Ok(())
}

fn main() -> io::Result<()> {
  let stdin = io::stdin();
  let mut buffer = String::new();

  loop {
    get_input(stdin.lock(), &mut buffer)?;

    let operations = match eql::lex_parse(&buffer) {
      Ok(ops) => ops,
      Err(e) => {
        eprintln!("{}", e);
        continue;
      }
    };
    for (i, op) in operations.iter().enumerate() {
      println!("{}: {:?}", i, op);
    }
  }
}
