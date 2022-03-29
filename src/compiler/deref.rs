use super::parser::{Element, Env, Index, RvES, RES};
use std::collections::HashMap;

struct State {
	points1: Vec<Element>,
	// points2: Vec<Element>,
}

pub fn parser(points1: Vec<Element>) -> RvES {
	// let mut points2: Vec<Element> = Vec::with_capacity(points1.len());
	let mut points2: Vec<Element> = vec![Element::Nothing; points1.len()];
	let mut state = State { points1 };

	state.dispatch(&mut points2, &HashMap::new(), 0)?;

	Ok(points2)
}

impl State {
	fn dispatch(
		&mut self,
		points2: &mut Vec<Element>,
		env: &Env,
		index: Index,
	) -> Result<Index, String> {
		let el = match self.points1[index].clone() {
			Element::Graph {
				keys,
				env: graph_env,
			} => {
				for k in &keys {
					self.dispatch(points2, &graph_env, *k)?;
				}

				points2[index] = Element::Graph {
					env: graph_env.clone(),
					keys: keys.clone(),
				};
			}

			Element::Clone {
				texts,
				keys,
				env: clone_env,
				index: _,
			} => {
				for k in &keys {
					self.dispatch(points2, &clone_env, *k)?;
				}

				let index_ = self.deref(env, &texts)?;

				points2[index] = Element::Clone {
					texts: texts.clone(),
					index: index_,
					env: clone_env.clone(),
					keys: keys.clone(),
				};
			}

			Element::Tuple { body } => {
				let mut newbody: Vec<Element> = Vec::new();
				for e in body {
					newbody.push(self.element(env, &e)?);
				}

				points2[index] = Element::Tuple { body: newbody };
			}
			// e => return Err(format!("Unexpected element: {:?}", e)),
			e => {
				// print!("Skipping: {:?}", e);
				// e.clone()
				panic!("unexpected element: {:?}", e);
			}
		};
		Ok(index)
	}

	fn element(&mut self, env: &Env, element: &Element) -> RES {
		let el = match element {
			Element::Ref { texts, index } => {
				let index = self.deref(env, texts)?;

				Element::Ref {
					texts: texts.clone(),
					index,
				}
			}
			Element::Pun { texts, index: _ } => {
				let parent_index =
					self.lookup(env, &String::from("(parent)"))?;
				let index = match &self.points1[parent_index] {
					Element::Graph { env, keys } => self.deref(env, texts)?,
					e => panic!("unexpected element: {:?}", e),
				};

				Element::Ref {
					texts: texts.clone(),
					index,
				}
			}
			e => e.clone(),
		};
		Ok(el)
	}
}

impl State {
	fn deref(
		&self,
		env: &Env,
		key_chain: &Vec<String>,
	) -> Result<Index, String> {
		let mut env: &Env = env;
		for (i, key) in key_chain.iter().enumerate() {
			if i == key_chain.len() - 1 {
				return Ok(self.lookup(env, key)?);
			} else {
				let index = self.lookup(env, key)?;
				env = match &self.points1[index] {
					Element::Graph { env, keys } => env,
					e => panic!("Unexpected element: {:?}", e),
				};
			}
		}
		Ok(0)
	}

	fn lookup(&self, env: &Env, key: &String) -> Result<Index, String> {
		match env.get(key) {
			Some(i) => Ok(*i),
			None => {
				match env.get(&String::from("(parent)")) {
					None => Err(format!("{} is undefined", key)),
					Some(i) => {
						match &self.points1[*i] {
							Element::Graph { env, keys } => {
								self.lookup(&env, key)
							}
							// Element::Clone { text, env, keys} => {
							// 	self.lookup(env, key)
							// }
							e => panic!("Unexpected element: {:?}", e),
						}
					}
				}
			}
		}
	}

	// fn clone_elements(
	// 	&self,
	// 	points2: &mut Vec<Element>,
	// 	keys_to_copy: &Vec<Index>,
	// ) -> Vec<Index> {
	// 	let mut new_keys: Vec<Index> = Vec::new();

	// 	for index in keys_to_copy {
	// 		new_keys.push(self.clone_element(points2, *index));
	// 	}

	// 	new_keys
	// }

	// fn clone_element(
	// 	&self,
	// 	points2: &mut Vec<Element>,
	// 	key: Index,
	// ) -> Index {
	// 	let index = points2.len();
	// 	let el = &points2[key];
	// 	let e = match el {
	// 		Element::Clone { env, keys, texts } => {
	// 			panic!("fudge");
	// 		}
	// 		Element::Graph { env, keys } => {
	// 			for (k, v) in env {

	// 			}
	// 		}
	// 		e => {
	// 			e.clone()
	// 		}
	// 	};
	// 	points2.push(e);
	// 	index
	// }
}
