use super::tokenizer::{Kind, Token};
use std::cell::RefCell;
use std::collections::HashMap;

pub type Index = usize;

pub type Env = HashMap<String, Index>;

#[derive(Debug, Clone)]
pub enum Element {
	Nothing,
	Parent {
		env: Env,
	},
	Graph {
		env: Env,
		keys: Vec<Index>,
	},
	Clone {
		texts: Vec<String>,
		index: Index,
		env: Env,
		keys: Vec<Index>,
	},
	Tuple {
		body: Vec<Element>,
	},
	Array {
		body: Vec<Element>,
	},
	//
	Number {
		text: String,
	},

	//
	// Key {
	// 	text: String,
	// 	body: Box<Element>,
	// },
	Par {
		text: String,
		index: Index,
	},
	Ref {
		texts: Vec<String>,
		index: Index,
	},
	Pun {
		texts: Vec<String>,
		index: Index,
	},
	//
	Function {
		head: Vec<Element>,
		body: Vec<Element>,
	},
}

// find keys
// find refs

// every graph needs to have a namespace stack

struct State {
	cursor: RefCell<usize>,
	tokens: Vec<Token>,
	points: Vec<Element>,
}

pub fn parser(tokens: Vec<Token>) -> RvES {
	let mut state = State {
		cursor: RefCell::new(0),
		tokens,
		points: Vec::new(),
	};

	state.program()?;
	Ok(state.points)
}

pub type RES = Result<Element, String>;
pub type RvES = Result<Vec<Element>, String>;

impl State {
	fn program(&mut self) -> Result<Index, String> {
		let mut env: Env = HashMap::new();
		// env.insert(String::from("(parent)"), 0);
		self.points.push(Element::Nothing);
		let keys = self.keys(&mut env, 0, &[])?;
		let program = Element::Graph { env, keys };
		self.points[0] = program;
		Ok(0)
	}
	fn keys(
		&mut self,
		env: &mut Env,
		index: Index,
		stop: &[Kind],
	) -> Result<Vec<Index>, String> {
		let mut points: Vec<Index> = Vec::new();
		while self.until(0, stop) {
			points.push(self.key(env, index)?);
		}
		Ok(points)
	}

	fn key(
		&mut self,
		env: &mut Env,
		parent: Index,
	) -> Result<Index, String> {
		let key_token = self.eat(Kind::Key)?;
		let key_text = key_token.meta.text.clone()[1..].to_string();
		if env.get(&key_text).is_some() {
			return Err(format!(
				"Point with key: {} has already been used in this context",
				key_text
			));
		}

		let index = self.points.len();
		self.points.push(Element::Nothing);

		let body = if self.is(0, Kind::BracketLF) {
			self.graph(index, parent)?
		} else if self.is(0, Kind::Ref) && self.is(1, Kind::BracketLF) {
			self.clone(index, parent)?
		} else {
			let mut params: Vec<String> = Vec::new();
			let mut body =
				self.stack(env, &mut params, &[Kind::Key, Kind::BracketRT])?;
			if body.len() == 0 {
				// key punning
				body.push(Element::Pun {
					texts: vec![key_text.clone()],
					index: 0,
				});
			}
			Element::Tuple { body }
		};

		// self.points[index] = Element::Key {
		// 	text: key_text.clone(),
		// 	body: Box::new(body),
		// };
		self.points[index] = body;
		env.insert(key_text, index);
		Ok(index)
	}

	fn stack(
		&mut self,
		env: &mut Env,
		params: &mut Vec<String>,
		stop: &[Kind],
	) -> RvES {
		let mut elements: Vec<Element> = Vec::new();
		while self.until(0, stop) {
			elements.push(self.word(env, params)?);
		}
		Ok(elements)
	}

	fn word(&mut self, env: &mut Env, params: &mut Vec<String>) -> RES {
		// self.stack won't call self.word unless it knows there is an element to eat
		let token = self.get(0).unwrap();
		match token.of.kind {
			Kind::ParenLF => self.tuple(env, params),
			Kind::SquarenLF => self.array(env, params),
			Kind::Post => self.function(env, params),
			Kind::Ref => self._ref(params),
			Kind::Number => self.number(),
			Kind::String => self.string(),
			_ => {
				return Err(format!("While processing word stack, unexpected token: {:?} was encountered", token))
			}
		}
	}

	fn graph(&mut self, index: Index, parent: Index) -> RES {
		let mut env: Env = HashMap::new();
		env.insert(String::from("(parent)"), parent);
		self.eat(Kind::BracketLF)?;
		let keys = self.keys(&mut env, index, &[Kind::BracketRT])?;
		self.eat(Kind::BracketRT)?;
		let graph = Element::Graph { env, keys };

		Ok(graph)
	}

