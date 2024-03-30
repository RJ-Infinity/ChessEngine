const CPD_DATA: &str = include_str!("../../standard.cpd");

use crate::{Data, Expresion, Interpreter, List, OuterDef, Parse, Pos, Range, Token, Variable};
use itertools::peek_nth;

#[test]
fn test_std(){
	// this doesnt test if the stdlib is parsed correctly just that it dosent crash whilst doing it
	let mut int = Interpreter::new("<test>".to_string());
	int.data = Some(CPD_DATA.to_string());
	if let Err(e) = int.lexer(){panic!("{}",e.error_string(&int, None))};
	// int.parser().unwrap();
}

macro_rules! lex_str{($lit:literal) =>{{
	let mut int = Interpreter::new("<test>".to_string());
	int.data = Some($lit.to_string());
	int.lexer().unwrap();
	int.tokens
}};}
#[test]
fn lexer(){
	assert_eq!(lex_str!("test"),Some(vec![Token{
		content: "test".to_string(),
		pos: Pos::new(1, 1)
	}]));
	assert_eq!(lex_str!("test.asdf"),Some(vec![
		Token{content: "test".to_string(), pos: Pos::new(1, 1)},
		Token{content: ".".to_string(), pos: Pos::new(1, 5)},
		Token{content: "asdf".to_string(), pos: Pos::new(1, 6)},
	]));
	assert_eq!(lex_str!("the // comments and white space should be ignored"),Some(vec![Token{
		content: "the".to_string(),
		pos: Pos::new(1, 1)
	},]));
	//expr types, comments
	assert_eq!(lex_str!("the // comments and white space should be ignored\n(but + the.things == on* (the /next) line/ should ),still,parse"),Some(vec![
		Token{content: "the".to_string(), pos: Pos{line: 1, chr: 1}},
		Token{content: "(".to_string(), pos: Pos{line: 2, chr: 1}},
		Token{content: "but".to_string(), pos: Pos{line: 2, chr: 2}},
		Token{content: "+".to_string(), pos: Pos{line: 2, chr: 6}},
		Token{content: "the".to_string(), pos: Pos{line: 2, chr: 8}},
		Token{content: ".".to_string(), pos: Pos{line: 2, chr: 11}},
		Token{content: "things".to_string(), pos: Pos{line: 2, chr: 12}},
		Token{content: "==".to_string(), pos: Pos{line: 2, chr: 19}},
		Token{content: "on".to_string(), pos: Pos{line: 2, chr: 22}},
		Token{content: "*".to_string(), pos: Pos{line: 2, chr: 24}},
		Token{content: "(".to_string(), pos: Pos{line: 2, chr: 26}},
		Token{content: "the".to_string(), pos: Pos{line: 2, chr: 27}},
		Token{content: "/".to_string(), pos: Pos{line: 2, chr: 31}},
		Token{content: "next".to_string(), pos: Pos{line: 2, chr: 32}},
		Token{content: ")".to_string(), pos: Pos{line: 2, chr: 36}},
		Token{content: "line".to_string(), pos: Pos{line: 2, chr: 38}},
		Token{content: "/".to_string(), pos: Pos{line: 2, chr: 42}},
		Token{content: "should".to_string(), pos: Pos{line: 2, chr: 44}},
		Token{content: ")".to_string(), pos: Pos{line: 2, chr: 51}},
		Token{content: ",".to_string(), pos: Pos{line: 2, chr: 52}},
		Token{content: "still".to_string(), pos: Pos{line: 2, chr: 53}},
		Token{content: ",".to_string(), pos: Pos{line: 2, chr: 58}},
		Token{content: "parse".to_string(), pos: Pos{line: 2, chr: 59}},
	]));
	assert_eq!(lex_str!("block /* comments should also work */ both \n on one /*line and when split\nover */ multiple /* also it should /* ignore another opening */ block comment in the comment"), Some(vec![
		Token{content: "block".to_string(), pos: Pos{line: 1, chr: 1}},
		Token{content: "both".to_string(), pos: Pos{line: 1, chr: 39}},
		Token{content: "on".to_string(), pos: Pos{line: 2, chr: 2}},
		Token{content: "one".to_string(), pos: Pos{line: 2, chr: 5}},
		Token{content: "multiple".to_string(), pos: Pos{line: 3, chr: 9}},
		Token{content: "block".to_string(), pos: Pos{line: 3, chr: 65}},
		Token{content: "comment".to_string(), pos: Pos{line: 3, chr: 71}},
		Token{content: "in".to_string(), pos: Pos{line: 3, chr: 79}},
		Token{content: "the".to_string(), pos: Pos{line: 3, chr: 82}},
		Token{content: "comment".to_string(), pos: Pos{line: 3, chr: 86}},
	]));
	assert_eq!(lex_str!("block // comment opening /* should be ignored\nin line /* comments and // visa \n versa */ "), Some(vec![
		Token{content: "block".to_string(), pos: Pos{line: 1, chr: 1}},
		Token{content: "in".to_string(), pos: Pos{line: 2, chr: 1}},
		Token{content: "line".to_string(), pos: Pos{line: 2, chr: 4}},
	]));
	//double types (any should work for the robust testing)
	assert_eq!(lex_str!("= = == == = && || << >>"), Some(vec![
		Token{content: "=".to_string(), pos: Pos{line: 1, chr: 1}},
		Token{content: "=".to_string(), pos: Pos{line: 1, chr: 3}},
		Token{content: "==".to_string(), pos: Pos{line: 1, chr: 5}},
		Token{content: "==".to_string(), pos: Pos{line: 1, chr: 8}},
		Token{content: "=".to_string(), pos: Pos{line: 1, chr: 11}},
		Token{content: "&&".to_string(), pos: Pos{line: 1, chr: 13}},
		Token{content: "||".to_string(), pos: Pos{line: 1, chr: 16}},
		Token{content: "<<".to_string(), pos: Pos{line: 1, chr: 19}},
		Token{content: ">>".to_string(), pos: Pos{line: 1, chr: 22}},
	]));
	// presedence shouldnt matter as this should always be invalid
	assert_eq!(lex_str!("==="),Some(vec![
		Token{content: "==".to_string(), pos: Pos::new(1, 1)},
		Token{content: "=".to_string(), pos: Pos::new(1, 3)},
	]));
	//equal types
	assert_eq!(lex_str!("!= <= >= !=!="),Some(vec![
		Token{content: "!=".to_string(), pos: Pos::new(1, 1)},
		Token{content: "<=".to_string(), pos: Pos::new(1, 4)},
		Token{content: ">=".to_string(), pos: Pos::new(1, 7)},
		Token{content: "!=".to_string(), pos: Pos{line: 1, chr: 10}},
		Token{content: "!=".to_string(), pos: Pos{line: 1, chr: 12}},
	]));
	// that numbers split but tokens can contain numbers if the dont start with them
	assert_eq!(lex_str!("0test test0 test0test 12345"),Some(vec![
		Token{content: "0".to_string(), pos: Pos::new(1, 1)},
		Token{content: "test".to_string(), pos: Pos::new(1, 2)},
		Token{content: "test0".to_string(), pos: Pos::new(1, 7)},
		Token{content: "test0test".to_string(), pos: Pos{line: 1, chr: 13}},
		Token{content: "12345".to_string(), pos: Pos{line: 1, chr: 23}},
	]));
	assert_eq!(lex_str!("="),Some(vec![Token{content: "=".to_string(), pos: Pos::new(1, 1)},]));
	// +/-
	assert_eq!(lex_str!("+/- + / - +/ - + /-"),Some(vec![
		Token{content: "+/-".to_string(), pos: Pos{line: 1, chr: 1}},
		Token{content: "+".to_string(), pos: Pos{line: 1, chr: 5}},
		Token{content: "/".to_string(), pos: Pos{line: 1, chr: 7}},
		Token{content: "-".to_string(), pos: Pos{line: 1, chr: 9}},
		Token{content: "+".to_string(), pos: Pos{line: 1, chr: 11}},
		Token{content: "/".to_string(), pos: Pos{line: 1, chr: 12}},
		Token{content: "-".to_string(), pos: Pos{line: 1, chr: 14}},
		Token{content: "+".to_string(), pos: Pos{line: 1, chr: 16}},
		Token{content: "/".to_string(), pos: Pos{line: 1, chr: 18}},
		Token{content: "-".to_string(), pos: Pos{line: 1, chr: 19}}
	]));
}

