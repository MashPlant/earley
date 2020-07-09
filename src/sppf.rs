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
      if prod.get(0) == Some(&self.start) && range.start == 0 && range.end == self.tokens.len() as u32 {
        stk.push(State::_0 { node: idx as u32, parent: !0, ch_idx: 0 });
      }
    }
    let tree = SPPF { parser: self.parser, start: self.start, tokens: self.tokens.clone(), nodes: Vec::new() };
    Iter { sppf: self, stk, poses: Vec::new(), tree }
  }
}

enum State {
  _0 { node: u32, parent: u32, ch_idx: u32 },
  _1,
  _2 { node: u32, cur: u32, ch_idx: u32 },
}

pub struct Iter<'a> {
  sppf: &'a SPPF<'a>,
  stk: Vec<State>,
  poses: Vec<(u32, u32, u32)>,
  tree: SPPF<'a>,
}

impl<'a> Iter<'a> {
  // `Iter::next` will return a (maybe infinite) sequence of trees before it return None
  // each tree is represented by `SPPF`, with `children.len() <= 1` guaranteed (so that it is a determined tree)
  //
  // this function may be somewhat hard to understand, this is because it is transformed from a recursive function
  // to a non-recursive one based on heap-allocated stack and state machine
  // the pseudo-code of the original recursive function is like (implicit `self`, omitting type cast, use safe functions):
  //
  // `sppf`: original sppf, the `children` in it may contain multiple choices
  // `tree`: target tree, we want to explore all the possibility in `sppf`, and generate concrete trees
  // param `node`: index of the node that we are going to explore in `sppf`
  // param `parent`: index of its parent in `tree` (when it has no parent, it is `!0`)
  // param `ch_idx`: index of the pointer to self in `tree.nodes[parent].children[0]`
  // `poses`: stack of `(node, parent, ch_idx)`, when we reach a leaf in `sppf`, we pop a pos and explore it
  // fn dfs(u32 node, u32 parent, u32 ch_idx)
  //   let { prod, range, children } = sppf[node]
  //   let cur = tree.nodes.len()
  //   tree.nodes.push(SPPFNode { prod, range, SmallVec::new() })
  //   if pos.0 is valid // I use a special value !0 as the invalid value
  //     tree.nodes[pos.0].children[0][ch_idx] = cur
  //   if children.is_empty() { // leaf node, no more decision to make on this branch
  //     if let Some((node, parent, ch_idx)) = poses.pop() {
  //       dfs(node, parent, ch_idx)
  //     else
  //       yield tree // this corresponds to the line `return Some(tree);`
  //   else
  //     tree.nodes[cur].children[0] = smallvec![0; prod.len() - 1] // tree.nodes[cur] is just pushed
  //     for ch in children
  //       let (fst, remain) = (ch[0], ch[1..])
  //       for (idx, &r) in remain.iter().enumerate().rev() // push siblings to `poses` reversely
  //         poses.push((r, cur, idx + 1));
  //       dfs(fst, cur, 0)
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
        Some(State::_0 { node, parent, ch_idx }) => unsafe {
          let SPPFNode { prod, range, children } = get(&sppf.nodes, node);
          let cur = tree.nodes.len() as u32;
          tree.nodes.push(SPPFNode { prod, range: range.clone(), children: SmallVec::new() });
          if parent != !0 {
            *get_mut(get_mut::<SmallVec<_>>(&mut get_mut(&mut tree.nodes, parent).children, 0), ch_idx) = cur;
          }
          if children.is_empty() {
            stk.push(State::_1);
            if let Some((node, parent, ch_idx)) = poses.pop() {
              stk.push(State::_0 { node, parent, ch_idx });
            } else {
              return Some(tree);
            }
          } else {
            get_mut(&mut tree.nodes, cur).children.push(smallvec![0; prod.len() - 1]);
            stk.push(State::_2 { node, cur, ch_idx: 0 });
          }
        }
        Some(State::_1) => { tree.nodes.pop(); }
        Some(State::_2 { node, cur, ch_idx }) => unsafe {
          let n = get(&sppf.nodes, node);
          if ch_idx < n.children.len() as u32 {
            let (fst, remain) = split1(get::<SmallVec<_>>(n.children.as_slice(), ch_idx));
            poses.reserve(remain.len());
            for (idx, &r) in remain.iter().enumerate().rev() {
              poses.push((r, cur, idx as u32 + 1));
            }
            stk.push(State::_2 { node, cur, ch_idx: ch_idx + 1 });
            stk.push(State::_0 { node: fst, parent: cur, ch_idx: 0 });
          } else {
            tree.nodes.pop();
          }
        }
        None => return None,
      }
    }
  }
}