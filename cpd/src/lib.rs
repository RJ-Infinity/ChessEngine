#![feature(let_chains)]
#![feature(option_take_if)]
#![feature(get_many_mut)]

use std::ops::{Add, Sub};
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
impl Sub<usize> for Pos{
	type Output = Self;
	fn sub(self, rhs: usize) -> Self{Self{line: self.line, chr: self.chr - rhs}}
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
	fn from_token(token: &Token)->Self{Self {
		start: token.pos,
		end: token.pos+token.content.chars().count()-1
	}}
	fn from_pos(pos: Pos)->Self{Self{start: pos, end: pos}}
}
impl From<Pos> for Range{fn from(pos: Pos) -> Self {Self::from_pos(pos)}}
impl From<&Pos> for Range{fn from(pos: &Pos) -> Self {Self::from_pos(pos.clone())}}
impl From<&mut Pos> for Range{fn from(pos: &mut Pos) -> Self {Self::from_pos(pos.clone())}}
impl From<Token> for Range{fn from(token: Token) -> Self {Self::from_token(&token)}}
impl From<&Token> for Range{fn from(token: &Token) -> Self {Self::from_token(token)}}
impl From<&mut Token> for Range{fn from(token: &mut Token) -> Self {Self::from_token(token)}}


trait Ranged{
	fn get_range(&self)->&Range;
	fn set_range<T:FnOnce(&Range)->Range>(&mut self, setter: T);
}

#[derive(PartialEq, Clone, Debug)]
pub struct Token{
	pub content: String,
	pub pos: Pos,
}

#[wasm_bindgen]
#[derive(PartialEq, Clone, Debug)]
pub struct UserMessage{
	message: String,
	location: Option<Range>,
	note: Option<Box<UserMessage>>,
}
impl UserMessage{
	pub fn new(message: impl Into<String>, location: impl Into<Range>)->Self{Self{
		message: message.into(),
		location: Some(location.into()),
		note: None,
	}}
	pub fn new_with_note(message: impl Into<String>, location: Option<Range>, note: UserMessage)->Self{Self{
		message: message.into(),
		location,
		note: Some(note.into()),
	}}
	pub fn error_string(&self, int: &Interpreter, message_prefix: Option<&str>)->String{
		// TODO: Handle tabs properly
		let empty = " ";
		let termination = "^";
		let underline = "=";
		let mut rv = String::new();
		rv += &format!(
			"{} `{}` in file {}",
			message_prefix.unwrap_or("Encountered error"),
			self.message,
			int.filename
		);
		if let Some(loc) = self.location{
			rv += &format!(":{}:{}\n", loc.start.line, loc.start.chr);
			let Some(ref data) = int.data else{return format!("ERROR: error refers to data which is not existent on the Interpreter passed in. This error was encountered whilst handling the message `{}`.",self.message)};
			let mut lines = data.split('\n');
			let Some(first_line) = lines.nth(loc.start.line-1) else{return format!("ERROR: error refers to line no. {}, which is not existent on the Interpreter passed in. This error was encountered whilst handling the message `{}`.",loc.start.line, self.message)};
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
					if lines.nth(loc.end.line - loc.start.line - 2).is_none(){return format!("ERROR: error refers to line no. {}, which is not existent on the Interpreter passed in. This error was encountered whilst handling the message `{}`.",loc.end.line, self.message)}
					rv += &empty.repeat(padding);
					rv += "...\n";
				}else{for i in 0..loc.end.line - loc.start.line - 1{
					let Some(line) = lines.next() else{return format!("ERROR: error refers to line no. {}, which is not existent on the Interpreter passed in. This error was encountered whilst handling the message `{}`.",loc.start.line+i+1, self.message)};
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
				let Some(line) = lines.next() else{return format!("ERROR: error refers to line no. {}, which is not existent on the Interpreter passed in. This error was encountered whilst handling the message `{}`.",loc.end.line, self.message)};
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

		if let Some(note) = &self.note{rv += &note.error_string(int, Some("with note"))}

		return rv;
	}
}
impl From<String> for UserMessage{fn from(value: String) -> Self {Self{
	message: value,
	location: None,
	note: None,
}}}
impl From<&str> for UserMessage{fn from(value: &str) -> Self {Self{
	message: value.to_string(),
	location: None,
	note: None,
}}}
trait Parse {fn parse<'a>(token_gen: &mut PeekNth<impl Iterator<Item=&'a Token>>)->Result<Self, UserMessage> where Self: Sized;}

#[derive(Debug, PartialEq)]
struct RootExpr(pub Vec<OuterDef>);
impl Parse for RootExpr{fn parse<'a>(token_gen: &mut PeekNth<impl Iterator<Item=&'a Token>>)->Result<Self, UserMessage>{
	let mut root = RootExpr(vec!());
	while token_gen.peek().is_some(){root.0.push(OuterDef::parse(token_gen)?)}
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
		name: String,
		moves:List<Move>,
		events: List<Event>,
		checkable: Data,
		value: Data,
		range: Range,
	},
}

