use wasm_bindgen::prelude::*;
use earley::{Parser, Item};
use std::fmt::Write;

#[global_allocator]
static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;

#[wasm_bindgen]
pub fn parse(rules: &str, input: &str, start: &str, kind: &str, n: u32) -> Result<String, JsValue> {
  let parser = Parser::from_rules(rules).map_err(|e| JsValue::from(&format!("{}", e)))?;
  let (chart, sppf) = parser.parse(input.split_ascii_whitespace(), start).map_err(|e| JsValue::from(&format!("{}", e)))?;
  match kind {
    "chart" => {
      let id2token = parser.id2token();
      let mut s = String::new();
      for (idx, set) in chart.iter().enumerate() {
        let _ = writeln!(s, "======{}======", idx);
        for &Item { prod, dot, orig } in set {
          let (lhs, rhs) = (prod[0], &prod[1..]);
          let _ = write!(s, "({} ->", id2token[lhs as usize]);
          for (idx, &r) in rhs.iter().enumerate() {
            let _ = write!(s, "{}{}", if idx + 1 == dot as usize { '.' } else { ' ' }, id2token[r as usize]);
          }
          let _ = writeln!(s, "{}, {})", if rhs.len() + 1 == dot as usize { "." } else { "" }, orig);
        }
      }
      Ok(s)
    }
    "sppf" => Ok(format!("{}", sppf)),
    "tree" => {
      let mut s = "digraph g {\n".to_owned();
      let mut iter = sppf.iter();
      let mut idx = 1;
      while let (Some(tree), true) = (iter.next(), idx <= n) {
        let _ = writeln!(s, r#"subgraph cluster_{} {{"#, idx);
        let _ = writeln!(s, r#"label = "tree #{}""#, idx);
        let tree = format!("{}", tree);
        for line in tree.lines().skip(1) {
          if line.contains("label") {
            let _ = writeln!(s, "t{}_{}", idx, &line[2..]);
          } else if let Some(arrow) = line.find(" -> ") {
            let _ = writeln!(s, "t{}_{} -> t{}_{}", idx, &line[2..arrow], idx, &line[arrow + 4..]);
          }
        }
        s.push('}');
        idx += 1;
      }
      s.push('}');
      Ok(s)
    }
    _ => Err(JsValue::from(r#"invalid kind, expect one of "chart", "sppf", "tree""#))
  }
}