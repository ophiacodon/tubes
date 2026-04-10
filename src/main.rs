mod tube;
mod node;
use std::error;
use std::fs;
use std::collections::BTreeMap;
use std::collections::HashMap;
use std::rc::Rc;
use tube::Tube;
use node::Node;

fn main() -> Result<(), Box<dyn error::Error>> {
	let args: Vec<String> = std::env::args().collect();
	let data_file = if args.len() > 1 { &args[1] } else { "color_data" };
	let tubes = init_tubes(data_file)?;
	println!("{}", data_file);
	for (_, tube) in &tubes {
		tube.print();
	}

	let mut seen: HashMap<u64,usize> = HashMap::new();
	let mut node = Node::new_root(tubes);
	let mut goal_depth = usize::MAX;
	let mut goal_node: Option<Rc<Node>> = None;
	loop {
		if node.goaled() {
			goal_node = Some(node.clone());
			// node.history();
			goal_depth = node.depth;
			if let Some(parent) = node.parent() {
				node = parent;
			}
		}
		else {
			if node.depth >= goal_depth- 1 {
				if let Some(parent) = node.parent() {
					node = parent;
				}
			}
			else if let Some(child_node) = node.next_child(&mut seen) {
				node = child_node;
			}
			else {
				if let Some(parent) = node.parent() {
					node = parent;
				}
				else {
					if let Some(node_rc) = &goal_node {
						node_rc.history();
					}
					else {
						println!("not found");
					}
					break
				}
			}
		}
	}
	println!("node count = {}", seen.len());
	
	Ok(())
}

fn init_tubes(data_file: &str) -> Result<BTreeMap<usize, Tube>, String> {

	let generate_error = |line_number : usize, line : &str| -> String {
		format!("invalid format on line {line_number} of '{data_file}' : {line}")
	};

	let content = fs::read_to_string(data_file)
		.map_err(|e| format!("failed to open '{data_file}' : {}", e))?;

	let mut tubes: BTreeMap<usize, Tube> = BTreeMap::new();
	let mut is_first = true;
	let mut tube_id: usize = 1;
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

		let mut tube = Tube::new(tube_id);
		for c in line.bytes() {
			if c == b'-' {break}
			if ! c.is_ascii_lowercase() {
				return Err(generate_error(i+1, line));
			}
			tube.push(c, 1);
		}		
		tubes.insert(tube_id, tube);
		tube_id += 1;
	}

	Ok(tubes)
}
