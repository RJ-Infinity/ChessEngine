const CPD_DATA: &str = r#"// chess piece definition file
<# name=white direction=(board.width)>
<# name=black direction=(-board.width)>
<@knight
	moves=[
		pos-board.width-2,
		pos-board.width*2-1,
		pos-board.width*2+1,
		pos-board.width+2,
		pos+board.width-2,
		pos+board.width*2-1,
		pos+board.width*2+1,
		pos+board.width-2
	]
	value=3
>
<@king
	moves=[
		pos+1,
		pos-1,
		pos-board.width,
		pos-board.width+1,
		pos-board.width-1,
		pos+board.width,
		pos+board.width+1,
		pos+board.width-1,
		pos+2:{
			board.getPiece(£move)==null &&
			!board.isAttacked(£move-1) &&
			!inCheck &&
			moveCount==0 &&
			board.getPiece(£move+1).moveCount==0 &&
			board.getPiece(£move-1) == null
		}${
			board.getPiece(£move+1).moveTo(£move-1)
		},
		pos-2:{
			board.getPiece(£move)==null &&
			!board.isAttacked(£move+1) &&
			!inCheck &&
			moveCount==0 &&
			board.getPiece(£move-2).moveCount==0 &&
			board.getPiece(£move+1) == null &&
			board.getPiece(£move-1) == null
		}${
			board.getPiece(£move-2).moveTo(£move+1)
		}
	]
	checkable=true
	events=($checkmate{colour.loose()})
	value=0
>
<@pawn
	moves=[
		pos+colour.direction:{board.getPiece(£move)==null},
		pos+colour.direction*2:{
			moveCount==0 &&
			board.getPiece(pos+colour.direction)==null &&
			board.getPiece(£move)==null
		},
		pos+colour.direction+1:{board.getPiece(£move)!=null},
		pos+colour.direction-1:{board.getPiece(£move)!=null},
		pos+colour.direction+1:{
			// there is no need for a check for an empty square as this must be the case
			// for the other pawn to move 
			board.getPiece(pos+1)=="pawn" &&
			board.getPiece(pos+1).colour!=colour &&
			board.getPiece(pos+1).lastmove==board.getPiece(pos+1).colour.lastmove &&
			board.getPiece(pos+1).lastmove.distance==board.getPiece(pos+1).colour.direction*2
		}${colour.takePiece(pos+1)},
		pos+colour.direction-1:{
			board.getPiece(pos-1)=="pawn" &&
			board.getPiece(pos+1).colour!=colour &&
			board.getPiece(pos-1).lastmove==board.getPiece(pos-1).colour.lastmove &&
			board.getPiece(pos-1).lastmove.distance==board.getPiece(pos-1).colour.direction*2
		}${colour.takePiece(pos-1)},
	]
	//note division is integer only so these comparisons work
	events=($move:{pos/board.width==0||pos/board.width==7}{
		board.removePiece(pos)
		board.setPiece(pos,colour.openSelectPiece(["queen","knight","rook","bishop"]),colour)
	})
	value=1
>
<@rook
	moves=(
		(
			(i)(pos)Σ(pos-(pos%board.width)+1)//between pos and the leftmost but one square on that row
			:{board.getPiece(£i)==null || board.getPiece(£i)==self}
			{£i-1}:{
				£i>=pos-(pos%board.width) &&
				board.getPiece(£i-1).colour!=colour
			}
		)+
		(
			(i)(pos)Σ(pos-(pos%board.width)+board.width-2)
			:{board.getPiece(£i)==null || board.getPiece(£i)==self}
			{£i+1}:{
				£i<<pos-(pos-(pos%board.width)+board.width) &&
				board.getPiece(£i+1).colour!=colour
			}
		)+
		(
			(i→(pos+£i*board.width))(0)Σ((board.length-board.width+(pos%board.width)-pos)/board.width)
			:{board.getPiece(£i)==null || board.getPiece(£i)==self}
			{£i+board.width}:{board.getPiece(£i+board.width).colour!=colour}
			//no bounds needed as it will just go out of bounds not overflow
		)+
		(
			(i→(pos-£i*board.width))(0)Σ((pos-(pos%board.width))/board.width)
			:{board.getPiece(£i)==null || board.getPiece(£i)==self}
			{£i-board.width}:{board.getPiece(£i-board.width).colour!=colour}
		)
	)
	value=5
