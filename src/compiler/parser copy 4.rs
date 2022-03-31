use super::tokenizer::{Kind, Token};
use std::cell::RefCell;
use std::collections::HashMap;

pub type Index = usize;

pub type EMap = HashMap<String, Element>;
pub type TMap = HashMap<String, Element>;
pub type PMap = HashMap<String, Point>;

#[derive(Debug, Clone)]
pub struct Element {
	// meta: Meta,
	item: Item,
	index: usize,
	// keychain: Vec<String>,
	text: String,
	head: Vec<Element>,
	body: Vec<Element>,
}

impl Element {
	fn new(item: Item) -> Element {
		Element {
			// meta: Meta { col: 0, row: 0 },
			item,
			index: 0,
			// keychain: Vec::new(),
			text: String::new(),
			head: Vec::new(),
			body: Vec::new(),
		}
	}
}

#[derive(Debug, Clone)]
pub struct Point {
	// meta: Meta,
	pub node: Node,
	pub key: String,
	pub r#ref: String,
	pub points: Vec<String>,
}

impl Point {
	fn new(node: Node) -> Point {
		Point {
			// meta: Meta { col: 0, row: 0 },
			node,
			r#ref: String::new(),
			key: String::new(),
			points: Vec::new(),
		}
	}
}

#[derive(Debug, Clone, PartialEq)]
pub enum Node {
	// Nothing,
	//
	Graph,
	Clone,
	Stack,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Item {
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
	pmap: PMap,
	emap: EMap,
	tmap: TMap,
}

pub fn parser(
	filename: &String,
	tokens: Vec<Token>,
) -> Result<(PMap, EMap, TMap), String> {
	let mut state = State {
		cursor: RefCell::new(0),
		tokens,
		keychain: Vec::new(),
		pmap: HashMap::new(),
		emap: HashMap::new(),
		tmap: HashMap::new(),
	};

	state.program(filename)?;
	Ok((state.pmap, state.emap, state.tmap))
	// Ok(state.points)
	// Ok()
}

impl State {
	fn program(&mut self, filename: &String) -> Result<Point, String> {
		// self.keychain.push(filename.clone());
		let mut program = self.points(&[])?;
		// program.key = self.keychain.join(".");
		Ok(program)
	}

	fn points(&mut self, stop: &[Kind]) -> Result<Point, String> {
		let mut graph = Point::new(Node::Graph);

		while self.until(0, stop) {
			// needs to filter for key redundancy
			graph.points.push(self.point()?.key.clone());
		}

		Ok(graph)
	}
	fn point(&mut self) -> Result<Point, String> {
		if self.is(0, Kind::Typ) {
			self.typ()
		} else {
			self.key()
		}
	}
	fn typ(&mut self) -> Result<Point, String> {
		let key_token = self.eat(Kind::Typ)?;
		let key = key_token.text[..key_token.text.len() - 1].to_string();

		self.keychain.push(key);
		let mut point = if self.is(0, Kind::BracketLF) {
			self.graph()?
		} else if self.is(0, Kind::Ref) && self.is(1, Kind::BracketLF) {
			self.clone()?
		} else {
			self.typed()?
		};

		let keystring = self.keychain.join(".");
		point.key = keystring.clone();
		self.pmap.insert(keystring, point.clone());

		self.keychain.pop();

		Ok(point)
	}

	fn key(&mut self) -> Result<Point, String> {
		let key_token = self.eat(Kind::Key)?;
		let key = key_token.text[..key_token.text.len() - 1].to_string();

		self.keychain.push(key);
		let mut point = self.stack()?;
		let keystring = self.keychain.join(".");
		point.key = keystring.clone();
		self.pmap.insert(keystring, point.clone());

		self.keychain.pop();

		Ok(point)
	}

	// typed is type stack, item stack
	fn typed(&mut self) -> Result<Point, String> {
		let point = Point::new(Node::Stack);
		let keystring = self.keychain.join(".");

		let mut typ = Element::new(Item::Typ);
		typ.body = self.elements(
			&Vec::new(),
			&[
				Kind::BracketRT,
				Kind::Key,
				Kind::Typ,
				Kind::Com,
				Kind::Arrow,
			],
		)?;

		if self.is(0, Kind::Arrow) {
			self.eat(Kind::Arrow)?;
			typ.head = typ.body;
			typ.body = self.elements(
				&Vec::new(),
				&[
					Kind::BracketRT,
					Kind::Key,
					Kind::Typ,
					Kind::Com,
					Kind::Arrow,
				],
			)?;
		}

		self.tmap.insert(keystring.clone(), typ);
		if self.is(0, Kind::Com) {
			self.eat(Kind::Com)?;
		}
		let stack = self.tuple(&Vec::new(), &[Kind::BracketRT, Kind::Key, Kind::Typ])?;
		self.emap.insert(keystring, stack);

		Ok(point)
	}
	// item stack
	fn stack(&mut self) -> Result<Point, String> {
		let point = Point::new(Node::Stack);
		let keystring = self.keychain.join(".");

		let stack = self.tuple(&Vec::new(), &[Kind::BracketRT, Kind::Key, Kind::Typ])?;
		self.emap.insert(keystring, stack);

		Ok(point)
	}
	fn graph(&mut self) -> Result<Point, String> {
		self.eat(Kind::BracketLF)?;
		let points = self.points(&[Kind::BracketRT])?;
		self.eat(Kind::BracketRT)?;
		Ok(points)
	}
	fn clone(&mut self) -> Result<Point, String> {
		let text = self.eat(Kind::Ref)?.text.clone();
		self.eat(Kind::BracketLF)?;
		let mut points = self.points(&[Kind::BracketRT])?;
		self.eat(Kind::BracketRT)?;
		points.node = Node::Clone;
		points.r#ref = text;
		Ok(points)
	}
	fn tuple(&mut self, pars: &Vec<String>, stop: &[Kind]) -> Result<Element, String> {
		let mut tuple = Element::new(Item::Tuple);

		while self.until(0, stop) {
			tuple.body.push(self.element(pars)?);
		}

		Ok(tuple)
	}

