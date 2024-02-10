#![feature(let_chains)]

use std::ops::Add;
use itertools::{PeekNth, peek_nth};

use wasm_bindgen::prelude::*;

#[cfg(test)]
mod test;

fn is_double_char(chr: &char)->bool{ ['&','|','=','<','>'].contains(chr) }
fn is_equal_type(chr: &char)->bool{ ['!','<','>'].contains(chr) }
fn is_expr_char(chr: &char)->bool{['@','#','<','>','-','[',']','$','£','+','.','*','{','}','(',')',':','/',',','!','"','%','Σ','→','='].contains(chr) || is_double_char(chr) || is_equal_type(chr) }
fn is_white_space_char(chr: &char)->bool{ [' ','\t','\r','\n'].contains(chr) }
fn is_number(chr: &char)->bool{ ['1','2','3','4','5','6','7','8','9','0'].contains(chr) }
fn is_letter(chr: &char)->bool{ ['a','b','c','d','e','f','g','h','i','j','k','l','m','n','o','p','q','r','s','t','u','v','w','x','y','z','A','B','C','D','E','F','G','H','I','J','K','L','M','N','O','P','Q','R','S','T','U','V','W','X','Y','Z'].contains(chr) }
fn is_all_valid_char(chr: &char)->bool{ is_expr_char(chr) || is_white_space_char(chr) || is_number(chr) || is_letter(chr) }

#[derive(PartialEq, Clone, Copy, Debug)]
#[wasm_bindgen]
pub struct Pos{
	pub line: usize,
	pub chr: usize,
}
impl Pos{fn new(line: usize, chr: usize)->Self{Pos{line, chr}}}
impl Add<usize> for Pos{
	type Output = Self;
	fn add(self, rhs: usize) -> Self{Self{line: self.line, chr: self.chr + rhs}}
}


#[derive(PartialEq, Clone, Copy, Debug)]
#[wasm_bindgen]
pub struct Range{
	pub start: Pos,
	pub end: Pos,
}
impl Range{
	fn from_first_last_token(first: &Token, last: &Token)->Self{Self{
		start: first.pos,
		end: Pos {
			line: last.pos.line,
			chr: last.pos.chr+last.content.chars().count()-1
		}
	}}
	fn from_pos(pos: Pos)->Self{Self{start: pos, end: pos}}
}
impl From<Pos> for Range{fn from(pos: Pos) -> Self {Self::from_pos(pos)}}
impl From<&Pos> for Range{fn from(pos: &Pos) -> Self {Self::from_pos(pos.clone())}}
impl From<&mut Pos> for Range{fn from(pos: &mut Pos) -> Self {Self::from_pos(pos.clone())}}

trait Ranged{fn get_range(&self)->&Range;}

#[derive(PartialEq, Clone, Debug)]
pub struct Token{
	pub content: String,
	pub pos: Pos,
}

