use earley::Parser;

fn main() {
  let p = Parser::from_rules(r#"
Sum     -> Sum     + Product
Sum     -> Sum     - Product
Sum     -> Product
Product -> Product * Factor
Product -> Product / Factor
Product -> Factor
Factor  -> ( Sum )
Factor  -> Number
"#).unwrap();
  let (_, sppf) = p.parse(["Number", "+", "(", "Number", "*", "Number", "-", "Number", ")"].iter().copied(), "Sum").unwrap();

  let mut it = sppf.iter();
  let mut cnt = 0;
  while let Some(tree) = it.next() {
    std::fs::write(format!("{}.dot", cnt), format!("{}", tree)).unwrap();
    cnt += 1;
  }
}