	fn elements(
		&mut self,
		pars: &Vec<String>,
		stop: &[Kind],
	) -> Result<Vec<Element>, String> {
		let mut elements = Vec::new();

		while self.until(0, stop) {
			elements.push(self.element(pars)?);
		}

		Ok(elements)
	}

	fn element(&mut self, pars: &Vec<String>) -> Result<Element, String> {
		let token = self.get(0).unwrap();
		match token.kind {
			Kind::ParenLF => self.paren(pars),
			Kind::SquarenLF => self.array(pars),
			Kind::Post => self.function(pars),
			Kind::Ref => self._ref(pars),
			// Kind::Number => self.number(),
			Kind::Integer => self.integer(),
			Kind::Decimal => self.decimal(),
			// Kind::String => self.string(),
			_ => {
				return Err(format!(
					"While processing word stack, unexpected token: {:?} was encountered",
					token
				))
			}
		}
	}

	fn paren(&mut self, pars: &Vec<String>) -> Result<Element, String> {
		self.eat(Kind::ParenLF)?;
		let tuple = self.tuple(pars, &[Kind::ParenRT])?;
		self.eat(Kind::ParenRT)?;
		Ok(tuple)
	}

	fn array(&mut self, pars: &Vec<String>) -> Result<Element, String> {
		self.eat(Kind::SquarenLF)?;
		let mut array = self.tuple(pars, &[Kind::SquarenRT])?;
		array.item = Item::Array;
		self.eat(Kind::SquarenRT)?;
		Ok(array)
	}

	fn function(&mut self, pars: &Vec<String>) -> Result<Element, String> {
		let head = self.pars(&mut pars.len())?;
		let mut pars: Vec<String> = pars.clone();
		for par in &head {
			pars.push(par.text.clone());
		}
		let mut func = self.tuple(
			&mut pars,
			&[
				Kind::BracketRT,
				Kind::ParenRT,
				Kind::Key,
				Kind::Typ,
				Kind::Com,
				Kind::SquarenRT,
			],
		)?;
		func.head = head;
		func.item = Item::Function;
		Ok(func)
	}

	fn pars(&mut self, count: &mut usize) -> Result<Vec<Element>, String> {
		let mut pars: Vec<Element> = Vec::new();
		self.eat(Kind::Post)?;
		while self.until(0, &[Kind::Post]) {
			pars.push(self.par(*count)?);
			*count += 1;
		}
		self.eat(Kind::Post)?;
		Ok(pars)
	}

	fn par(&mut self, index: Index) -> Result<Element, String> {
		let token = self.eat(Kind::Ref)?;
		let mut element = Element::new(Item::Par);
		element.index = index;
		element.text = token.text.clone();
		Ok(element)
	}

	fn _ref(&mut self, pars: &Vec<String>) -> Result<Element, String> {
		let ref_token = self.eat(Kind::Ref)?;
		let text = ref_token.text.clone();
		let mut includes = false;
		let mut smarap = pars.clone();
		smarap.reverse();
		//
		let mut index = 0;
		for (i, para) in smarap.iter().enumerate() {
			if para == &text {
				includes = true;
				index = (i as i64 - pars.len() as i64 + 1).abs() as usize;
				break;
			}
		}

		let mut element = Element::new(Item::Nothing);
		element.text = text;
		if includes {
			element.item = Item::Par;
			element.index = index;
		} else {
			element.item = Item::Ref;
			// element.keychain = [
			// 	self.keychain.clone(),
			// 	element.text.split(".").map(str::to_string).collect(),
			// ]
			// .concat();
		}
		Ok(element)
	}

	fn integer(&mut self) -> Result<Element, String> {
		let token = self.eat(Kind::Integer)?;
		let mut number = Element::new(Item::Integer);
		number.text = token.text.clone();
		Ok(number)
	}

	fn decimal(&mut self) -> Result<Element, String> {
		let token = self.eat(Kind::Decimal)?;
		let mut number = Element::new(Item::Decimal);
		number.text = token.text.clone();
		Ok(number)
	}
}

impl State {
	fn eat(&mut self, kind: Kind) -> Result<&Token, String> {
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

	fn is(&self, offset: usize, stop: Kind) -> bool {
		match self.get(offset) {
			Some(t) => t.kind == stop,
			None => false,
		}
	}
	fn _any(&self, offset: usize, kinds: &[Kind]) -> bool {
		for kind in kinds {
			if self.is(offset, *kind) {
				return true;
			}
		}
		false
	}

	fn until(&self, offset: usize, stops: &[Kind]) -> bool {
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