#[wasm_bindgen]
#[derive(PartialEq, Clone, Debug)]
pub struct Error{
	error: String,
	location: Option<Range>,
}
impl Error{
	pub fn new(error: String, location: Option<Range>)->Self{Self{error, location}}
	pub fn as_string(&self, int: Interpreter)->String{
		// TODO: Handle tabs properly
		let empty = " ";
		let termination = "^";
		let underline = "=";
		let mut rv = String::new();
		rv += &format!("Encountered error `{}` in file {}", self.error, int.filename);
		if let Some(loc) = self.location{
			rv += &format!(":{}:{}\n", loc.start.line, loc.start.chr);
			let Some(data) = int.data else{return format!("ERROR: error refers to data which is not existent on the Interpreter passed in. This error was encountered whilst handling the error `{}`.",self.error)};
			let mut lines = data.split('\n');
			let Some(first_line) = lines.nth(loc.start.line-1) else{return format!("ERROR: error refers to line no. {}, which is not existent on the Interpreter passed in. This error was encountered whilst handling the error `{}`.",loc.start.line, self.error)};
			let first_line = first_line.replace("\t", "    ");
			let padding = loc.end.line.to_string().chars().count() + 1;
			let first_lineno_len = loc.start.line.to_string().chars().count();
			
			rv += &loc.start.line.to_string();
			rv += &empty.repeat(padding - first_lineno_len);
			rv += &first_line;
			rv += "\n";
			rv += &empty.repeat(loc.start.chr-1+padding);
			rv += termination;
			if loc.start.line == loc.end.line{if loc.end.chr > loc.start.chr{
				rv += &underline.repeat(loc.end.chr-loc.start.chr-1);
				rv += termination;
			}}else{
				rv+=&underline.repeat(first_line.chars().count() - loc.start.chr - 1);
				rv+="\n";
				if loc.end.line - loc.start.line - 1 > 5{
					if lines.nth(loc.end.line - loc.start.line - 2).is_none(){return format!("ERROR: error refers to line no. {}, which is not existent on the Interpreter passed in. This error was encountered whilst handling the error `{}`.",loc.end.line, self.error)}
					rv += &empty.repeat(padding);
					rv += "...\n";
				}else{for i in 0..loc.end.line - loc.start.line - 1{
					let Some(line) = lines.next() else{return format!("ERROR: error refers to line no. {}, which is not existent on the Interpreter passed in. This error was encountered whilst handling the error `{}`.",loc.start.line+i+1, self.error)};
					let line = line.replace("\t", "    ");
					let line_no_len = (loc.start.line+i+1).to_string().chars().count();

					rv += &(loc.start.line+i+1).to_string();
					rv += &empty.repeat(padding-line_no_len);
					rv += &line;
					rv += "\n";
					rv += &empty.repeat(padding);
					rv += &underline.repeat(line.chars().count());
					rv += "\n";
				}}
				let Some(line) = lines.next() else{return format!("ERROR: error refers to line no. {}, which is not existent on the Interpreter passed in. This error was encountered whilst handling the error `{}`.",loc.end.line, self.error)};
				let line = line.replace("\t", "    ");
				rv += &loc.end.line.to_string();
				rv += &empty.repeat(padding-1);
				rv += &line;
				rv += "\n";
				rv += &empty.repeat(padding);
				rv += &underline.repeat(loc.end.chr-1);
				rv += termination;
			}
		}
		rv += "\n";

		return rv;
	}
}
trait Parse {fn parse<'a>(token_gen: &mut PeekNth<impl Iterator<Item=&'a Token>>)->Result<Self, String> where Self: Sized;}

#[derive(Debug, PartialEq)]
struct RootExpr(pub Vec<OuterDef>);
impl Parse for RootExpr{fn parse<'a>(token_gen: &mut PeekNth<impl Iterator<Item=&'a Token>>)->Result<Self, String>{
	let mut root = RootExpr(vec!());
	while token_gen.peek().is_some(){
		root.0.push(OuterDef::parse(token_gen)?);
		println!("{:#?}",root.0.last().unwrap());
	}
	return Ok(root);
}}
#[derive(Debug, PartialEq)]
enum OuterDef{
	Colour{
		name: String,
		direction: Data,
		range: Range,
	},
	Peice{
		moves:Vec<Move>,
		events: Vec<Event>,
		checkable: bool,
		value: usize,
		range: Range,
	},
}

