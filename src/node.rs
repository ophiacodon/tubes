use std::rc::Rc;
use crate::tube::Tube;
use std::collections::BTreeMap;
use std::collections::HashSet;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
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
		root_indices: (usize, usize),
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
			next_child_indices: Cell::new((0, 0)),
		})
	}
	fn new_child(self:&Rc<Self>, tubes:BTreeMap<usize, Tube>,
		src_idx:usize, dst_idx:usize, src_fixed: bool) -> Rc<Self> {
		let root_indices = match &self.kind {
			NodeKind::Child {root_indices, ..} => {
				if root_indices.0 == 0 {
					if root_indices.1 < src_idx {
						(root_indices.1, src_idx)
					}
					else {
						(src_idx, root_indices.1)
					}
				}
				else {
					*root_indices
				}
			}
			NodeKind::Root => {
				(0, src_idx)
			}
		};
			
		Rc::new(Self {
			kind: NodeKind::Child {
				parent: self.clone(),
				src_idx: src_idx,
				dst_idx: dst_idx,
				src_fixed: src_fixed,
				root_indices: root_indices,
			},
			depth: self.depth + 1,
			tubes: tubes,
			next_child_indices: Cell::new((0, 0)),
		})
	}
	pub fn get_first_indices(self: &Rc<Self>) ->&(usize, usize) {
		if let NodeKind::Child {root_indices, ..} = &self.kind {
			root_indices
		}
		else {&(0, 0)}
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
	fn set_next_indices (self: &Rc<Self>, mut i:usize, mut j:usize) {
		if let Some((&next_j, _)) = self.tubes.range((Excluded(&j), Unbounded)).next() {
			j = next_j;
		}
		else {
			if let Some((next_i, _)) = self.tubes.range((Excluded(&i), Unbounded)).next() {
				j = 0;
				i = *next_i;
			}
			else {
				i += 1;
			}
		}
		self.next_child_indices.set((i, j));
	}
	fn move_water (tubes: &mut BTreeMap<usize, Tube>, color: u8, src_idx: usize, dst_idx: usize, cnt: usize) {
		tubes.get_mut(&src_idx).unwrap().pop(cnt);
		let dst_tube = tubes.get_mut(&dst_idx).unwrap();
		dst_tube.push(color, cnt);
		if dst_tube.is_complete() {
			tubes.remove(&dst_idx);
		}
	}
	fn process (self: &Rc<Self>, src_idx: usize, initial_dst_idx: usize, tube_src: &Tube) -> Option<Rc<Self>> {
		let (color, upper_cnt) = tube_src.upper_info();
		let src_pure = tube_src.is_pure();
		if color == 0 {return None;}
		for (&dst_idx, tube_dst) in self.tubes.range((Included(initial_dst_idx), Unbounded)) {
			if src_idx == dst_idx {continue}
			let dst_remain_cnt = tube_dst.remain_cnt();
			if dst_remain_cnt == 0 {continue}
			if src_pure && tube_dst.is_empty() {continue}
			let dst_color = tube_dst.upper_color();
			if dst_color != 0 && dst_color != color {continue}
			let mut child_tubes = self.tubes.clone();
			let (cnt, src_fixed) = if dst_remain_cnt < upper_cnt {
				(dst_remain_cnt, true)
			}
			else {
				(upper_cnt, false)
			};
			Self::move_water(&mut child_tubes, color, src_idx, dst_idx, cnt);
			self.set_next_indices(src_idx, dst_idx);
			return Some(self.new_child(child_tubes, src_idx, dst_idx, src_fixed));
		}
		None
	}
	pub fn next_child(self: &Rc<Self>) -> Option<Rc<Node>> {
		let (ni, nj) = self.next_child_indices.get();

		if let NodeKind::Child {src_fixed: true, src_idx, ..} = &self.kind {
			if *src_idx < ni {
				None
			}
			else {
				self.process(*src_idx, nj, &self.tubes[src_idx])
			}
		}
		else {
			if (ni, nj) == (0, 0) {
				for (src_idx, tube_src) in &self.tubes {
					if !tube_src.is_pure() {continue;}
					let color = tube_src.upper_color();

					for (dst_idx, tube_dst) in &self.tubes {
						if src_idx == dst_idx {continue}
						if tube_dst.is_empty() {continue}
						if !tube_dst.is_pure() {continue}
						if tube_dst.upper_color() != color {continue}
						let (&src, &dst, cnt) = if tube_src.height() <= tube_dst.height() {
							(src_idx, dst_idx, tube_src.height())
						}
						else {
							(dst_idx, src_idx, tube_dst.height())
						};
						let mut child_tubes = self.tubes.clone();
						Self::move_water(&mut child_tubes, color, src, dst, cnt);
						self.set_next_indices(usize::MAX, 0);
						return Some(self.new_child(child_tubes, src, dst, false));
					}
				}
			}
			let mut initial_dst_idx = nj;
			for (&src_idx, tube_src) in self.tubes.range((Included(ni), Unbounded)) {
				if let Some(child) = self.process(src_idx, initial_dst_idx, tube_src) {
					return Some(child);
				}
				initial_dst_idx = 1;
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
					let hash = node.get_hash();
					println!("{:>2}: {:>2}->{:>2} {:#016x}", node.depth, src_idx, dst_idx, hash);
					node = parent;
				}
			}
		}
		println!("--- <<<");
	}
}