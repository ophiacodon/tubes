mod tube;
mod node;
use std::rc::Rc;
use std::error;
use std::fs;
use tube::Tube;
use node::Node;

fn main() -> Result<(), Box<dyn error::Error>> {
	let tubes = init_tubes()?;
	for tube in &tubes {
		tube.print();
	}
	let mut node = Node::new_root(tubes);

	loop {
		if node.goaled() {
			print_history(&node);
			break
		}
		if let Some(child_node) = node.next_child() {
			node = child_node;
		}
		else {
			if let Some(parent) = node.parent() {
				node = parent;
			}
			else {
				break
			}
		}
	}
	
	Ok(())
}

fn print_history(node : &Rc<Node>) {
	for (src_idx, dst_idx) in &node.history() {
		println!("{}, {}", src_idx, dst_idx);
	}
}

const DATA_FILE: &str = "color_data";

fn generate_error(line_number : usize, line : &str) -> String {
	format!("invalid format on line {line_number} of '{DATA_FILE}' : {line}")
}

fn init_tubes() -> Result<Vec<Tube>, String> {
	let content = fs::read_to_string(DATA_FILE)
		.map_err(|e| format!("failed to open '{DATA_FILE}' : {}", e))?;

	let mut tubes: Vec<Tube> = Vec::new();
	let mut is_first = true;
	for (i,line) in content.lines().enumerate() {
		if line.is_empty() || line.starts_with('#') {
			continue;
		}
		
		if is_first {
			Tube::init_max_cnt(line.len());
			is_first = false;
		}
		else {
			if line.len() > Tube::max_cnt() {
				return Err(generate_error(i+1, line));
			}
		}

		let mut tube = Tube::new();
		for c in line.bytes() {
			if c == b'-' {break}
			if ! c.is_ascii_lowercase() || c > b'k' {
				return Err(generate_error(i+1, line));
			}
			tube.push(c, 1);
		}		
		tubes.push(tube);
	}

	Ok(tubes)
}
