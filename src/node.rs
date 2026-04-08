use std::rc::Rc;
use crate::tube::Tube;
use std::collections::VecDeque;
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
	tubes: Vec<Tube>,
	next_child_indices: Cell<(usize, usize)>,
}

impl Node {
	pub fn new_root(tubes: Vec<Tube>) -> Rc<Self> {
		Rc::new(Self {
			kind: NodeKind::Root,
			tubes: tubes,
			next_child_indices: Cell::new((0, 0)),
		})
	}
	fn new_child(self:&Rc<Self>, tubes:Vec<Tube>, src_idx:usize, dst_idx:usize, src_fixed: bool) -> Rc<Self> {
		Rc::new(Self {
			kind: NodeKind::Child {
				parent: self.clone(),
				src_idx: src_idx,
				dst_idx: dst_idx,
				src_fixed: src_fixed,
			},
			tubes: tubes,
			next_child_indices: Cell::new((0, 0)),
		})
	}
	pub fn parent(self: &Rc<Self>) -> Option<Rc<Node>> {
		if let NodeKind::Child {parent, ..} = &self.kind {
			Some(parent.clone())
		}
		else {
			None
		}
	}
	pub fn next_child(self: &Rc<Self>) -> Option<Rc<Node>> {
		let (ni, nj) = self.next_child_indices.get();
		let set_next_indices = |mut i:usize, mut j:usize, tube_cnt:usize| {
			if j >= tube_cnt {
				j = 0;
				i += 1;
			}
			self.next_child_indices.set((i, j));
		};
		let process = |i: usize, tube_src: &Tube, src_fixed: bool| -> Option<Rc<Self>> {
			let (color, upper_cnt) = tube_src.upper_info();
			if upper_cnt == 0 || upper_cnt == Tube::max_cnt() {return None;}
			for (j, tube_dst) in self.tubes.iter().enumerate().skip(nj) {
				if i == j {continue}
				let dst_remain_cnt = tube_dst.remain_cnt();
				if dst_remain_cnt == 0 {continue}
				if dst_remain_cnt == Tube::max_cnt() {
					if tube_src.remain_cnt() + upper_cnt == Tube::max_cnt() {continue}
				}
				if let Some(&dst_color) = tube_dst.upper_color() {
					if color != dst_color {continue}
				}
				let mut child_tubes = self.tubes.clone();
				set_next_indices(i, j+1, child_tubes.len());
				if dst_remain_cnt >= upper_cnt {
					child_tubes[i].pop(upper_cnt);
					child_tubes[j].push(color, upper_cnt);
					return Some(self.new_child(child_tubes, i, j, src_fixed));
				}
				child_tubes[i].pop(dst_remain_cnt);
				child_tubes[j].push(color, dst_remain_cnt);
				return Some(self.new_child(child_tubes, i, j, true));
			}
			None
		};

		if let NodeKind::Child {src_fixed:true, src_idx, ..} = &self.kind {
			process(*src_idx, &self.tubes[*src_idx], true)
		}
		else {
			for (i, tube_src) in self.tubes.iter().enumerate().skip(ni) {
				if let Some(child) = process(i, tube_src, false) {
					return Some(child);
				}
			}
			None
		}
	}
	pub fn goaled(self: &Rc<Self>) -> bool {
		for tube in &self.tubes {
			let size = tube.upper_info().1;
			if size == 0 || size == Tube::max_cnt() {continue}
			return false;
		}
		true
	}
	pub fn history(self: &Rc<Self>) -> VecDeque<(&usize,&usize)> {
		let mut deq = VecDeque::new();
		let mut node = self;
		loop {
			match &node.kind {
				NodeKind::Root => break,
				NodeKind::Child { parent, src_idx, dst_idx, .. } => {
					deq.push_front((src_idx, dst_idx));
					node = parent;
				}
			}
		}
		deq
	}
}