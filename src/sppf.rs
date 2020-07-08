use std::{ops::Range, fmt::{self, Write}};
use smallvec::{smallvec, SmallVec};
use crate::{Parser, split1};

pub struct SPPFNode<'a> {
  // for non-terminal, prod[0] is always lhs, so prod is never empty
  // for terminal, prod is empty, range.start == range.end == position of this token, children is empty
  pub(crate) prod: &'a [u32],
  pub(crate) range: Range<u32>,
  // use index in Vec as pointers; Rc is not suitable, because there may be cycles
  pub(crate) children: SmallVec<[SmallVec<[u32; 4]>; 1]>,
}

pub struct SPPF<'a> {
  pub(crate) parser: &'a Parser<'a>,
  // `start` is the id of the start non-terminal
  pub(crate) start: u32,
  pub(crate) tokens: Vec<u32>,
  pub(crate) nodes: Vec<SPPFNode<'a>>,
}

impl<'a> SPPF<'a> {
  pub(crate) fn find(&mut self, prod: &'a [u32], range: Range<u32>) -> usize {
    self.nodes.iter_mut().position(|x| x.prod.as_ptr() == prod.as_ptr() && x.range == range)
      .unwrap_or_else(|| (self.nodes.len(), self.nodes.push(SPPFNode { prod, range, children: SmallVec::new() })).0)
  }

