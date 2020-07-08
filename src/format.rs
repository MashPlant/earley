use std::fmt;
use crate::{split1, get, Parser, sppf::*};

struct ShowProd<'a>(pub &'a [&'a str], pub &'a [u32]);

impl fmt::Display for ShowProd<'_> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    unsafe {
      let (lhs, rhs) = split1(self.1);
      write!(f, "{} ->", get(self.0, lhs))?;
      for &r in rhs { write!(f, " {}", get(self.0, r))?; }
      Ok(())
    }
  }
}

impl fmt::Display for Parser<'_> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let id2token = self.id2token();
    for prod in &self.prods { writeln!(f, "{}", ShowProd(&id2token, prod))?; }
    Ok(())
  }
}

impl fmt::Display for SPPF<'_> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    writeln!(f, "digraph g {{")?;
    let id2token = self.parser.id2token();
    let mut circles = 0;
    for (idx, SPPFNode { prod, range, children }) in self.nodes.iter().enumerate() {
      if !prod.is_empty() {
        writeln!(f, r#"  {}[shape=rect, label="{}, {:?}"]"#, idx, ShowProd(&id2token, prod), range)?;
        for ch in children {
          if ch.len() == 1 { writeln!(f, "  {} -> {}", idx, ch[0])?; } else {
            writeln!(f, "  {} -> circle{}", idx, circles)?;
            writeln!(f, r#"  circle{}[shape=circle, label="", width=0.2]"#, circles)?;
            for t in ch { writeln!(f, "  circle{} -> {}", circles, t)?; }
            circles += 1;
          }
        }
      } else {
        writeln!(f, r#"  {}[shape=circle, label="{}"]"#, idx, unsafe { get(&id2token, *get(&self.tokens, range.start)) })?;
      }
    }
    writeln!(f, "}}")
  }
}

#[derive(Copy, Eq, PartialEq, Clone, Debug)]
pub struct ParseRulesError(pub(crate) usize);

impl fmt::Display for ParseRulesError {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    if self.0 == 0 { write!(f, "empty rules") } else { write!(f, "rules line {} is not in the form of \"lhs -> rhs1 rhs2 ...\"", self.0) }
  }
}

#[derive(Copy, Eq, PartialEq, Clone, Debug)]
pub struct ParseError<'a>(pub(crate) &'a str);

impl fmt::Display for ParseError<'_> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    write!(f, "no such terminal or non-terminal: {}", self.0)
  }
}