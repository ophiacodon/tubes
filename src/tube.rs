use std::sync::OnceLock;

static MAX_CNT: OnceLock<usize> = OnceLock::new();

#[derive(Clone)]
pub struct Tube {
	pub id: usize,
	stack: Vec<u8>,
}

impl Tube {
	pub fn new(id: usize) -> Self {
		Self {
			id: id,
			stack: Vec::with_capacity(Self::max_cnt())
		}
	}
	pub fn init_max_cnt(n: usize) {
		MAX_CNT.set(n).expect("already initialized")
	}
	pub fn max_cnt() -> usize {
		*MAX_CNT.get().expect("not initialized")
	}
	pub fn remain_cnt(&self) -> usize {
		Self::max_cnt() - self.stack.len()
	}
	pub fn is_empty(&self) -> bool {
		self.stack.len() == 0
	}
	pub fn is_pure(&self) -> bool {
		if self.stack.len() > 0 {
			let last_color = self.stack.last().unwrap();
			for color in &self.stack {
				if color != last_color {
					return false;
				}
			}
			true
		}
		else {
			false
		}
	}
	pub fn height(&self) ->usize {
		self.stack.len()
	}
	pub fn is_complete(&self) -> bool {
		self.upper_info().1 == Self::max_cnt()
	}
	pub fn push(&mut self, color: u8, cnt: usize) {
		self.stack.resize(self.stack.len() + cnt, color);
		debug_assert!(self.stack.len() <= Self::max_cnt());
	}
	pub fn pop(&mut self, cnt: usize) {
		debug_assert!(self.stack.len() >= cnt);
		self.stack.truncate(self.stack.len() - cnt);
	}
	pub fn upper_color(&self) -> u8 {
		if let Some(&color) = self.stack.last() {return color;}
		0
	}
	pub fn upper_info(&self) -> (u8, usize) {
		if self.stack.len() > 0 {
			let top_color = self.stack.last().cloned().unwrap();
			let cnt = self.stack.iter().rev().take_while(|&&x| x == top_color).count();
			(top_color, cnt)
		}
		else {
			(0, 0)
		}
	}
	pub fn get_stack(&self) ->&Vec<u8> {
		&self.stack
	}
	pub fn print(&self) {
		println!("{:>2}: {}", self.id, std::str::from_utf8(&self.stack).unwrap());
	}
}