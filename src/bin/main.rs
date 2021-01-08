use eql::lexer;

fn main() {
  let res = lexer::lex("src/parser.rs:34:1");
  res.map_or_else(
    |err| println!("{}", err),
    |tokens| {
      tokens
        .into_iter()
        .map(|t| t.value)
        .for_each(|t| println!("{}", t))
    },
  )
}