macro_rules! properties {($token_gen:expr, $first_token: expr, $constructor: path, {$($prop: ident => $parse: expr;)*}) => {{
	let Some(name_token) = $token_gen.next()else{return Err("expected a name however reached the end of the file".into())};
	if !is_letter(&name_token.content.chars().next().expect("the lexer shouldnt emit empty token")){
		return Err(UserMessage::new("expected the name to be made up of characters", name_token));
	}
	$(
		let mut $prop = None;
	)?
	let last_token = loop{
		let Some(token) = $token_gen.next() else{return Err(UserMessage::new("mismatched `<` expected a closing tag for the opening angled bracket.", $first_token.pos))};
		match token.content.as_str(){
			$(
				stringify!($prop)=>{
					let Some(token) = $token_gen.next() else{return Err("expected `=` hoever reached the end of the file".into())};
					if token.content != "="{return Err(UserMessage::new(format!("expected `=` however got `{}`",token.content), token))}
					$prop = Some($parse)
				},
			)?
			">"=>break token,
			_=>return Err(UserMessage::new(format!("invalid token `{}` in the definition",token.content), token)),
		}
	};
	let range = Range::from_first_last_token($first_token, last_token);
	$(
		let Some($prop) = $prop else{return Err(UserMessage::new(format!("expected the property {} in the definition.",stringify!($prop)),range));};
	)?
	$constructor{
		name: name_token.content.clone(),
		$(
			$prop,
		)?
		range
	}
}};}
impl Parse for OuterDef{fn parse<'a>(token_gen: &mut PeekNth<impl Iterator<Item=&'a Token>>)->Result<Self, UserMessage>{
	let Some(first_token) = token_gen.next() else {panic!("EOF")};// this condition should be handled by root Expr
	if first_token.content != "<"{return Err(UserMessage::new(format!("expected `<` to open a definition however got `{}`",first_token.content),first_token))};
	let Some(kind_token) = token_gen.next() else{return Err("expected a definition type however reached the end of the file".into())};
	return Ok(match kind_token.content.as_str(){
		"#"=>properties!(token_gen, first_token, OuterDef::Colour, {
			direction=>Data::parse(token_gen)?;
		}),
		"@"=>properties!(token_gen, first_token, OuterDef::Peice, {
			moves=>List::parse(token_gen)?;
			events=>List::parse(token_gen)?;
			checkable=>Data::parse(token_gen)?;
			value=>Data::parse(token_gen)?;
		}),
		v=>{return Err(UserMessage::new(format!("invalid type of definition `{}` the valid types are `#` or `@`",v),kind_token));}
	});
}}
impl Ranged for OuterDef{
	fn get_range(&self)->&Range {match self{
		OuterDef::Colour { range, .. } |
		OuterDef::Peice { range, .. } => range,
	}}
	fn set_range<T:FnOnce(&Range)->Range>(&mut self, setter: T) {match self{
		OuterDef::Colour { ref mut range, .. } |
		OuterDef::Peice { ref mut range, .. } => *range = setter(&range),
	}}
}

#[derive(Debug, PartialEq)]
struct Move{

}
impl Parse for Move{fn parse<'a>(_token_gen: &mut PeekNth<impl Iterator<Item=&'a Token>>)->Result<Self, UserMessage>{
	todo!()
}}
impl Ranged for Move{
	fn get_range(&self)->&Range {
		todo!()
	}
	fn set_range<T:FnOnce(&Range)->Range>(&mut self, _setter: T) {
		todo!()
	}
}