macro_rules! cond_where {($cond:expr; where $pat:pat = $expr:expr) => {
	if let $pat = $expr {$cond}else{false}
};}
impl Parse for OuterDef{fn parse<'a>(token_gen: &mut PeekNth<impl Iterator<Item=&'a Token>>)->Result<Self, String>{
	let Some(first_token) = token_gen.next() else {panic!()};
	if first_token.content != "<"{panic!()};
	let Some(kind_token) = token_gen.next() else{panic!()};
	return Ok(match kind_token.content.as_str(){
		"#"=>{
			let mut name = None;
			let mut direction = None;

			let last_token = loop{
				let Some(token) = token_gen.next() else{return Err(format!("mismatched `<` expected a closing tag for the opening angled bracket found at {:?}.", first_token.pos))};
				match token.content.as_str(){
					"name"=>{
						if !cond_where!(token.content == "="; where Some(token) = token_gen.next()){panic!();}
						name = Some(token_gen.next().unwrap().content.clone());
					}
					"direction"=>{
						if !cond_where!(token.content == "="; where Some(token) = token_gen.next()){panic!();}
						direction = Some(Data::parse(token_gen)?);
					}
					">"=>break token,
					_=>panic!()
				}
			};
			let Some(name) = name else{return Err(format!("TODOMSG"));};
			let Some(direction) = direction else{return Err(format!("TODOMSG"));};

			OuterDef::Colour {
				name,
				direction,
				range: Range::from_first_last_token(first_token, last_token)
			}
		}
		"@"=>{

			todo!();
		}
		v=>{return Err(String::from(v));}
	});
}}
impl Ranged for OuterDef{fn get_range(&self)->&Range {match self{
	OuterDef::Colour { name:_, direction:_, range } |
	OuterDef::Peice { moves:_, events:_, checkable:_, value:_, range } => range,
}}}

#[derive(Debug, PartialEq)]
struct Move{

}

#[derive(Debug, PartialEq)]
struct Event{

}
#[derive(Debug, PartialEq)]
enum Expresion{
	UnaryAddition(Data, Range),
	UnarySubtraction(Data, Range),
	BinaryAddition(Data, Data, Range),
	BinarySubtraction(Data, Data, Range),
	Multipication(Data, Data, Range),
	Division(Data, Data, Range),
}
const OOO: [&str; 4]=["+","-","*","/"];
macro_rules! next_ooo {($op: ident) => {OOO[OOO.iter().position(|o|o==&$op).expect("failed")+1]};}
impl Expresion{
	fn get_binary_expr(op: &str, lhs:Data, rhs:Data)->Expresion{
		let lhs_range = *lhs.get_range();
		let rhs_range = *rhs.get_range();
		return match op {
			"+"=>Expresion::BinaryAddition,
			"-"=>Expresion::BinarySubtraction,
			"*"=>Expresion::Multipication,
			"/"=>Expresion::Division,
			_=>panic!("invalid op"),
		}(lhs, rhs, Range{
			start: lhs_range.start,
			end: rhs_range.end,
		});
	}
	///parses the order of operations
	fn parse_ooo(operations: Vec<&Token>, mut data: Vec<Data>, op: &str)->Expresion{
		fn get_data(mut data_vec: Vec<Data>, ops: Vec<&Token>, op: &str)->Data
		{if data_vec.len() == 1{data_vec.pop().expect("just checke the length")}else{
			let data = Expresion::parse_ooo(ops, data_vec, op);
			let range = *data.get_range();
			Data::Expresion(data.into(),range)
		}}

		assert!(operations.len() == data.len() - 1);
		if data.len() == 2{
			debug_assert!(operations.len() == 1);
			let rhs = data.pop().expect("just checked that the length is enough");
			let lhs = data.pop().expect("just checked that the length is enough");
			return Self::get_binary_expr(&operations[0].content, lhs, rhs);
		}
		return if let Some(split) = operations.iter().position(|o|o.content==op){
			let rhs_data = data.split_off(split+1);
			let lhs_data = data;
			let mut lhs_ops = operations;
			let rhs_ops = lhs_ops.split_off(split+1);
			let pop_op = lhs_ops.pop();
			debug_assert_eq!(pop_op.unwrap().content, op);

			Self::get_binary_expr(
				op,
				get_data(lhs_data,lhs_ops,op),
				get_data(rhs_data,rhs_ops,op),
			)
		}else{Self::parse_ooo(operations, data, next_ooo!(op))};
	}

