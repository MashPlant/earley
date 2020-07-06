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
  let sppf = p.parse(["Number", "+", "(", "Number", "*", "Number", "-", "Number", ")"].iter().copied(), "Sum").unwrap();
  println!("{}", sppf.print_dot());
}
