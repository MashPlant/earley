#![feature(iter_partition_in_place)]

pub mod format;
pub mod sppf;

use std::{collections::HashMap, ops::Range};
use crate::{sppf::*, format::*};
use smallvec::{smallvec, SmallVec};

#[derive(Eq, Copy, Clone)]
pub struct Item<'a> {
  pub prod: &'a [u32],
  pub dot: u32,
  pub orig: u32,
}

// the dp chart type
pub type Chart<'a> = Vec<Vec<Item<'a>>>;

// compare production array using pointer, because each prod is considered unique
impl PartialEq for Item<'_> {
  fn eq(&self, other: &Self) -> bool {
    self.prod.as_ptr() == other.prod.as_ptr() && self.dot == other.dot && self.orig == other.orig
  }
}

pub struct Parser<'a> {
  // prod[0] == lhs
  pub(crate) prods: Vec<SmallVec<[u32; 4]>>,
  // 0..#non-terminals is non-terminal, #non-terminals..#tokens is terminal
  pub(crate) tokens: HashMap<&'a str, u32>,
  // nullable.len() == #non-terminals, which is recorded **nowhere else**
  pub(crate) nullable: Vec<bool>,
}

#[inline(always)]
unsafe fn split1(x: &[u32]) -> (u32, &[u32]) {
  debug_assert!(!x.is_empty());
  (*x.get_unchecked(0), x.get_unchecked(1..))
}

// according to document, `slice::get_unchecked` can't perform `debug_assert`
#[inline(always)]
unsafe fn get<T>(x: &[T], i: u32) -> &T {
  debug_assert!(i < x.len() as u32);
  x.get_unchecked(i as usize)
}

#[inline(always)]
unsafe fn get_mut<T>(x: &mut [T], i: u32) -> &mut T {
  debug_assert!(i < x.len() as u32);
  x.get_unchecked_mut(i as usize)
}

// getters
impl<'a> Parser<'a> {
  #[inline(always)]
  pub fn prods(&self) -> &Vec<SmallVec<[u32; 4]>> { &self.prods }
  #[inline(always)]
  pub fn tokens(&self) -> &HashMap<&'a str, u32> { &self.tokens }
  #[inline(always)]
  pub fn nullable(&self) -> &Vec<bool> { &self.nullable }
  #[inline(always)]
  pub fn token_id(&self, t: &str) -> Option<u32> { self.tokens.get(t).copied() }
  #[inline(always)]
  pub fn terminal_id(&self, t: &str) -> Option<u32> { self.token_id(t).filter(|&x| x >= self.nullable.len() as u32) }
  #[inline(always)]
  pub fn non_terminal_id(&self, t: &str) -> Option<u32> { self.token_id(t).filter(|&x| x < self.nullable.len() as u32) }

  pub fn id2token(&self) -> Vec<&str> {
    unsafe {
      let mut ret = Vec::with_capacity(self.tokens.len());
      ret.set_len(self.tokens.len());
      for (&k, &v) in &self.tokens { *get_mut(&mut ret, v) = k; }
      ret
    }
  }
}

