use super::parser::{EMap, Element, Index, Node, PMap, Point, TMap};
// use std::collections::HashMap;

// NOTE: this doesn't cover the case of punning and cloning at the same time
// to solve this, the cloner must search for stacks for puns that actually reference graph points (instead of stack points)

struct State {
	pmap: PMap,
	emap: EMap,
	tmap: TMap,
}

fn last_point(key: &String) -> String {
	key.split(".")
		.map(str::to_string)
		.collect::<Vec<String>>()
		.last()
		.unwrap()
		.clone()
}

fn point_path(key: &String) -> Vec<String> {
	let mut k = key.split(".").map(str::to_string).collect::<Vec<String>>();
	k.pop();
	k
}

fn swap_point(keychain: &String, key: &String) -> String {
	let mut k = point_path(keychain);
	k.push(key.clone());
	k.join(".")
}

// swap second to last name in key chain
fn swap_graph(keychain: &String, key: &String) -> String {
	let mut k = keychain
		.split(".")
		.map(str::to_string)
		.collect::<Vec<String>>();
	let point = k.pop().unwrap();
	k.pop();
	k.push(key.clone());
	k.push(point);
	k.join(".")
}

fn sub_key_match(key_a: &String, key_b: &String) -> bool {
	last_point(key_a) == *key_b
}

fn contains(keys: &Vec<String>, key: &String) -> bool {
	let key = last_point(key);
	for k in keys {
		if sub_key_match(&k, &key) {
			return true;
		}
	}
	false
}

pub fn parser(pmap: PMap, emap: EMap, tmap: TMap) -> Result<(PMap, EMap, TMap), String> {
	let mut state = State { pmap, emap, tmap };

	let keys: Vec<String> = state.pmap.keys().cloned().collect();
	let vals: Vec<Point> = state.pmap.values().cloned().collect();
	for (i, key) in keys.iter().enumerate() {
		if vals[i].node == Node::Clone {
			state.begin_clone(key.clone())?;
		}
	}

	Ok((state.pmap, state.emap, state.tmap))
}

impl State {
	fn clone(
		&mut self,
		source_key: &String,
		target_key: &String,
	) -> Result<String, String> {
		let mut val = self.pmap.get(source_key).unwrap().clone();

		let new_key = [target_key.clone(), last_point(source_key)].join(".");

		match val.node {
			Node::Clone => {
				self.begin_clone(source_key.clone())?;
				// let val = self.pmap.get(source_key).unwrap().clone();
				self.clone(source_key, target_key)?;
				dbg!("hi");
			}
			Node::Stack => {
				val.key = new_key.clone();
				self.pmap.insert(new_key.clone(), val);
				let element = self.emap.get(source_key).unwrap().clone();
				self.emap.insert(new_key.clone(), element);
			}
			Node::Graph => {
				val.key = new_key.clone();
				let mut new_points: Vec<String> = Vec::new();
				for point in &val.points {
					// dbg!(point);
					new_points.push(self.clone(point, &new_key)?);
				}
				// dbg!(new_key);
				// panic!("hi");
				val.points = new_points;
				self.pmap.insert(new_key.clone(), val);
			}
		}
		Ok(new_key)
		// unimplemented!()
	}
	fn begin_clone(&mut self, key: String) -> Result<Index, String> {
		let mut clone = self.pmap.get(&key).unwrap().clone();
		// let clone_point = last_point(&clone.key);
		let mut keychain: Vec<String> = key.split(".").map(str::to_string).collect();
		let mut refchain: Vec<String> =
			clone.r#ref.split(".").map(str::to_string).collect();
		// keychain.pop();
		// keychain.append(&mut refchain);

		// dbg!(&keychain);

		// let origin = self.lookup(&mut keychain)?;
		// if origin.node == Node::Clone {
		// 	self.begin_clone(origin.key.clone())?;
		// }
		// // dbg!();
		// let origin = self.lookup(&mut keychain)?;

		let origin = self.lookup(&keychain, &refchain)?;
		if origin.node == Node::Clone {
			self.begin_clone(origin.key.clone())?;
		}
		let origin = self.lookup(&keychain, &refchain)?;
		// dbg!(origin);
		// panic!();

		let mut points_to_clone: Vec<String> = Vec::new();
		for op in &origin.points {
			if !contains(&clone.points, op) {
				points_to_clone.push(op.clone());
			}
		}

		dbg!(&points_to_clone);

		for point in points_to_clone {
			clone.points.push(self.clone(&point, &clone.key)?);
		}

		clone.node = Node::Graph;
		self.pmap.insert(key, clone);

		// dbg!(key);
		Ok(0)
	}

	fn lookup(
		&mut self,
		keychain: &Vec<String>,
		refchain: &Vec<String>,
	) -> Result<Point, String> {
		let key = [keychain.join("."), refchain.join(".")].join(".");
		dbg!(&key);
		// panic!();
		match self.pmap.get(&key) {
			Some(v) => Ok(v.clone()),
			None => {
				if keychain.len() > 1 {
					let mut keychain = keychain.clone();
					keychain.swap_remove(keychain.len() - 1);
					self.lookup(&keychain, refchain)
				} else {
					Err(format!("{} is undefined", keychain[0]))
				}
			}
		}
	}
}