>
<@bishop
	moves=(
		(//top right
			(i→(pos+£i*(board.width+1)))(0)Σ(board.width-(pos%board.width)-2)
			:{board.getPiece(£i)==null || board.getPiece(£i)==self}
			{£i+board.width+1}:{board.getPiece(£i+board.width+1).colour!=colour}
		)+
		(//top left
			(i→(pos+£i*(board.width-1)))(0)Σ((pos%board.width)-1)
			:{board.getPiece(£i)==null || board.getPiece(£i)==self}
			{£i+board.width-1}:{board.getPiece(£i+board.width-1).colour!=colour}
		)+
		(//bottom right
			(i→(pos+£i*(board.width+1)))(0)Σ(board.width-(pos%board.width)-2)
			:{board.getPiece(£i)==null || board.getPiece(£i)==self}
			{£i-board.width+1}:{board.getPiece(£i-board.width+1).colour!=colour}
		)+
		(//bottom left
			(i→(pos+£i*(board.width-1)))(0)Σ((pos%board.width)-1)
			:{board.getPiece(£i)==null || board.getPiece(£i)==self}
			{£i-board.width-1}:{board.getPiece(£i-board.width-1).colour!=colour}
		)
	)
	value=3
>
<@queen
	moves=(
		(//rook moves
			(i)(pos)Σ(pos-(pos%board.width)+1)//between pos and the leftmost but one square on that row
			:{board.getPiece(£i)==null || board.getPiece(£i)==self}
			{£i-1}:{
				£i>=pos-(pos%board.width) &&
				board.getPiece(£i-1).colour!=colour
			}
		)+
		(
			(i)(pos)Σ(pos-(pos%board.width)+board.width-2)
			:{board.getPiece(£i)==null || board.getPiece(£i)==self}
			{£i+1}:{
				£i<<pos-(pos-(pos%board.width)+board.width) &&
				board.getPiece(£i+1).colour!=colour
			}
		)+
		(
			(i→(pos+£i*board.width))(0)Σ((board.length-board.width+(pos%board.width)-pos)/board.width)
			:{board.getPiece(£i)==null || board.getPiece(£i)==self}
			{£i+board.width}:{board.getPiece(£i+board.width).colour!=colour}
		)+
		(
			(i→(pos-£i*board.width))(0)Σ((pos-(pos%board.width))/board.width)
			:{board.getPiece(£i)==null || board.getPiece(£i)==self}
			{£i-board.width}:{board.getPiece(£i-board.width).colour!=colour}
		)+
		(//bishop moves
			(i→(pos+£i*(board.width+1)))(0)Σ(board.width-(pos%board.width)-2)
			:{board.getPiece(£i)==null || board.getPiece(£i)==self}
			{£i+board.width+1}:{board.getPiece(£i+board.width+1).colour!=colour}
		)+
		(
			(i→(pos+£i*(board.width-1)))(0)Σ((pos%board.width)-1)
			:{board.getPiece(£i)==null || board.getPiece(£i)==self}
			{£i+board.width-1}:{board.getPiece(£i+board.width-1).colour!=colour}
		)+
		(
			(i→(pos+£i*(board.width+1)))(0)Σ(board.width-(pos%board.width)-2)
			:{board.getPiece(£i)==null || board.getPiece(£i)==self}
			{£i-board.width+1}:{board.getPiece(£i-board.width+1).colour!=colour}
		)+
		(
			(i→(pos+£i*(board.width-1)))(0)Σ((pos%board.width)-1)
			:{board.getPiece(£i)==null || board.getPiece(£i)==self}
			{£i-board.width-1}:{board.getPiece(£i-board.width-1).colour!=colour}
		)
	)
	value=9
>"#;

use crate::{Interpreter, Pos, Token};