macro_rules! get_token_gen{($data: literal) =>{{
	let mut int = Interpreter::new("<test>".to_string());
	int.data = Some($data.to_string());
	int.lexer().unwrap();
	&mut peek_nth(int.tokens.unwrap().iter())
}};}
#[test]
fn variable_parser(){
	assert_eq!(Variable::parse(get_token_gen!("test")),Ok(Variable{
		route: vec!["test".to_string()],
		range: Range{start: Pos::new(1,1), end: Pos::new(1,4)}
	}));
	assert_eq!(Variable::parse(get_token_gen!("test.second")),Ok(Variable{
		route: vec!["test".to_string(),"second".to_string()],
		range: Range{start: Pos::new(1,1), end: Pos::new(1,11)}
	}));
	assert_eq!(Variable::parse(get_token_gen!("test.second.third")),Ok(Variable{
		route: vec!["test".to_string(),"second".to_string(),"third".to_string(),],
		range: Range{start: Pos::new(1,1), end: Pos::new(1,17)}
	}));
}
#[test]
fn expresion_parser(){
	assert_eq!(
		Expresion::parse_unary(get_token_gen!("-1")),
		Some(Ok(Expresion::UnarySubtraction(
			Data::Value(1, Range{start: Pos{line: 1, chr: 2}, end: Pos{line: 1, chr: 2}}),
			Range{start: Pos{line: 1, chr: 1}, end: Pos{line: 1, chr: 2}}
		)))
	);
	assert_eq!(
		Expresion::parse(
			Data::Value(0, Range{start: Pos{line: 1, chr: 1}, end: Pos{line: 1, chr: 2}}),
			get_token_gen!(" /9")
		),
		Ok(Ok(Expresion::Division(
			Data::Value(0, Range{start: Pos{line: 1, chr: 1}, end: Pos{line: 1, chr: 2}}),
			Data::Value(9, Range{start: Pos{line: 1, chr: 3}, end: Pos{line: 1, chr: 3}}),
			Range{start: Pos{line: 1, chr: 1}, end: Pos{line: 1, chr: 3}}
		)))
	);
	assert!(Expresion::parse(
		Data::Value(0, Range{start: Pos{line: 1, chr: 1}, end: Pos{line: 1, chr: 2}}),
		get_token_gen!(" asdf")
	).is_err());
}
#[test]
fn outer_def_parser(){
	assert_eq!(
		OuterDef::parse(get_token_gen!("<#white direction=forwards>")),
		Ok(OuterDef::Colour{
			name: "white".to_string(),
			direction: Data::Variable(Variable{
				route: vec!["forwards".to_string()],
				range: Range{start: Pos{line: 1, chr: 19}, end: Pos{line: 1, chr: 26}}
			}, Range{start: Pos{line: 1, chr: 19}, end: Pos{line: 1, chr: 26}}),
			range: Range{start: Pos{line: 1, chr: 1}, end: Pos{line: 1, chr: 27}}}
		)
	);
}
#[test]
fn data_parser(){
	assert_eq!(
		Data::parse(get_token_gen!("-1")),
		Ok(Data::Expresion(Box::new(Expresion::UnarySubtraction(
			Data::Value(1, Range{start: Pos{line: 1, chr: 2},end: Pos{line: 1, chr: 2}}),
			Range{start: Pos{line: 1, chr: 1}, end: Pos{line: 1, chr: 2}}
		)), Range{start: Pos{line: 1, chr: 1}, end: Pos{line: 1, chr: 2}}))
	);
	assert_eq!(
		Data::parse(get_token_gen!("1+2/3-4*5")),
		Ok(Data::Expresion(Box::new(Expresion::BinaryAddition(
			Data::Value(1, Range{start: Pos{line: 1, chr: 1}, end: Pos{line: 1, chr: 1}}),
			Data::Expresion(Box::new(Expresion::BinarySubtraction(
				Data::Expresion(Box::new(Expresion::Division(
					Data::Value(2, Range{start: Pos{line: 1, chr: 3}, end: Pos{line: 1, chr: 3}}),
					Data::Value(3, Range{start: Pos{line: 1, chr: 5}, end: Pos{line: 1, chr: 5}}),
					Range{start: Pos{line: 1, chr: 3}, end: Pos{line: 1, chr: 5}}
				)), Range{start: Pos{line: 1, chr: 3}, end: Pos{line: 1, chr: 5}}),
				Data::Expresion(Box::new(Expresion::Multipication(
					Data::Value(4, Range{start: Pos{line: 1, chr: 7}, end: Pos{line: 1, chr: 7}}),
					Data::Value(5, Range{start: Pos{line: 1, chr: 9}, end: Pos{line: 1, chr: 9}}),
					Range{start: Pos{line: 1, chr: 7}, end: Pos{line: 1, chr: 9}}
				)), Range{start: Pos{line: 1, chr: 7}, end: Pos{line: 1, chr: 9}}),
				Range{start: Pos{line: 1, chr: 3}, end: Pos{line: 1, chr: 9}}
			)), Range{start: Pos{line: 1, chr: 3}, end: Pos{line: 1, chr: 9}}),
			Range{start: Pos{line: 1, chr: 1}, end: Pos{line: 1, chr: 9}}
		)), Range{start: Pos{line: 1, chr: 1}, end: Pos{line: 1, chr: 9}}))
	);
	assert_eq!(
		Data::parse(get_token_gen!("(1+2)/(4)")),
		Ok(Data::Expresion(Box::new(Expresion::Division(
			Data::Expresion(Box::new(Expresion::BinaryAddition(
				Data::Value(1, Range{start: Pos{line: 1, chr: 2}, end: Pos{line: 1, chr: 2}}),
				Data::Value(2, Range{start: Pos{line: 1, chr: 4}, end: Pos{line: 1, chr: 4}}),
				Range{start: Pos{line: 1, chr: 2}, end: Pos{line: 1, chr: 4}}
			)), Range{start: Pos{line: 1, chr: 2}, end: Pos{line: 1, chr: 4}}),
			Data::Value(4, Range{start: Pos{line: 1, chr: 8}, end: Pos{line: 1, chr: 8}}),
			Range{start: Pos{line: 1, chr: 2}, end: Pos{line: 1, chr: 8}}
		)), Range{start: Pos{line: 1, chr: 2}, end: Pos{line: 1, chr: 8}}))
	)
}
#[test]
fn list_parser(){
	assert_eq!(
		List::<Data>::parse(get_token_gen!("[(1+2)/(4), 1,2,3, testing.theis, thisn]")),
		Ok(List{
			data: vec![
				Data::Expresion(Box::new(Expresion::Division(
					Data::Expresion(Box::new(Expresion::BinaryAddition(
						Data::Value(1, Range{start: Pos{line: 1, chr: 3}, end: Pos{line: 1, chr: 3}}),
						Data::Value(2, Range{start: Pos{line: 1, chr: 5}, end: Pos{line: 1, chr: 5}}),
						Range{start: Pos{line: 1, chr: 3}, end: Pos{line: 1, chr: 5}}
					)), Range{start: Pos{line: 1, chr: 3}, end: Pos{line: 1, chr: 5}}),
					Data::Value(4, Range{start: Pos{line: 1, chr: 9}, end: Pos{line: 1, chr: 9}}),
					Range{start: Pos{line: 1, chr: 3}, end: Pos{line: 1, chr: 9}}
				)), Range{start: Pos{line: 1, chr: 3}, end: Pos{line: 1, chr: 9}}),
				Data::Value(1, Range{start: Pos{line: 1, chr: 13}, end: Pos{line: 1, chr: 13}}),
				Data::Value(2, Range{start: Pos{line: 1, chr: 15}, end: Pos{line: 1, chr: 15}}),
				Data::Value(3, Range{start: Pos{line: 1, chr: 17}, end: Pos{line: 1, chr: 17}}),
				Data::Variable(Variable{
					route: vec!["testing".to_string(), "theis".to_string()],
					range: Range{start: Pos{line: 1, chr: 20}, end: Pos{line: 1, chr: 32}}
				}, Range{start: Pos{line: 1, chr: 20}, end: Pos{line: 1, chr: 32}}),
				Data::Variable(Variable{
					route: vec!["thisn".to_string()],
					range: Range{start: Pos{line: 1, chr: 35}, end: Pos{line: 1, chr: 39}}
				}, Range{start: Pos{line: 1, chr: 35}, end: Pos{line: 1, chr: 39}})
			],range: Range{start: Pos{line: 1, chr: 1}, end: Pos{line: 1, chr: 40}}
		})
	);
	assert_eq!(
		List::<Data>::parse(get_token_gen!("[]")),
		Ok(List{
			data:Vec::new(),
			range: Range{start: Pos{line: 1, chr: 1}, end: Pos{line: 1, chr: 2}}
		})
	);
	assert_eq!(
		List::<Data>::parse(get_token_gen!("[123]")),
		Ok(List{
			data:vec![Data::Value(123, Range{start: Pos{line: 1, chr: 2}, end: Pos{line: 1, chr: 4}})],
			range: Range{start: Pos{line: 1, chr: 1}, end: Pos{line: 1, chr: 5}}
		})
	);
}