#[derive(Debug, PartialEq)]
struct Event{

}
impl Parse for Event{fn parse<'a>(_token_gen: &mut PeekNth<impl Iterator<Item=&'a Token>>)->Result<Self, UserMessage>{
	todo!()
}}
impl Ranged for Event{
	fn get_range(&self)->&Range {
		todo!()
	}
	fn set_range<T:FnOnce(&Range)->Range>(&mut self, _setter: T) {
		todo!()
	}
}
#[derive(Debug, PartialEq)]
enum Expresion{
	UnaryAddition(Data, Range),
	UnarySubtraction(Data, Range),
	UnaryPlusMinus(Data, Range),
	BinaryAddition(Data, Data, Range),
	BinarySubtraction(Data, Data, Range),
	Multipication(Data, Data, Range),
	Division(Data, Data, Range),
}
const OOO: [&str; 4]=["+","-","*","/"];
macro_rules! next_ooo {($op: ident) => {OOO[OOO.iter().position(|o|o==&$op).expect("parse_ooo should always provide valid data")+1]};}
impl Expresion{
	fn get_binary_expr(op: &str, lhs:Data, rhs:Data)->Expresion{
		let lhs_range = *lhs.get_range();
		let rhs_range = *rhs.get_range();
		return match op {
			"+"=>Expresion::BinaryAddition,
			"-"=>Expresion::BinarySubtraction,
			"*"=>Expresion::Multipication,
			"/"=>Expresion::Division,
			_=>panic!("invalid op"), // this case should be handled by parse_ooo
		}(lhs, rhs, Range{
			start: lhs_range.start,
			end: rhs_range.end,
		});
	}
	///parses the order of operations
	fn parse_ooo(operations: Vec<&Token>, mut data: Vec<Data>, op: &str)->Expresion{
		fn get_data(mut data_vec: Vec<Data>, ops: Vec<&Token>, op: &str)->Data
		{if data_vec.len() == 1{data_vec.pop().expect("just checked the length")}else{
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
	)->Result<Result<Self, UserMessage>, Data>{
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
	fn parse_unary<'a>(token_gen: &mut PeekNth<impl Iterator<Item=&'a Token>>)->Option<Result<Self, UserMessage>>{
		let mut unary_tokens = vec!();
		while let Some(token) = token_gen.next_if(|t|
			t.content == "-" ||
			t.content == "+" ||
			t.content == "+/-"
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
				"+/-"=>Expresion::UnaryPlusMinus,
				_=>panic!("this is handled at the begining of the function")
			}(base, range));
		}
		return rv.map(|v|Ok(v));
	}
}
impl Ranged for Expresion{
	fn get_range(&self)->&Range{match self{
		Expresion::UnaryAddition(_, r) |
		Expresion::UnarySubtraction(_, r) |
		Expresion::UnaryPlusMinus(_, r) |
		Expresion::BinaryAddition(_, _, r) |
		Expresion::BinarySubtraction(_, _, r) |
		Expresion::Multipication(_, _, r) |
		Expresion::Division(_, _, r) => r,
	}}
	fn set_range<T:FnOnce(&Range)->Range>(&mut self, setter: T) {match self{
		Expresion::UnaryAddition(_, ref mut r) |
		Expresion::UnarySubtraction(_, ref mut r) |
		Expresion::UnaryPlusMinus(_, ref mut r) |
		Expresion::BinaryAddition(_, _, ref mut r) |
		Expresion::BinarySubtraction(_, _, ref mut r) |
		Expresion::Multipication(_, _, ref mut r) |
		Expresion::Division(_, _, ref mut r) => *r = setter(&r),
	}}
}
#[derive(Debug, PartialEq)]
struct Variable{
	route: Vec<String>,
	range: Range,
}
impl Parse for Variable{fn parse<'a>(token_gen: &mut PeekNth<impl Iterator<Item=&'a Token>>)->Result<Self, UserMessage>{
	let Some(first_token) = token_gen.next() else{return Err("expected a variable reference however reached the end of the file".into())};
	let mut route = vec!();
	let mut curr_token = first_token;
	loop{
		let Some(first_char) = curr_token.content.chars().next()else{panic!("the lexer shoudnt emmit tokens with no length")};
		if !is_letter(&first_char){return Err(UserMessage::new(format!("expected a variable however a variable cannot start with `{}`", first_char),curr_token))}
		route.push(curr_token.content.clone());
		let Some(sep) = token_gen.peek()else{break;};
		if sep.content != "."{break;}
		token_gen.next().expect("peek failed");
		curr_token = match token_gen.next(){Some(v)=>v,None=>return Err("expected a variable path identifier after the `.` however reached the end of the file".into())};
	}
	Ok(Self{route, range: Range::from_first_last_token(first_token, curr_token)})
}}
impl Ranged for Variable{
	fn get_range(&self)->&Range{&self.range}
	fn set_range<T:FnOnce(&Range)->Range>(&mut self, setter: T){self.range = setter(&self.range)}
}

