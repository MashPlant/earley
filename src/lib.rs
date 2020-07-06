pub mod sppf;

use std::{collections::HashMap, ops::Range, fmt};
use crate::sppf::{SPPF, ShowProd};
use smallvec::SmallVec;

#[derive(Eq, Copy, Clone)]
pub struct Item<'a> {
  prod: &'a [u32],
  dot: u32,
  orig: u32,
}

// compare production array using pointer, because their contents are unique
impl PartialEq for Item<'_> {
  fn eq(&self, other: &Self) -> bool {
    self.prod.as_ptr() == other.prod.as_ptr() && self.dot == other.dot && self.orig == other.orig
  }
}

pub struct Parser<'a> {
  // prod[0] == lhs
  prods: Vec<Vec<u32>>,
  tokens: HashMap<&'a str, u32>,
  nt_num: u32,
}

impl fmt::Display for Parser<'_> {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    let id2token = self.id2token();
    for prod in &self.prods { writeln!(f, "{}", ShowProd(&id2token, prod))?; }
    Ok(())
  }
}

impl Parser<'_> {
  pub fn from_rules(rules: &str) -> Option<Parser> {
    let (mut prods, mut tokens) = (Vec::new(), HashMap::new());
    let lines = rules.lines().filter(|rule| !rule.trim().is_empty());
    for rule in lines.clone() {
      let mut sp = rule.split_whitespace();
      let lhs = sp.next()?;
      let id = tokens.len() as u32;
      prods.push(vec![*tokens.entry(lhs).or_insert(id)]);
    }
    let nt_num = tokens.len() as u32;
    for (rule, prod) in lines.zip(prods.iter_mut()) {
      let mut sp = rule.split_whitespace();
      match sp.nth(1) { Some("->") => {} _ => return None };
      for rhs in sp {
        let id = tokens.len() as u32;
        prod.push(*tokens.entry(rhs).or_insert(id));
      }
    }
    Some(Parser { prods, tokens, nt_num })
  }

  pub fn id2token(&self) -> Vec<&str> {
    let mut ret = vec![""; self.tokens.len()];
    for (&k, &v) in &self.tokens { ret[v as usize] = k; }
    ret
  }

  pub fn parse<'a>(&self, string: impl IntoIterator<Item=&'a str>, start: &str) -> Result<SPPF, &'static str> {
    let mut tokens = Vec::new();
    for t in string {
      tokens.push(self.tokens.get(t).copied().filter(|&x| x >= self.nt_num).ok_or("no such terminal")?);
    }
    let start = self.tokens.get(start).copied().filter(|&x| x < self.nt_num).ok_or("no such non-terminal")?;
    Ok(unsafe { self.do_parse(tokens, start) })
  }

  unsafe fn do_parse(&self, tokens: Vec<u32>, start: u32) -> SPPF {
    let mut sets = vec![Vec::new(); tokens.len() + 1];
    let p = sets.as_mut_ptr();
    for prod in &self.prods {
      if *prod.get_unchecked(0) == start {
        // dot = 1 is start position, because [0] is lhs
        (*p).push(Item { prod, dot: 1, orig: 0 });
      }
    }
    macro_rules! set_add { ($v: expr, $x: expr) => { if !$v.contains(&$x) { $v.push($x); } }; }
    for i in 0..tokens.len() + 1 {
      let si = &mut *p.add(i);
      let token = tokens.get(i).copied(); // when i == tokens.len(), token is None
      let mut j = 0;
      while let Some(&Item { prod, dot, orig }) = si.get(j) {
        if let Some(&nxt) = prod.get(dot as usize) {
          if nxt < self.nt_num { // is a non-terminal
            for prod in &self.prods { // PREDICATE step
              if *prod.get_unchecked(0) == nxt {
                set_add!(si, Item { prod, dot: 1, orig: i as u32 });
              }
            }
          } else { // is a terminal
            if token == Some(nxt) { // SCAN step
              // this never causes duplication, so no need to check `contains`
              // when i == tokens.len(), i + 1 seems to be out of range of `sets`, but it will never really causes an error
              // because when i == tokens.len(), token is None, so this branch is never entered
              (&mut *p.add(i + 1)).push(Item { prod, dot: dot + 1, orig });
            }
          }
        } else { // COMPLETE step
          let lhs = *prod.get_unchecked(0);
          for &Item { prod, dot, orig } in &*p.add(orig as usize) {
            if prod.get(dot as usize).copied() == Some(lhs) {
              set_add!(si, Item { prod, dot: dot + 1, orig });
            }
          }
        }
        j += 1;
      }
    }

    let mut completed_sets = vec![Vec::new(); tokens.len() + 1];
    for (idx, set) in sets.iter().enumerate() {
      for &Item { prod, dot, orig } in set {
        if prod.len() == dot as usize {
          completed_sets.get_unchecked_mut(orig as usize).push(Item { prod, dot, orig: idx as u32 });
        }
      }
    }

    struct DfsCtx<'a> {
      range: Range<u32>,
      nt_num: u32,
      completed_sets: *const Vec<Item<'a>>,
      prod: &'a [u32],
      sppf: SPPF<'a>,
      path: Vec<(u32, u32)>,
    }

    impl DfsCtx<'_> {
      unsafe fn dfs(&mut self, cur: usize, start: usize) {
        if let Some(&x) = self.prod.get(cur) {
          if x < self.nt_num {
            for (idx, it) in (*self.completed_sets.add(start)).iter().enumerate() {
              if *it.prod.get_unchecked(0) == x {
                self.path.push((start as u32, idx as u32));
                self.dfs(cur + 1, it.orig as usize);
                self.path.pop();
              }
            }
          } else if self.sppf.tokens.get(start).copied() == Some(x) {
            self.path.push((!0, start as u32));
            self.dfs(cur + 1, start + 1);
            self.path.pop();
          }
        } else if !self.path.is_empty() { // finished
          let item = self.sppf.find(self.prod, self.range.clone());
          let mut ch = SmallVec::new();
          let mut cur = self.range.start;
          for &(state, idx) in &self.path {
            if state != !0 {
              let &Item { prod, orig, .. } = (*self.completed_sets.add(state as usize)).get_unchecked(idx as usize);
              ch.push(self.sppf.find(prod, cur..orig) as u32);
              cur = orig;
            } else {
              ch.push(self.sppf.find(&[], cur..cur) as u32);
              cur += 1;
            }
          }
          if cur == self.range.end {
            self.sppf.items.get_unchecked_mut(item).children.push(ch);
          }
        }
      }
    }

    let mut ctx = DfsCtx {
      range: 0..tokens.len() as u32,
      nt_num: self.nt_num,
      completed_sets: completed_sets.as_ptr(),
      prod: &[],
      path: Vec::new(),
      sppf: SPPF { parser: self, tokens, items: Vec::new() },
    };
    for &Item { prod, orig, .. } in completed_sets.get_unchecked(0) {
      if *prod.get_unchecked(0) == start && orig as usize == ctx.sppf.tokens.len() {
        ctx.prod = prod;
        ctx.dfs(1, 0);
      }
    }
    let mut i = 0;
    while let Some(x) = ctx.sppf.items.get(i) {
      if x.children.is_empty() && !x.prod.is_empty() { // not visited && non-terminal
        let range = x.range.clone();
        ctx.range = range.clone();
        ctx.prod = x.prod;
        ctx.dfs(1, range.start as usize);
      }
      i += 1;
    }
    ctx.sppf
  }
}