	fn clone(&mut self, index: Index, parent: Index) -> RES {
		let mut env: Env = HashMap::new();
		env.insert(String::from("(parent)"), parent);
		let text = self.eat(Kind::Ref)?.meta.text.clone();
		let texts = text.split(".").map(str::to_string).collect();
		self.eat(Kind::BracketLF)?;
		let keys = self.keys(&mut env, index, &[Kind::BracketRT])?;
		self.eat(Kind::BracketRT)?;
		let clone = Element::Clone {
			texts,
			env,
			keys,
			index: 0,
		};
		Ok(clone)
	}

	fn tuple(&mut self, env: &mut Env, params: &mut Vec<String>) -> RES {
		self.eat(Kind::ParenLF)?;
		let tuple = Element::Tuple {
			body: self.stack(env, params, &[Kind::ParenRT])?,
		};
		self.eat(Kind::ParenRT)?;
		Ok(tuple)
	}

	fn array(&mut self, env: &mut Env, params: &mut Vec<String>) -> RES {
		self.eat(Kind::SquarenLF)?;
		let array = Element::Array {
			body: self.stack(env, params, &[Kind::SquarenRT])?,
		};
		self.eat(Kind::SquarenRT)?;
		Ok(array)
	}

	fn function(&mut self, env: &mut Env, params: &Vec<String>) -> RES {
		let mut count = params.len();
		let head = self.params(&mut count)?;
		let mut pars: Vec<String> = params.clone();
		for par in &head {
			match par {
				Element::Par { text, index } => pars.push(text.clone()),
				_ => {}
			}
		}
		let body = self.stack(
			env,
			&mut pars,
			&[Kind::BracketLF, Kind::ParenLF, Kind::Key, Kind::SquarenLF],
		)?;
		let function = Element::Function { head, body };
		Ok(function)
	}

	fn params(&mut self, count: &mut usize) -> RvES {
		let mut pars: Vec<Element> = Vec::new();
		self.eat(Kind::Post)?;
		while self.until(0, &[Kind::Post]) {
			pars.push(self.par(*count)?);
			*count += 1;
		}
		self.eat(Kind::Post)?;
		Ok(pars)
	}

	fn par(&mut self, index: Index) -> RES {
		let par_token = self.eat(Kind::Ref)?;
		Ok(Element::Par {
			text: par_token.meta.text.clone(),
			index,
		})
	}

	fn _ref(&mut self, params: &Vec<String>) -> RES {
		let ref_token = self.eat(Kind::Ref)?;
		let text = ref_token.meta.text.clone();
		let mut includes = false;
		let mut smarap = params.clone();
		smarap.reverse();
		//
		let mut index = 0;
		for (i, para) in smarap.iter().enumerate() {
			if para == &text {
				includes = true;
				index = (i as i64 - params.len() as i64 + 1).abs() as usize;
				break;
			}
		}

		let element = if includes {
			Element::Par { text, index }
		} else {
			// let textstr = text.split(".").collect::<Vec<&str>>();
			// let mut texts: Vec<String> = Vec::new();
			// for t in textstr {
			// 	texts.push(t.to_string())
			// }
			let texts = text.split(".").map(str::to_string).collect();

			Element::Ref { texts, index: 0 }
		};
		Ok(element)
	}

	fn number(&mut self) -> RES {
		let number_token = self.eat(Kind::Number)?;
		Ok(Element::Number {
			text: number_token.meta.text.clone(),
		})
	}

	fn string(&mut self) -> RES {
		unimplemented!();
		// Ok(Element::Nothing)
	}
}

// Point with key "" has already been used in this context

impl State {
	fn eat(&mut self, kind: Kind) -> Result<&Token, String> {
		match self.get(0) {
			Some(t) => {
				*self.cursor.borrow_mut() += 1;
				if t.of.kind == kind {
					Ok(t)
				} else {
					Err(format!(
						"UnexpectedToken: {:?} of {:?} on line {}\nExpected token of name: {:?}",
						t.meta.text, t.of.kind, t.meta.line, kind
					))
				}
			}
			None => Err("UnexpectedEndOfInput".to_string()),
		}
	}

	fn get(&self, offset: usize) -> Option<&Token> {
		if *self.cursor.borrow() + offset < self.tokens.len() {
			Some(&self.tokens[*self.cursor.borrow() + offset])
		} else {
			None
		}
	}

	fn is(&self, offset: usize, stop: Kind) -> bool {
		match self.get(offset) {
			Some(t) => t.of.kind == stop,
			None => false,
		}
	}

	fn until(&self, offset: usize, stops: &[Kind]) -> bool {
		match self.get(offset) {
			Some(t) => {
				for stop in stops {
					if t.of.kind == *stop {
						return false;
					}
				}
				true
			}
			None => false,
		}
	}
}
