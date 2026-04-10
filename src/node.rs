use std::rc::Rc;
use crate::tube::Tube;
use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::collections::HashMap;
use std::vec::Vec;
use std::ops::Bound::{Included, Excluded, Unbounded};
use std::cell::Cell;
pub enum NodeKind {
	Root,
	Child {
		parent: Rc<Node>,
		src_idx: usize,
		dst_idx: usize,
		src_fixed: bool,
	}
}
pub struct Node {
	kind: NodeKind,
	pub depth: usize,
	tubes: BTreeMap<usize, Tube>,
	next_child_indices: Cell<(usize, usize)>,
}

impl Node {
	pub fn new_root(tubes: BTreeMap<usize, Tube>) -> Rc<Self> {

		Rc::new(Self {
			kind: NodeKind::Root,
			depth: 0,
			tubes: tubes,
			next_child_indices: Cell::new((1, 2)),
		})
	}
	fn new_child(self:&Rc<Self>, tubes:BTreeMap<usize, Tube>, src_idx:usize, dst_idx:usize, src_fixed: bool) -> Rc<Self> {
		Rc::new(Self {
			kind: NodeKind::Child {
				parent: self.clone(),
				src_idx: src_idx,
				dst_idx: dst_idx,
				src_fixed: src_fixed,
			},
			depth: self.depth + 1,
			tubes: tubes,
			next_child_indices: Cell::new((1, 2)),
		})
	}
	pub fn get_hash(&self) ->u64 {
		let mut hash_raw: Vec<&Vec<u8>> = Vec::with_capacity(self.tubes.len());
		for (_, tube) in &self.tubes {
			hash_raw.push(tube.get_stack());
		}
		hash_raw.sort();
		let mut hasher = DefaultHasher::new();
		hash_raw.hash(&mut hasher);
		hasher.finish()
	}
	pub fn parent(self: &Rc<Self>) -> Option<Rc<Node>> {
		if let NodeKind::Child {parent, ..} = &self.kind {
			Some(parent.clone())
		}
		else {
			None
		}
	}
	pub fn next_child(self: &Rc<Self>, seen: &mut HashMap<u64, usize>) -> Option<Rc<Node>> {
		let (ni, nj) = self.next_child_indices.get();
		let set_next_indices = |mut i:usize, mut j:usize| {
			if let Some((next_j, _)) = self.tubes.range((Excluded(&j), Unbounded)).next() {
				j = *next_j;
			}
			else {
				if let Some((next_i, _)) = self.tubes.range((Excluded(&i), Unbounded)).next() {
					j = 1;
					i = *next_i;
				}
				else {
					i += 1;
				}
			}
			self.next_child_indices.set((i, j));
		};
		let move_water = |tubes: &mut BTreeMap<usize, Tube>, color: u8, src_idx: usize, dst_idx: usize, cnt: usize| ->u64 {
			tubes.get_mut(&src_idx).unwrap().pop(cnt);
			let dst_tube = tubes.get_mut(&dst_idx).unwrap();
			dst_tube.push(color, cnt);
			if dst_tube.is_complete() {
				tubes.remove(&dst_idx);
			}
			let mut hash_raw: Vec<Vec<u8>> = Vec::with_capacity(tubes.len());
			for (_, tube) in tubes {
				hash_raw.push(tube.get_stack().clone());
			}
			hash_raw.sort();
			let mut hasher = DefaultHasher::new();
			hash_raw.hash(&mut hasher);
			hasher.finish()
		};
		let mut process = |src_idx: usize, initial_dst_idx: usize, tube_src: &Tube| -> Option<Rc<Self>> {
			let (color, upper_cnt) = tube_src.upper_info();
			if color == 0 {return None;}
			let src_is_pure = tube_src.is_pure();
			for (ref_j, tube_dst) in self.tubes.range((Included(initial_dst_idx), Unbounded)) {
				let j = *ref_j;
				if src_idx == j {continue}
				let dst_remain_cnt = tube_dst.remain_cnt();
				if dst_remain_cnt == 0 {continue}
				if src_is_pure {
					if tube_dst.is_empty() {continue}
					if tube_dst.is_pure() && tube_src.height() > tube_dst.height() {continue}
				}
				if let Some(&dst_color) = tube_dst.upper_color() {
					if color != dst_color {continue}
				}
				let mut child_tubes = self.tubes.clone();
				let (cnt, src_fixed) = if dst_remain_cnt < upper_cnt {
					(dst_remain_cnt, true)
				}
				else {
					(upper_cnt, false)
				};
				let hash = move_water(&mut child_tubes, color, src_idx, j, cnt);
				if let Some(depth) = seen.get(&hash) {
					if *depth <= self.depth + 1 {
						continue;
					}
				}
				seen.insert(hash, self.depth + 1);
				set_next_indices(src_idx, j);
				return Some(self.new_child(child_tubes, src_idx, j, src_fixed));
			}
			None
		};

		if let NodeKind::Child {src_fixed:true, src_idx, ..} = &self.kind {
			if ni != *src_idx {
				None
			}
			else {
				process(*src_idx, nj, &self.tubes[src_idx])
			}
		}
		else {
			let mut dst_idx = nj;
			for (i, tube_src) in self.tubes.range((Included(ni), Unbounded)) {
				if let Some(child) = process(*i, dst_idx, tube_src) {
					return Some(child);
				}
				dst_idx = 1;
			}
			None
		}
	}
	pub fn goaled(self: &Rc<Self>) -> bool {
		for (_, tube) in &self.tubes {
			if !tube.is_empty() {return false;}			
		}
		true
	}
	pub fn history(self: &Rc<Self>) {
		println!(">>> ---");
		let mut node = self;
		loop {
			match &node.kind {
				NodeKind::Root => break,
				NodeKind::Child { parent, src_idx, dst_idx, .. } => {
					println!("{:>2}: {:>2}->{:>2} {:016x}", node.depth, src_idx, dst_idx, node.get_hash());
					node = parent;
				}
			}
		}
		println!("--- <<<");
	}
}