	fn parse<'a>(
		lhs: Data,
		token_gen: &mut PeekNth<impl Iterator<Item=&'a Token>>
	)->Result<Result<Self, String>, Data>{
		let Some(op_token) = token_gen.peek()else{return Err(lhs);};
		if !OOO.contains(&op_token.content.as_str()){return Err(lhs);}
		let Some(op_token) = token_gen.next()else{panic!("peek failed")};

		let rhs = match
			if let Some(unary) = Data::parse_unary(token_gen){unary}else{Data::parse_without_expr(token_gen)}
		{Ok(v) => v,Err(e) => return Ok(Err(e)),};

		let mut operations = vec![op_token];
		let mut data = vec![lhs,rhs];

		loop{
			let Some(op_token) = token_gen.peek()else{break;};
			if !OOO.contains(&op_token.content.as_str()){break;};
			let Some(op_token) = token_gen.next()else{panic!("peek failed")};
			operations.push(op_token);

			data.push(match
				if let Some(unary) = Data::parse_unary(token_gen){unary}else{Data::parse_without_expr(token_gen)}
			{Ok(v) => v,Err(e) => return Ok(Err(e)),});
		}

		return Ok(Ok(Self::parse_ooo(operations, data, OOO[0])));
	}
	fn parse_unary<'a>(token_gen: &mut PeekNth<impl Iterator<Item=&'a Token>>)->Option<Result<Self, String>>{
		let mut unary_tokens = vec!();
		while let Some(token) = token_gen.next_if(|t|
			t.content == "-" ||
			t.content == "+"
		){unary_tokens.push(token);}

		let mut rv: Option<Self> = None;
		for token in unary_tokens.iter().rev(){
			let base = match rv{
				None => match Data::parse_without_expr(token_gen){
					Ok(v)=>v,Err(e)=>return Some(Err(e))
				},Some(v) =>{
					let range = *v.get_range();
					Data::Expresion(v.into(),range)
				}
			};
			let mut range = base.get_range().clone();
			range.start = token.pos;

			rv = Some(match token.content.as_str(){
				"-"=>Expresion::UnarySubtraction,
				"+"=>Expresion::UnaryAddition,
				_=>panic!()
			}(base, range));
		}
		return rv.map(|v|Ok(v));
	}
}
impl Ranged for Expresion{fn get_range(&self)->&Range{match self{
	Expresion::UnaryAddition(_, r) |
	Expresion::UnarySubtraction(_, r) |
	Expresion::BinaryAddition(_, _, r) |
	Expresion::BinarySubtraction(_, _, r) |
	Expresion::Multipication(_, _, r) |
	Expresion::Division(_, _, r) => r,
}}}
#[derive(Debug, PartialEq)]
struct Variable{
	route: Vec<String>,
	range: Range,
}
impl Parse for Variable{fn parse<'a>(token_gen: &mut PeekNth<impl Iterator<Item=&'a Token>>)->Result<Self, String>{
	let Some(first_token) = token_gen.next() else{panic!()};
	let mut route = vec!();
	let mut curr_token = first_token;
	loop{
		let Some(first_char) = curr_token.content.chars().next()else{panic!()};
		if !is_letter(&first_char){panic!()}
		route.push(curr_token.content.clone());
		let Some(sep) = token_gen.peek()else{break;};
		if sep.content != "."{break;}
		token_gen.next().expect("peek failed");
		curr_token = match token_gen.next(){Some(v)=>v,None=>panic!()};
	}
	Ok(Self{route, range: Range::from_first_last_token(first_token, curr_token)})
}}
impl Ranged for Variable{fn get_range(&self)->&Range{&self.range}}

