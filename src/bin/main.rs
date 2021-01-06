use eql::lexer;

fn main() {
  let res = lexer::lex("\r\nCreate Science! Add Michael, аника аншоы and 孫德明 to Science.\nGet Science. Get Engineering?    ");
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
