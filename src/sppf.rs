use std::ops::Range;
use smallvec::{smallvec, SmallVec};
use crate::{Parser, split1, get, get_mut};

pub struct SPPFNode<'a> {
  // for non-terminal, prod[0] is always lhs, so prod is never empty
  // for terminal, prod is empty, range.start == range.end == position of this token, children is empty
  pub prod: &'a [u32],
  pub range: Range<u32>,
  // use index in Vec as pointers; Rc is not suitable, because there may be cycles
  pub children: SmallVec<[SmallVec<[u32; 4]>; 1]>,
}

pub struct SPPF<'a> {
  pub(crate) parser: &'a Parser<'a>,
  // `start` is the id of the start non-terminal
  pub(crate) start: u32,
  pub(crate) tokens: Vec<u32>,
  pub(crate) nodes: Vec<SPPFNode<'a>>,
}

// getters
impl<'a> SPPF<'a> {
  #[inline(always)]
  pub fn parser(&self) -> &'a Parser<'a> { self.parser }
  #[inline(always)]
  pub fn start(&self) -> u32 { self.start }
  #[inline(always)]
  pub fn tokens(&self) -> &Vec<u32> { &self.tokens }
  #[inline(always)]
  pub fn nodes(&self) -> &Vec<SPPFNode<'a>> { &self.nodes }
}

impl<'a> SPPF<'a> {
  pub(crate) fn find(&mut self, prod: &'a [u32], range: Range<u32>) -> u32 {
    self.nodes.iter_mut().position(|x| x.prod.as_ptr() == prod.as_ptr() && x.range == range)
      .unwrap_or_else(|| (self.nodes.len(), self.nodes.push(SPPFNode { prod, range, children: SmallVec::new() })).0) as u32
  }

  pub fn iter(&self) -> Iter {
    let mut stk = Vec::new();
    for (idx, SPPFNode { prod, range, .. }) in self.nodes.iter().enumerate() {
      if prod.get(0) == Some(&self.start) && range.end == self.tokens.len() as u32 {
        stk.push(State::_0 { node: idx as u32, pos: ParentSibling { sib: std::ptr::null(), parent: !0, sib_len: 0 } });
      }
    }
    let tree = SPPF { parser: self.parser, start: self.start, tokens: self.tokens.clone(), nodes: Vec::new() };
    Iter { sppf: self, stk, poses: Vec::new(), tree }
  }
}

// this is logically a pair (&[u32], u32), except for its size is smaller on 64-bit platform
// this pair means `(indices of siblings that is right to itself in `sppf`, index of parent in `tree`)
// (for more detail, please refer to the comment on Iter::next)
#[derive(Copy, Clone)]
struct ParentSibling {
  sib: *const u32,
  sib_len: u32,
  parent: u32,
}

enum State {
  _0 { node: u32, pos: ParentSibling },
  _1,
  _2 { node: u32, cur: u32, ch_idx: u32 },
}

pub struct Iter<'a> {
  sppf: &'a SPPF<'a>,
  stk: Vec<State>,
  poses: Vec<ParentSibling>,
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
  // // `pos` is the `ParentSibling` information of current node
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
          let SPPFNode { prod, range, children } = get(&sppf.nodes, node);
          let cur = tree.nodes.len() as u32;
          tree.nodes.push(SPPFNode { prod, range: range.clone(), children: SmallVec::new() });
          if pos.parent != !0 {
            let ch = get_mut(&mut get_mut(&mut tree.nodes, pos.parent).children, 0);
            let len = ch.len() as u32;
            *get_mut(ch, len - pos.sib_len - 1) = cur;
          }
          if children.is_empty() {
            if let Some(pos) = poses.iter_mut().rfind(|pos| pos.sib_len != 0) {
              let fst = *pos.sib;
              pos.sib = pos.sib.add(1);
              pos.sib_len -= 1;
              let pos = *pos;
              stk.push(State::_1);
              stk.push(State::_0 { node: fst, pos });
            } else {
              stk.push(State::_1);
              return Some(tree);
            }
          } else {
            get_mut(&mut tree.nodes, cur).children.push(smallvec![0; prod.len() - 1]);
            let (fst, remain) = split1(get::<SmallVec<_>>(children, 0));
            let pos = ParentSibling { sib: remain.as_ptr(), sib_len: remain.len() as u32, parent: cur };
            poses.push(pos);
            stk.push(State::_2 { node, cur, ch_idx: 0 });
            stk.push(State::_0 { node: fst, pos });
          }
        }
        Some(State::_1) => { tree.nodes.pop(); }
        Some(State::_2 { node, cur, ch_idx }) => unsafe {
          let n = get(&sppf.nodes, node);
          poses.pop();
          let ch_idx = ch_idx + 1;
          if ch_idx < n.children.len() as u32 {
            let (fst, remain) = split1(get::<SmallVec<_>>(&n.children, ch_idx));
            let pos = ParentSibling { sib: remain.as_ptr(), sib_len: remain.len() as u32, parent: cur };
            poses.push(pos);
            stk.push(State::_2 { node, cur, ch_idx });
            stk.push(State::_0 { node: fst, pos });
          } else {
            tree.nodes.pop();
          }
        }
        None => return None,
      }
    }
  }
}