  pub fn print_dot(&self) -> String {
    let ref id2token = self.parser.id2token();
    let mut s = "digraph g {\n".to_owned();
    let mut circles = 0;
    for (idx, SPPFNode { prod, range, children }) in self.nodes.iter().enumerate() {
      if !prod.is_empty() {
        let _ = writeln!(s, r#"{}[shape=rect, label="{}, {:?}"]"#, idx, ShowProd(id2token, prod), range);
        for ch in children {
          if ch.len() == 1 { let _ = writeln!(s, "{} -> {}", idx, ch[0]); } else {
            let _ = writeln!(s, "{} -> circle{}", idx, circles);
            let _ = writeln!(s, r#"circle{}[shape=circle, label="", width=0.2]"#, circles);
            for t in ch { let _ = writeln!(s, "circle{} -> {}", circles, t); }
            circles += 1;
          }
        }
      } else {
        let _ = writeln!(s, r#"{}[shape=circle, label="{}"]"#, idx, id2token[self.tokens[range.start as usize] as usize]);
      }
    }
    s.push('}');
    s
  }

  pub fn iter(&self) -> Iter {
    let mut stk = Vec::new();
    for (idx, SPPFNode { prod, range, .. }) in self.nodes.iter().enumerate() {
      if prod.get(0) == Some(&self.start) && range.end == self.tokens.len() as u32 {
        stk.push(State::_0 { node: idx, pos: (!0, &[]) });
      }
    }
    let tree = SPPF { parser: self.parser, start: self.start, tokens: self.tokens.clone(), nodes: Vec::new() };
    Iter { sppf: self, stk, poses: Vec::new(), tree }
  }
}

enum State<'a> {
  _0 { node: usize, pos: (usize, &'a [u32]) },
  _1,
  _2 { node: usize, cur: usize, ch_idx: usize },
}

pub struct Iter<'a> {
  sppf: &'a SPPF<'a>,
  stk: Vec<State<'a>>,
  poses: Vec<(usize, &'a [u32])>,
  tree: SPPF<'a>,
}

impl<'a> Iter<'a> {
  // `Iter::next` will return a (maybe infinite) sequence of trees before it return None
  // each tree is represented by `SPPF`, with `children.len() <= 1` guaranteed (so that it is a determined tree)
  //
  // this function may be somewhat hard to understand, this is because it is transformed from a recursive function
  // to a non-recursive one, based on heap-allocated stack and state machine
  // the pseudo-code of the original recursive function is like:
  // (implicit `self`, omitting type cast, use safe functions)
  //
  // // `sppf` is the original sppf, the `children` in it may contain multiple choices
  // // `tree` is the target tree, we want to explore all the possibility in `sppf`, and generate concrete trees
  // // `node` is the index of the node that we are going to explore in `sppf`
  // // `pos` is `(index of parent in `tree`, indices of siblings that is right to itself in `sppf`)`
  // // `poses` is a stack of `pos`, when we reach a leaf in `sppf`, we lookup from top to bottom in `poses`
  // // and find the first one with `siblings` not empty, and go to explore the first sibling
  // fn dfs(node: usize, pos: (usize, &[u32]))
  //   let { prod, range, children } = sppf[node]
  //   let cur = tree.nodes.len()
  //   tree.nodes.push(SPPFNode { prod, range, SmallVec::new() })
  //   if pos.0 is valid // I use a special value !0 as the invalid value
  //     let ch = tree.nodes[pos.0][0]
  //     // this is the location that the node of `tree.nodes` in its parent's pointers
  //     ch[ch.len() - pos.1.len() - 1] = cur
  //   if children.is_empty() { // leaf node, no more decision to make on this branch
  //     if let Some((parent, ch)) = poses.iter_mut().rfind(|(_, ch)| !ch.is_empty()) {
  //       let (fst, remain) = (ch[0], ch[1..])
  //       *ch = remain
  //       dfs(fst, (*parent, remain))
  //     else
  //       yield tree // this corresponds to the line `return Some(tree);`
  //   else
  //     tree.nodes[cur].children[0] = smallvec![0; prod.len() - 1] // tree.nodes[cur] is just pushed
  //     for ch in children
  //       let (fst, remain) = (ch[0], ch[1..])
  //       poses.push((cur, remain))
  //       dfs(fst, (cur, remain))
  //       poses.pop()
  //   tree.nodes.pop()
  //
  // it can't (directly) implement Iterator, because we can't specify Item = &'b SPPF<'a>
  // in order to implement Iterator, I will need to create another struct, with `stk`, `poses`, `tree` borrowed
  // I don't think the extra work is worthwhile
  pub fn next<'b>(&'b mut self) -> Option<&'b SPPF<'a>> {
    let Iter { sppf, stk, poses, tree } = self;
    let sppf = *sppf;
    loop {
      match stk.pop() {
        Some(State::_0 { node, pos }) => unsafe {
          let SPPFNode { prod, range, children } = sppf.nodes.get_unchecked(node);
          let cur = tree.nodes.len();
          tree.nodes.push(SPPFNode { prod, range: range.clone(), children: SmallVec::new() });
          if pos.0 != !0 {
            let ch = &mut tree.nodes.get_unchecked_mut(pos.0).children.get_unchecked_mut(0);
            let len = ch.len();
            *ch.get_unchecked_mut(len - pos.1.len() - 1) = cur as u32;
          }
          if children.is_empty() {
            if let Some((parent, ch)) = poses.iter_mut().rfind(|(_, ch)| !ch.is_empty()) {
              let parent = *parent;
              let (fst, remain) = split1(ch);
              *ch = remain;
              stk.push(State::_1);
              stk.push(State::_0 { node: fst as usize, pos: (parent, remain) });
            } else {
              stk.push(State::_1);
              return Some(tree);
            }
          } else {
            tree.nodes.get_unchecked_mut(cur).children.push(smallvec![0; prod.len() - 1]);
            let (fst, remain) = split1(children.get_unchecked(0));
            poses.push((cur, remain));
            stk.push(State::_2 { node, cur, ch_idx: 0 });
            stk.push(State::_0 { node: fst, pos: (cur, remain) });
          }
        }
        Some(State::_1) => { tree.nodes.pop(); }
        Some(State::_2 { node, cur, ch_idx }) => unsafe {
          let n = sppf.nodes.get_unchecked(node);
          poses.pop();
          let ch_idx = ch_idx + 1;
          if ch_idx < n.children.len() {
            let (fst, remain) = split1(n.children.get_unchecked(ch_idx));
            poses.push((cur, remain));
            stk.push(State::_2 { node, cur, ch_idx });
            stk.push(State::_0 { node: fst, pos: (cur, remain) });
          } else {
            tree.nodes.pop();
          }
        }
        None => return None,
      }
    }
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