#[derive(Debug, PartialEq)]
enum Data{
	Expresion(Box<Expresion>, Range),
	Value(isize, Range),
	Variable(Variable, Range),
}
impl Data{
	/// this can acctualy include an expr but only when it is contained within brackets
	fn parse_without_expr<'a>(token_gen: &mut PeekNth<impl Iterator<Item=&'a Token>>)->Result<Self, String>{
		if token_gen.next_if(|token|token.content == "(").is_some(){
			let rv = Data::parse(token_gen)?;
			return if
				let Some(token) = token_gen.next()
				&& token.content == ")"
			{Ok(rv)}else{panic!("{:?}",rv)}
		}

		let Some(first_token) = token_gen.peek() else{panic!()};
		let Some(first_char) = first_token.content.chars().next() else{panic!()};
		if is_letter(&first_char){
			let var = Variable::parse(token_gen)?;
			let range = var.range;
			return Ok(Data::Variable(var,range));
		}else if is_number(&first_char){
			let first_token = token_gen.next().expect("peek failed");
			return Ok(Data::Value(first_token.content.parse().map_err(
				|_|format!("Failed to parse int {}",first_token.content)
			)?, Range::from_first_last_token(first_token, first_token)))
		}
		panic!()
	}
	fn parse_unary<'a>(token_gen: &mut PeekNth<impl Iterator<Item=&'a Token>>)->Option<Result<Self, String>>{
		let Some(unary) = Expresion::parse_unary(token_gen) else{return None;};
		let unary = match unary{Ok(v)=>v,Err(e)=>return Some(Err(e))};
		let range = *unary.get_range();
		Some(Ok(Data::Expresion(unary.into(), range)))
	}
}
impl Parse for Data{fn parse<'a>(token_gen: &mut PeekNth<impl Iterator<Item=&'a Token>>)->Result<Self, String>{
	let first_data = if let Some(unary) = Self::parse_unary(token_gen){unary?}else{
		Self::parse_without_expr(token_gen)?
	};
	
	Ok(match Expresion::parse(first_data,token_gen){Ok(expr)=>{
		let expr = expr?;
		let range = *expr.get_range();
		Data::Expresion(expr.into(),range)
	}, Err(first_data)=>first_data})
}}
impl Ranged for Data{fn get_range(&self)->&Range{match self{
	Data::Expresion(_, r) |
	Data::Value(_, r) |
	Data::Variable(_, r) => r,
}}}
#[wasm_bindgen]
#[derive(Debug)]
pub struct Interpreter{
	#[wasm_bindgen(skip)]
	pub data: Option<String>,
	tokens: Option<Vec<Token>>,
	syntax_tree: Option<RootExpr>,
	#[wasm_bindgen(skip)]
	pub filename: String,
}

#[wasm_bindgen]
impl Interpreter{
	#[wasm_bindgen(getter = data)]
	pub fn _data(&self) -> Option<String> {self.data.clone()}
	#[wasm_bindgen(setter = data)]
	pub fn _set_data(&mut self, data: Option<String>) {self.data = data;}

	#[wasm_bindgen(getter = filename)]
	pub fn _filename(&self) -> String {self.filename.clone()}
	#[wasm_bindgen(setter = filename)]
	pub fn _set_filename(&mut self, filename: String) {self.filename = filename;}

