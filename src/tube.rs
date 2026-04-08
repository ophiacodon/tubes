use std::sync::OnceLock;

static MAX_CNT: OnceLock<usize> = OnceLock::new();

#[derive(Clone)]
pub struct Tube {
	stack: Vec<u8>,
}

impl Tube {
	pub fn new() -> Self {
		Self { stack: Vec::new() }
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
	pub fn push(&mut self, color: u8, cnt: usize) {
		self.stack.resize(self.stack.len() + cnt, color);
	}
	pub fn pop(&mut self, cnt: usize) {
		debug_assert!(self.stack.len() >= cnt);
		self.stack.truncate(self.stack.len() - cnt);
	}
	pub fn upper_color(&self) -> Option<&u8> {
		self.stack.last()
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
	pub fn print(&self) {
		println!("{}", std::str::from_utf8(&self.stack).unwrap());
	}
}