#[test]
fn test_std(){
	// this doesnt test if the stdlib is parsed correctly just that it dosent crash whilst doing it
	let mut int = Interpreter::new("<test>".to_string());
	int.data = Some(CPD_DATA.to_string());
	int.lexer().unwrap();
}
macro_rules! lex_str {($lit:literal) => {{
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
		Token { content: "the".to_string(), pos: Pos { line: 1, chr: 1 } },
		Token { content: "(".to_string(), pos: Pos { line: 2, chr: 1 } },
		Token { content: "but".to_string(), pos: Pos { line: 2, chr: 2 } },
		Token { content: "+".to_string(), pos: Pos { line: 2, chr: 6 } },
		Token { content: "the".to_string(), pos: Pos { line: 2, chr: 8 } },
		Token { content: ".".to_string(), pos: Pos { line: 2, chr: 11 } },
		Token { content: "things".to_string(), pos: Pos { line: 2, chr: 12 } },
		Token { content: "==".to_string(), pos: Pos { line: 2, chr: 19 } },
		Token { content: "on".to_string(), pos: Pos { line: 2, chr: 22 } },
		Token { content: "*".to_string(), pos: Pos { line: 2, chr: 24 } },
		Token { content: "(".to_string(), pos: Pos { line: 2, chr: 26 } },
		Token { content: "the".to_string(), pos: Pos { line: 2, chr: 27 } },
		Token { content: "/".to_string(), pos: Pos { line: 2, chr: 31 } },
		Token { content: "next".to_string(), pos: Pos { line: 2, chr: 32 } },
		Token { content: ")".to_string(), pos: Pos { line: 2, chr: 36 } },
		Token { content: "line".to_string(), pos: Pos { line: 2, chr: 38 } },
		Token { content: "/".to_string(), pos: Pos { line: 2, chr: 42 } },
		Token { content: "should".to_string(), pos: Pos { line: 2, chr: 44 } },
		Token { content: ")".to_string(), pos: Pos { line: 2, chr: 51 } },
		Token { content: ",".to_string(), pos: Pos { line: 2, chr: 52 } },
		Token { content: "still".to_string(), pos: Pos { line: 2, chr: 53 } },
		Token { content: ",".to_string(), pos: Pos { line: 2, chr: 58 } },
		Token { content: "parse".to_string(), pos: Pos { line: 2, chr: 59 } },
	]));
	assert_eq!(lex_str!("block /* comments should also work */ both \n on one /*line and when split\nover */ multiple"), Some(vec![
		Token { content: "block".to_string(), pos: Pos { line: 1, chr: 1 } },
		Token { content: "both".to_string(), pos: Pos { line: 1, chr: 39 } },
		Token { content: "on".to_string(), pos: Pos { line: 2, chr: 2 } },
		Token { content: "one".to_string(), pos: Pos { line: 2, chr: 5 } },
		Token { content: "multiple".to_string(), pos: Pos { line: 3, chr: 9 } },
	]));
	//double types (any should work for the robust testing)
	assert_eq!(lex_str!("= = == == = && || << >>"), Some(vec![
		Token { content: "=".to_string(), pos: Pos { line: 1, chr: 1 } },
		Token { content: "=".to_string(), pos: Pos { line: 1, chr: 3 } },
		Token { content: "==".to_string(), pos: Pos { line: 1, chr: 5 } },
		Token { content: "==".to_string(), pos: Pos { line: 1, chr: 8 } },
		Token { content: "=".to_string(), pos: Pos { line: 1, chr: 11 } },
		Token { content: "&&".to_string(), pos: Pos { line: 1, chr: 13 } },
		Token { content: "||".to_string(), pos: Pos { line: 1, chr: 16 } },
		Token { content: "<<".to_string(), pos: Pos { line: 1, chr: 19 } },
		Token { content: ">>".to_string(), pos: Pos { line: 1, chr: 22 } },
	]));
	// presedence shouldnt matter as this should always be invalid
	assert_eq!(lex_str!("==="),Some(vec![
		Token { content: "==".to_string(), pos: Pos::new(1, 1) },
		Token { content: "=".to_string(), pos: Pos::new(1, 3) },
	]));
	//equal types
	assert_eq!(lex_str!("!= <= >= !=!="),Some(vec![
		Token { content: "!=".to_string(), pos: Pos::new(1, 1) },
		Token { content: "<=".to_string(), pos: Pos::new(1, 4) },
		Token { content: ">=".to_string(), pos: Pos::new(1, 7) },
		Token { content: "!=".to_string(), pos: Pos { line: 1, chr: 10 } },
		Token { content: "!=".to_string(), pos: Pos { line: 1, chr: 12 } },
	]));
	// that numbers split but tokens can contain numbers if the dont start with them
	assert_eq!(lex_str!("0test test0 test0test 12345"),Some(vec![
		Token { content: "0".to_string(), pos: Pos::new(1, 1) },
		Token { content: "test".to_string(), pos: Pos::new(1, 2) },
		Token { content: "test0".to_string(), pos: Pos::new(1, 7) },
		Token { content: "test0test".to_string(), pos: Pos { line: 1, chr: 13 } },
		Token { content: "12345".to_string(), pos: Pos { line: 1, chr: 23 } },
	]));
}