#[derive(Debug, PartialEq)]
enum Data{
	Expresion(Box<Expresion>, Range),
	Value(isize, Range),
	Variable(Variable, Range),
	Vector(Box<Vector>, Range),
}
impl Data{
	/// this can acctualy include an expr but only when it is contained within brackets
	fn parse_without_expr<'a>(token_gen: &mut PeekNth<impl Iterator<Item=&'a Token>>)->Result<Self, UserMessage>{
		if let Some(opening) = token_gen.next_if(|token|token.content == "("){
			let rv = Data::parse(token_gen)?;
			return if let Some(closing) = token_gen.next(){
				if closing.content == ")"{
					Ok(rv)
				}else{Err(UserMessage::new_with_note(format!("expected a closing brace however got `{}`", closing.content), Some(closing.into()), UserMessage::new("this is probably supposed to close this bracket", opening)))}
			}else{Err(UserMessage::new_with_note("expected a closing brace however reached the end of the file", None, UserMessage::new("this is probably supposed to close this bracket", opening)))}
		}

		let Some(first_token) = token_gen.peek() else{return Err("expected data however reached the end of the file".into())};
		let Some(first_char) = first_token.content.chars().next() else{panic!("lexer shouldnt emit empty tokens")};
		if is_letter(&first_char){
			let var = Variable::parse(token_gen)?;
			let range = *var.get_range();
			return Ok(Data::Variable(var,range));
		}else if is_number(&first_char){
			let first_token = token_gen.next().expect("peek failed");
			return Ok(Data::Value(first_token.content.parse().map_err(
				|_|UserMessage::new(format!("Failed to parse int {}",first_token.content), Range::from_token(first_token))
			)?, Range::from_first_last_token(first_token, first_token)))
		}else if first_char == '{'{
			let var = Vector::parse(token_gen)?;
			let range = *var.get_range();
			return Ok(Data::Vector(Box::new(var),range));
		}
		return Err(UserMessage::new(format!("Expected data however got `{}` try putting a number literal, mathmatical expresion or a variable reference",first_token.content),*first_token))
	}
	fn parse_unary<'a>(token_gen: &mut PeekNth<impl Iterator<Item=&'a Token>>)->Option<Result<Self, UserMessage>>{
		let Some(unary) = Expresion::parse_unary(token_gen) else{return None;};
		let unary = match unary{Ok(v)=>v,Err(e)=>return Some(Err(e))};
		let range = *unary.get_range();
		Some(Ok(Data::Expresion(unary.into(), range)))
	}
}
impl Parse for Data{fn parse<'a>(token_gen: &mut PeekNth<impl Iterator<Item=&'a Token>>)->Result<Self, UserMessage>{
	let first_data = if let Some(unary) = Self::parse_unary(token_gen){unary?}else{
		Self::parse_without_expr(token_gen)?
	};
	
	Ok(match Expresion::parse(first_data,token_gen){Ok(expr)=>{
		let expr = expr?;
		let range = *expr.get_range();
		Data::Expresion(expr.into(),range)
	}, Err(first_data)=>first_data})
}}
impl Ranged for Data{
	fn get_range(&self)->&Range{match self{
		Data::Expresion(_, r) |
		Data::Value(_, r) |
		Data::Vector(_, r) |
		Data::Variable(_, r) => r,
	}}
	fn set_range<T:FnOnce(&Range)->Range>(&mut self, setter: T) {match self{
		Data::Expresion(_, ref mut r) |
		Data::Value(_, ref mut r) |
		Data::Vector(_, ref mut r) |
		Data::Variable(_, ref mut r) => *r = setter(&r),
	}}
}

