use std::{ops::Range, fmt::{self, Write}};
use smallvec::SmallVec;
use crate::Parser;

pub struct SPPFItem<'a> {
  // for non-terminal, prod[0] is always lhs, so prod is never empty
  // for terminal, prod is empty, range.start == range.end == position of this token, children is empty
  pub(crate) prod: &'a [u32],
  pub(crate) range: Range<u32>,
  // use index in Vec as pointers; Rc is not suitable, because there may be cycles
  pub(crate) children: SmallVec<[SmallVec<[u32; 4]>; 1]>,
}

pub struct SPPF<'a> {
  pub(crate) parser: &'a Parser<'a>,
  pub(crate) tokens: Vec<u32>,
  pub(crate) items: Vec<SPPFItem<'a>>,
}

impl<'a> SPPF<'a> {
  pub(crate) fn find(&mut self, prod: &'a [u32], range: Range<u32>) -> usize {
    self.items.iter_mut().position(|x| x.prod.as_ptr() == prod.as_ptr() && x.range == range)
      .unwrap_or_else(|| (self.items.len(), self.items.push(SPPFItem { prod, range, children: SmallVec::new() })).0)
  }

  pub fn print_dot(&self) -> String {
    let ref id2token = self.parser.id2token();
    let mut s = "digraph g {\n".to_owned();
    let mut circles = 0;
    for (idx, item) in self.items.iter().enumerate() {
      if !item.prod.is_empty() {
        let _ = writeln!(s, r#"{}[shape=rect, label="{}, {:?}"]"#, idx, ShowProd(id2token, item.prod), item.range);
        for ch in &item.children {
          if ch.len() == 1 { let _ = writeln!(s, "{} -> {}", idx, ch[0]); } else {
            let _ = writeln!(s, "{} -> circle{}", idx, circles);
            let _ = writeln!(s, r#"circle{}[shape=circle, label="", width=0.2]"#, circles);
            for t in ch { let _ = writeln!(s, "circle{} -> {}", circles, t); }
            circles += 1;
          }
        }
      } else {
        let _ = writeln!(s, r#"{}[shape=circle, label="{}"]"#, idx, id2token[self.tokens[item.range.start as usize] as usize]);
      }
    }
    s.push('}');
    s
  }
}

pub(crate) struct ShowProd<'a>(pub &'a [&'a str], pub &'a [u32]);

impl fmt::Display for ShowProd<'_> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let (lhs, rhs) = (self.1[0], &self.1[1..]);
    write!(f, "{} ->", self.0[lhs as usize])?;
    for &r in rhs { write!(f, " {}", self.0[r as usize])?; }
    Ok(())
  }
}