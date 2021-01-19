extern crate eql;

use eql::lexer::last_token_value;
use std::io::{self, BufRead};

fn get_input(mut handle: impl BufRead) -> io::Result<String> {
  let mut buf = String::new();
  let mut total_read = 0;

  loop {
    let read = handle.read_line(&mut buf)?;
    total_read += read;
    if let Some(tail) = last_token_value(&buf[total_read - read..total_read].trim_end()) {
      // FIXME: Break on actual terminators
      if !matches!(tail, eql::lexer::Word(_)) {
        break;
      }
    }
  }

  Ok(buf)
}

fn main() -> io::Result<()> {
  let stdin = io::stdin();

  loop {
    let source = get_input(stdin.lock())?;

    let operations = match eql::lex_parse(&source) {
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
