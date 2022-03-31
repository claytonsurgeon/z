use super::tokenizer::{Kind as TKind, Token};
use std::cell::RefCell;
use std::collections::HashMap;

pub type Index = usize;

pub type EMap = HashMap<String, Element>;
pub type TMap = HashMap<String, Element>;
pub type NMap = HashMap<String, Network>;

#[derive(Debug, Clone)]
pub struct Element {
	// meta: Meta,
	kind: Kind,
	para: usize,
	text: String,
	head: Vec<Element>,
	body: Vec<Element>,
}

impl Element {
	fn new(kind: Kind) -> Element {
		Element {
			// meta: Meta { col: 0, row: 0 },
			kind,
			para: 0,
			text: String::new(),
			head: Vec::new(),
			body: Vec::new(),
		}
	}
}

#[derive(Debug, Clone)]
pub struct Network {
	// meta: Meta,
	pub node: Node,
	pub text: String,
	pub path: String,
	pub kids: Vec<String>,
}

impl Network {
	fn new(node: Node) -> Network {
		Network {
			// meta: Meta { col: 0, row: 0 },
			node,
			text: String::new(),
			path: String::new(),
			kids: Vec::new(),
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
	// Nothing,
	//
	Graph,
	Clone,
	Point,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
	Nothing,
	//
	Tuple,
	Array,
	//
	Par,
	Ref,
	Pun,
	Typ,
	//
	// Number,
	Integer,
	Decimal,
	// String,
	//
	Function,
	// Operator,
}

struct State {
	cursor: RefCell<usize>,
	tokens: Vec<Token>,
	keychain: Vec<String>,
	nmap: NMap,
	emap: EMap,
	tmap: TMap,
}

pub fn parser(
	filename: &String,
	tokens: Vec<Token>,
) -> Result<(NMap, EMap, TMap), String> {
	let mut state = State {
		cursor: RefCell::new(0),
		tokens,
		keychain: Vec::new(),
		nmap: HashMap::new(),
		emap: HashMap::new(),
		tmap: HashMap::new(),
	};

	state.program(filename)?;
	Ok((state.nmap, state.emap, state.tmap))
	// Ok(state.Networks)
	// Ok()
}

impl State {
	fn program(&mut self, filename: &String) -> Result<Network, String> {
		unimplemented!();
		// self.keychain.push(filename.clone());
		let mut program = self.networks(&[])?;
		// program.key = self.keychain.join(".");
		Ok(program)
	}

	fn networks(&mut self, stop: &[TKind]) -> Result<Network, String> {
		let mut graph = Network::new(Node::Graph);

		while self.until(0, stop) {
			graph.kids.push(self.network()?.path.clone());
		}
		graph.kids.sort();
		graph.kids.dedup();

		Ok(graph)
	}

	fn network(&mut self) -> Result<Network, String> {
		let typ = self.is(0, TKind::Typ);
		let text = if typ {
			let token = self.eat(TKind::Typ)?;
			token.text[..token.text.len() - 1].to_string()
		} else {
			let token = self.eat(TKind::Key)?;
			token.text[..token.text.len() - 1].to_string()
		};

		self.keychain.push(text);

		let network = if typ {
			if self.is(0, TKind::BracketLF) {
				self.graph()?
			} else if self.is(0, TKind::Ref) && self.is(1, TKind::BracketLF) {
				self.clone()?
			} else {
				self.typed()?
			}
		} else {
			self.point()?
		};

		network.path = self.keychain.join(".");
		self.nmap.insert(network.path.clone(), network.clone());

		self.keychain.pop();

		Ok(network)
	}

	fn typed(&mut self) -> Result<Network, String> {
		let mut typ = Element::new(Kind::Typ);

		// typ.body = self.elements(
		// 	&Vec::new(),
		// 	&[
		// 		TKind::BracketRT,
		// 		TKind::Key,
		// 		TKind::Typ,
		// 		TKind::Com,
		// 		TKind::Arrow,
		// 	],
		// )?;

		// if self.is(0, TKind::Arrow) {
		// 	self.eat(TKind::Arrow)?;
		// 	typ.head = typ.body;
		// 	typ.body = self.elements(
		// 		&Vec::new(),
		// 		&[
		// 			TKind::BracketRT,
		// 			TKind::Key,
		// 			TKind::Typ,
		// 			TKind::Com,
		// 			TKind::Arrow,
		// 		],
		// 	)?;
		// }

		self.tmap.insert(self.keychain.join("."), typ);

		if self.is(0, TKind::Com) {
			self.eat(TKind::Com)?;
		}
		self.point()?;

		Ok(Network::new(Node::Point))
	}

	fn point(&mut self) -> Result<Network, String> {
		let network = Network::new(Node::Point);
		let keystring = self.keychain.join(".");

		let stack = self.stack(&Vec::new(), &[TKind::BracketRT, TKind::Key, TKind::Typ])?;
		self.emap.insert(keystring, stack);

		Ok(network)
	}

	fn graph(&mut self) -> Result<Network, String> {
		unimplemented!();
	}

	fn clone(&mut self) -> Result<Network, String> {
		unimplemented!();
	}

	fn stack(&mut self, pars: &Vec<String>, stop: &[TKind]) -> Result<Element, String> {
		let mut element = Element::new(Kind::Tuple);
		element.body = self.elements(pars, stop)?;

		Ok(element)
	}
	fn elements(
		&mut self,
		pars: &Vec<String>,
		stop: &[TKind],
	) -> Result<Vec<Element>, String> {
		let mut elements = Vec::new();

		while self.until(0, stop) {
			elements.push(self.element(pars)?);
		}

		Ok(elements)
	}

	fn element(&mut self, pars: &Vec<String>) -> Result<Element, String> {
		unimplemented!();
	}
}

impl State {
	fn eat(&mut self, kind: TKind) -> Result<&Token, String> {
		match self.get(0) {
			Some(t) => {
				*self.cursor.borrow_mut() += 1;
				if t.kind == kind {
					Ok(t)
				} else {
					Err(format!(
						"UnexpectedToken: {:?} of {:?} on line {}\nExpected token of name: {:?}",
						t.text, t.kind, t.meta.row, kind
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

	fn is(&self, offset: usize, stop: TKind) -> bool {
		match self.get(offset) {
			Some(t) => t.kind == stop,
			None => false,
		}
	}
	fn _any(&self, offset: usize, kinds: &[TKind]) -> bool {
		for kind in kinds {
			if self.is(offset, *kind) {
				return true;
			}
		}
		false
	}

	fn until(&self, offset: usize, stops: &[TKind]) -> bool {
		match self.get(offset) {
			Some(t) => {
				for stop in stops {
					if t.kind == *stop {
						return false;
					}
				}
				true
			}
			None => false,
		}
	}
}
