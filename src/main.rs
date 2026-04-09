mod tube;
mod node;
use std::rc::Rc;
use std::error;
use std::fs;
use tube::Tube;
use node::Node;

fn main() -> Result<(), Box<dyn error::Error>> {
	let args: Vec<String> = std::env::args().collect();
	let data_file = if args.len() > 1 { &args[1] } else { "color_data" };
	let tubes = init_tubes(data_file)?;
	for tube in &tubes {
		tube.print();
	}
	let mut node = Node::new_root(tubes);

	loop {
		if node.goaled() {
			print_history(&node);
			break;
		}
		else if let Some(child_node) = node.next_child() {
			node = child_node;
		}
		else {
			if let Some(parent) = node.parent() {
				node = parent;
			}
			else {
				println!("failed");
				break
			}
		}
	}
	
	Ok(())
}

fn print_history(node : &Rc<Node>) {
	println!(">>>");
	for (i, (src_idx, dst_idx)) in node.history().iter().enumerate() {
		println!("{}, {}, {}", i + 1, src_idx, dst_idx);
	}
	println!("<<<");
}

fn init_tubes(data_file: &str) -> Result<Vec<Tube>, String> {

	let generate_error = |line_number : usize, line : &str| -> String {
		format!("invalid format on line {line_number} of '{data_file}' : {line}")
	};

	let content = fs::read_to_string(data_file)
		.map_err(|e| format!("failed to open '{data_file}' : {}", e))?;

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
			if ! c.is_ascii_lowercase() {
				return Err(generate_error(i+1, line));
			}
			tube.push(c, 1);
		}		
		tubes.push(tube);
	}

	Ok(tubes)
}