#[derive(Debug, PartialEq)]
struct List<T:Parse+Ranged>{
	data: Vec<T>,
	range: Range,
}
impl<T:Parse+Ranged> List<T>{

}
impl<T:Parse+Ranged> Parse for List<T>{fn parse<'a>(token_gen: &mut PeekNth<impl Iterator<Item=&'a Token>>)->Result<Self, UserMessage>{
	let Some(first_token) = token_gen.next() else{return Err("expected a list however the file ended".into())};
	if first_token.content != "["{return Err(UserMessage::new(format!("expected a list however got `{}` to start a list use the `[` token", first_token.content), Range::from_token(first_token)))}
	let mut data = Vec::new();
	let last_token = loop{
		let Some(token) = token_gen.peek() else{return Err(UserMessage::new("mismatched `[` expected a closing tag for the opening square bracket.", first_token.pos))};
		if token.content == "]"{break token_gen.next().expect("peek failed");}
		data.push(T::parse(token_gen)?);
		let Some(token) = token_gen.next() else{return Err(UserMessage::new("mismatched `[` expected a closing tag for the opening square bracket.", first_token.pos))};
		if token.content == "]"{break token;}
		if token.content != ","{return Err(UserMessage::new(format!("expected a comma (`,`) to continue the list or closing bracket (`]`) to end it, however got `{}`", token.content),token))}
	};
	return Ok(Self{data, range:Range::from_first_last_token(first_token, last_token)})
}}
impl<T:Parse+Ranged> Ranged for List<T>{
	fn get_range(&self)->&Range {&self.range}
	fn set_range<Fn:FnOnce(&Range)->Range>(&mut self, setter: Fn) {self.range = setter(&self.range)}
}