	#[wasm_bindgen(constructor)]
	///constructs a new Interpreter
	pub fn new(filename: String)->Self{Interpreter{
		data: None,
		tokens: None,
		syntax_tree: None,
		filename: filename,
	}}
	///preforms the lexing. Requires data to be not None
	pub fn lexer(&mut self) -> Result<(),Error>{
		if self.data.is_none(){return Err(Error::new("Lexer Error: no data, Data must be added before lexing can start".to_string(),None));}
		
		let mut tokens = Vec::new();
		let pos = &mut Pos{
			line: 1,
			chr: 0,
		};
		let mut curr_expr = String::new();
		let mut curr_expr_pos = None;
		let mut in_inline_comment = false;
		let mut in_comment = false;

		for chr in self.data.as_ref().unwrap().chars(){
			pos.chr+=1;
			if !is_all_valid_char(&chr){return Err(Error::new(format!("Lexer Error: invalid char '{}'", chr),Some(pos.into())));}
			if chr == '\n'{
				pos.line+=1;
				pos.chr=0;
				if in_comment{
					in_comment = false;
					curr_expr = String::new();
					curr_expr_pos = None;
				}
			}
			if in_inline_comment && {
				//if the last two chars == "*/" note the check is flipped as that is the order the itterator works
				let mut chars = curr_expr.chars();
				chars.next_back() == Some('/') && chars.next_back() == Some('*')
			}{
				in_inline_comment=false;
				curr_expr=String::new();
				curr_expr_pos=None;
			}
			if in_comment || in_inline_comment{
				curr_expr+=&chr.to_string();
				continue;
			}

			if
				chr == '*' &&
				tokens.last() == Some(&Token{
					content: "/".to_string(),
					pos: Pos::new(pos.line, pos.chr-1)
				})
			{
				in_inline_comment = true;
				tokens.pop();
				curr_expr=String::new();
				curr_expr_pos = None;
			}else if
				chr == '/' &&
				tokens.last() == Some(&Token{
					content: "/".to_string(),
					pos: Pos::new(pos.line, pos.chr-1)
				})
			{
				in_comment = true;
				tokens.pop();
				curr_expr = String::new();
				curr_expr_pos = None;
			}else if is_expr_char(&chr){
				if curr_expr.len()>0{
					tokens.push(Token{content: curr_expr, pos: curr_expr_pos.unwrap(),});
					curr_expr=String::new();
					curr_expr_pos = None;
				}
				if is_double_char(&chr) && tokens.last() == Some(&Token{
					content: chr.to_string(),
					pos: Pos::new(pos.line, pos.chr-1)
				}) {tokens.last_mut().unwrap().content += &chr.to_string()}
				else if
					chr=='=' && if let Some(last) = tokens.last(){
						last.content.chars().count() == 1 &&
						is_equal_type(&last.content.chars().next().unwrap()) &&
						last.pos == (Pos::new(pos.line, pos.chr-1))
					}else{false}
				{tokens.last_mut().unwrap().content += &chr.to_string();} else{
					tokens.push(Token{content: chr.to_string(), pos: pos.clone(),});
					curr_expr=String::new();
					curr_expr_pos = None;
				}
			}else if is_white_space_char(&chr){
				if curr_expr.len()>0 {
					tokens.push(Token{content: curr_expr, pos: curr_expr_pos.unwrap(),});
					curr_expr=String::new();
					curr_expr_pos = None;
				}
			}else{
				if
					let Some(first_chr) = curr_expr.chars().next()
					&& is_number(&first_chr)
					&& is_letter(&chr)
				{
					tokens.push(Token{content: curr_expr, pos: curr_expr_pos.unwrap(),});
					curr_expr=String::new();
					curr_expr_pos = None;
				}
				if curr_expr_pos==None { curr_expr_pos=Some(pos.clone()); }
				curr_expr+=&chr.to_string()
			}
		}
		if curr_expr.len()>0 && !in_comment && !in_inline_comment{
			tokens.push(Token{content: curr_expr, pos: curr_expr_pos.unwrap(),});
		}
		self.tokens = Some(tokens);
		return Ok(());
	}
	pub fn parser(&mut self) -> Result<(),String>{
		let Some(ref tokens) = self.tokens else {
			return Err("Tokeniser Error: no tokens, Tokens must be added or generated before tokenising can start".to_string());
		};

		let mut token_gen = peek_nth(tokens.iter());
		let root =  RootExpr::parse(&mut token_gen);
		todo!();//"{:#?}, {:#?}",token_gen, root);

	}
	pub fn get_debug(&self)->String{format!("{:#?}",self)}
	fn pretty_print_pos(&self, pos: &Pos)->String{format!("{}:{}:{}", self.filename, pos.line, pos.chr)}
}