impl Parser<'_> {
  pub fn from_rules(rules: &str) -> Result<Parser, ParseRulesError> {
    let (mut prods, mut tokens) = (Vec::new(), HashMap::new());
    let lines = rules.lines().enumerate().filter(|(_, rule)| {
      let rule = rule.trim();
      !rule.is_empty() && !rule.starts_with('#') // comment line start with #
    });
    for (line, rule) in lines.clone() {
      let mut sp = rule.split_whitespace();
      let lhs = sp.next().ok_or(ParseRulesError(line + 1))?;
      let id = tokens.len() as u32;
      prods.push(smallvec![*tokens.entry(lhs).or_insert(id)]);
    }
    if prods.is_empty() { return Err(ParseRulesError(0)); }
    let mut nullable = vec![false; tokens.len()];
    for ((line, rule), prod) in lines.zip(prods.iter_mut()) {
      let mut sp = rule.split_whitespace();
      match sp.nth(1) { Some("->") => {} _ => return Err(ParseRulesError(line + 1)) };
      for rhs in sp {
        let id = tokens.len() as u32;
        prod.push(*tokens.entry(rhs).or_insert(id));
      }
    }
    unsafe { // compute nullable set
      loop {
        let mut changed = false;
        for p in &prods {
          let (lhs, rhs) = split1(p);
          if *get(&nullable, lhs) { continue; }
          if rhs.iter().all(|&r| nullable.get(r as usize).copied().unwrap_or(false)) {
            *get_mut(&mut nullable, lhs) = true;
            changed = true;
          }
        }
        if !changed { break; }
      }
    }
    Ok(Parser { prods, tokens, nullable })
  }

  // return Err only when input/start contains undefined terminal/non-terminal
  // so returning Ok doesn't mean the parse succeeds
  // `SPPF::next` will return a sequence of possible trees, so the parse succeeds when it returns anything
  pub fn parse<'a, 'b>(&'a self, input: impl IntoIterator<Item=&'b str>, start: &'b str) -> Result<(Chart<'a>, SPPF), ParseError<'b>> {
    let mut tokens = Vec::new();
    for t in input {
      tokens.push(self.terminal_id(t).ok_or(ParseError(t))?);
    }
    let start = self.non_terminal_id(start).ok_or(ParseError(start))?;
    Ok(unsafe { self.do_parse(tokens, start) })
  }

  unsafe fn do_parse(&self, tokens: Vec<u32>, start: u32) -> (Chart, SPPF) {
    let nt_num = self.nullable.len() as u32;
    let mut sets = vec![Vec::new(); tokens.len() + 1];
    let p = sets.as_mut_ptr();
    for prod in &self.prods {
      if *get(prod, 0) == start {
        // dot = 1 is start position, because [0] is lhs
        (*p).push(Item { prod, dot: 1, orig: 0 });
      }
    }
    fn set_add<'a>(v: &mut Vec<Item<'a>>, x: Item<'a>) {
      if !v.contains(&x) { v.push(x); }
    }
    for i in 0..tokens.len() + 1 {
      let si = &mut *p.add(i);
      let token = tokens.get(i).copied(); // when i == tokens.len(), token is None
      let mut j = 0;
      while let Some(&Item { prod, dot, orig }) = si.get(j) {
        if let Some(&nxt) = prod.get(dot as usize) {
          if nxt < nt_num { // is a non-terminal
            for prod in &self.prods { // PREDICATE step
              if *get(prod, 0) == nxt {
                set_add(si, Item { prod, dot: 1, orig: i as u32 });
              }
            }
            // this is a modification (or say correction) to the original earley parser
            // a nullable non-terminal A can be advanced without seeing A -> string. during PREDICATE step
            // for more detail, see https://courses.engr.illinois.edu/cs421/sp2012/project/PracticalEarleyParsing.pdf
            if *get(&self.nullable, nxt) {
              set_add(si, Item { prod, dot: dot + 1, orig });
            }
          } else if token == Some(nxt) { // is a terminal, SCAN step
            // this never causes duplication, so no need to check `contains`
            // when i == tokens.len(), i + 1 seems to be out of range of `sets`, but it will never really causes an error
            // because when i == tokens.len(), token is None, so this branch is never entered
            (*p.add(i + 1)).push(Item { prod, dot: dot + 1, orig });
          }
        } else { // COMPLETE step
          let lhs = *get(&prod, 0);
          // caution: it is possible that i == orig, so can't use iterator
          // the semantics of this step is just iterating over those already in the sets
          let orig = &*p.add(orig as usize);
          for idx in 0..orig.len() {
            let &Item { prod, dot, orig } = orig.get_unchecked(idx);
            if prod.get(dot as usize) == Some(&lhs) {
              set_add(si, Item { prod, dot: dot + 1, orig });
            }
          }
        }
        j += 1;
      }
    }

    let mut complete = vec![Vec::new(); tokens.len() + 1];
    for (idx, set) in sets.iter().enumerate() {
      for &Item { prod, dot, orig } in set {
        if prod.len() as u32 == dot {
          get_mut(&mut complete, orig).push(Item { prod, dot, orig: idx as u32 });
        }
      }
    }

    struct DfsCtx<'a> {
      range: Range<u32>,
      nt_num: u32,
      complete: Vec<Vec<Item<'a>>>,
      prod: &'a [u32],
      sppf: SPPF<'a>,
      path: Vec<(u32, u32)>,
    }

    impl DfsCtx<'_> {
      unsafe fn dfs(&mut self, cur: usize, start: usize) {
        if let Some(&x) = self.prod.get(cur) {
          if x < self.nt_num {
            for (idx, it) in (*self.complete.as_ptr().add(start)).iter().enumerate() {
              if *get(it.prod, 0) == x {
                self.path.push((start as u32, idx as u32));
                self.dfs(cur + 1, it.orig as usize);
                self.path.pop();
              }
            }
          } else if self.sppf.tokens.get(start) == Some(&x) {
            self.path.push((!0, start as u32));
            self.dfs(cur + 1, start + 1);
            self.path.pop();
          } // else: this branch of dfs fails
        } else if !self.path.is_empty() { // finished
          let node = self.sppf.find(self.prod, self.range.clone());
          let mut ch = SmallVec::new();
          let mut cur = self.range.start;
          for &(state, idx) in &self.path {
            if state != !0 {
              let &Item { prod, orig, .. } = get(get::<Vec<_>>(&self.complete, state), idx);
              ch.push(self.sppf.find(prod, cur..orig));
              cur = orig;
            } else {
              ch.push(self.sppf.find(&[], cur..cur));
              cur += 1;
            }
          }
          if cur == self.range.end {
            get_mut(&mut self.sppf.nodes, node).children.push(ch);
          }
        }
      }
    }

    let mut ctx = DfsCtx { range: 0..tokens.len() as u32, nt_num, complete, prod: &[], path: Vec::new(), sppf: SPPF { parser: self, start, tokens, nodes: Vec::new() } };
    for &Item { prod, orig, .. } in &*ctx.complete.as_ptr() {
      if *get(prod, 0) == start && orig == ctx.sppf.tokens.len() as u32 {
        ctx.prod = prod;
        ctx.dfs(1, 0);
      }
    }
    let mut i = 0;
    while let Some(SPPFNode { prod, range, children }) = ctx.sppf.nodes.get(i) {
      if children.is_empty() && !prod.is_empty() { // not visited && non-terminal
        let range = range.clone();
        ctx.range = range.clone();
        ctx.prod = prod;
        ctx.dfs(1, range.start as usize);
      }
      i += 1;
    }

    let nodes = ctx.sppf.nodes.as_ptr();
    // reorder, so that the children containing no production are in the front
    // in this way the dfs on sppf can generate trees from short to tall, instead of insisting on a infinite tall tree
    for n in &mut ctx.sppf.nodes {
      if n.children.len() >= 1 { // this if is not necessary, just save some work
        n.children.iter_mut().partition_in_place(|ch| ch.iter().all(|&x| (*nodes.add(x as usize)).children.is_empty()));
      }
    }
    (sets, ctx.sppf)
  }
}