#[derive(Debug, PartialEq)]
struct Vector{
	x: Data,
	y: Data,
	range: Range,
}
impl Parse for Vector{
	fn parse<'a>(token_gen: &mut PeekNth<impl Iterator<Item=&'a Token>>)->Result<Self, UserMessage> where Self: Sized {
		let Some(first_token) = token_gen.next() else{return Err("expected a vector however the file ended".into())};
		if first_token.content != "{"{return Err(UserMessage::new(format!("expected a vector however got `{}` to start a vector use the `{{` token", first_token.content), Range::from_token(first_token)))}
		let x = Data::parse(token_gen)?;
		let Some(token) = token_gen.next() else{return Err(UserMessage::new("mismatched `{{` expected a closing tag for the opening curly bracket.", first_token.pos))};
		if token.content != ","{return Err(UserMessage::new(format!("expected a comma (`,`) to to seperate the x and y components however got `{}`", token.content),token))}
		let y = Data::parse(token_gen)?;
		let Some(token) = token_gen.next() else{return Err(UserMessage::new("mismatched `{{` expected a closing tag for the opening curly bracket.", first_token.pos))};
		if token.content != "}"{return Err(UserMessage::new(format!("expected a closing curly bracket as a vector can only be 2d however got `{}`", token.content),token))}
		return Ok(Self{x, y, range: Range::from_first_last_token(first_token, token)})
	}
}
impl Ranged for Vector{
	fn get_range(&self)->&Range {&self.range}
	fn set_range<T:FnOnce(&Range)->Range>(&mut self, setter: T) {self.range = setter(&self.range);}
}

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
	pub fn lexer(&mut self) -> Result<(),UserMessage>{
		let Some(data) = &self.data else{return Err("Lexer Error: no data, Data must be added before lexing can start".into());};
		
		let mut tokens = Vec::new();
		let pos = &mut Pos{
			line: 1,
			chr: 0,
		};
		let mut curr_expr:Option<(String, Pos)> = None;
		let mut in_inline_comment = false;
		let mut in_comment = false;

		for chr in data.chars(){
			pos.chr+=1;
			if !is_all_valid_char(&chr){return Err(UserMessage::new(format!("Lexer Error: invalid char '{}'", chr),pos));}
			if chr == '\n'{
				pos.line+=1;
				pos.chr=0;
				if in_comment{
					in_comment = false;
					curr_expr = None;
				}
			}
			if in_inline_comment && {if let Some(ref curr_expr) = curr_expr{
				//if the last two chars == "*/" note the check is flipped as that is the order the itterator works
				let mut chars = curr_expr.0.chars();
				chars.next_back() == Some('/') && chars.next_back() == Some('*')
			}else{false}}{
				in_inline_comment=false;
				curr_expr=None;
			}
			if in_comment || in_inline_comment{
				if let Some(ref mut curr_expr) = curr_expr{
					curr_expr.0+=&chr.to_string();
				}else{curr_expr = Some((chr.to_string(),pos.clone()))}
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
				curr_expr=None;
			}else if
				chr == '/' &&
				tokens.last() == Some(&Token{
					content: "/".to_string(),
					pos: Pos::new(pos.line, pos.chr-1)
				})
			{
				in_comment = true;
				tokens.pop();
				curr_expr = None;
			}else if is_expr_char(&chr){
				if let Some(curr_expr) = curr_expr.take(){// take sets to None
					tokens.push(Token{content: curr_expr.0, pos: curr_expr.1,});
				}
				let len = tokens.len();
				if
					chr == '-'
					&& len >= 2
					&& let Ok([plus, slash]) = tokens.get_many_mut([len-2, len-1])
					&& plus == (&mut Token{
						content: "+".to_string(),
						pos: Pos::new(pos.line, pos.chr-2),
					})
					&& slash == (&Token{
						content: "/".to_string(),
						pos: Pos::new(pos.line, pos.chr-1),
					})
				{
					plus.content += "/-";
					tokens.pop();
				}else if
					is_double_char(&chr)
					&& let Some(last_t) = tokens.last_mut()
					&& last_t == (&mut Token{
						content: chr.to_string(),
						pos: Pos::new(pos.line, pos.chr-1)
					})
				{last_t.content += &chr.to_string()} else if
					chr=='='
					&& let Some(last) = tokens.last_mut()
					&& last.content.chars().count() == 1
					&& is_equal_type(&last.content.chars().next().unwrap())
					&& last.pos == (Pos::new(pos.line, pos.chr-1))
				{last.content += &chr.to_string();} else{
					tokens.push(Token{content: chr.to_string(), pos: pos.clone(),});
					curr_expr = None;
				}
			}else if is_white_space_char(&chr){
				if let Some(curr_expr) = curr_expr.take(){// take sets to None
					tokens.push(Token{content: curr_expr.0, pos: curr_expr.1,});
				}
			}else{
				if let Some(curr_expr) = curr_expr.take_if(|curr_expr|
					is_number(&if let Some(chr) = curr_expr.0.chars().next(){chr}else{panic!("curr_expr.1 should never be empty")})
					&& is_letter(&chr)
				){tokens.push(Token{
					content: curr_expr.0,
					pos: curr_expr.1,
				});};
				if let Some(ref mut curr_expr) = curr_expr{
					curr_expr.0+=&chr.to_string();
				}else{curr_expr = Some((chr.to_string(),pos.clone()))}
			}
		}
		if let Some(curr_expr) = curr_expr.take() && !in_comment && !in_inline_comment{
			tokens.push(Token{content: curr_expr.0, pos: curr_expr.1,});
		}
		self.tokens = Some(tokens);
		return Ok(());
	}
	pub fn parser(&mut self) -> Result<(),String>{
		let Some(ref tokens) = self.tokens else {
			return Err("Tokeniser Error: no tokens, Tokens must be added or generated before tokenising can start".to_string());
		};

		let mut token_gen = peek_nth(tokens.iter());
		let root = RootExpr::parse(&mut token_gen);
		match root{
			Ok(root) => todo!("{:#?}",root),
			Err(err) => todo!("{:#?}\n{}",token_gen, err.error_string(&self, None)),
		}
	}
	pub fn get_debug(&self)->String{format!("{:#?}",self)}
	fn pretty_print_pos(&self, pos: &Pos)->String{format!("{}:{}:{}", self.filename, pos.line, pos.chr)}
}