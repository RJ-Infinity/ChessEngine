
#[derive(PartialEq, Clone, Copy, Debug)]
pub enum TokenKind{
	Root,
	OuterDef,
	OuterDefPiece,
	OuterDefColour,
	List,
	Data,
	Statement,
	PieceRef,
	Sub,
	Event,
	Var,
	Add,
	Seperator,
	Mul,
	Condition,
	Div,
	Sep,
	Inv,
	Mod,
	Sum,
	Map,
	Set,
	And,
	Or,
	Eq,
	Lt,
	Gt,
	Neq,
	Leq,
	Geq,
}

#[derive(Clone)]
pub enum Arg{
	Litteral{
		litteral:String,
		pos: Pos,
	},
	Token(Arc<RwLock<TypedToken>>)
}
impl std::fmt::Debug for Arg{
	fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
		match self {
			Self::Litteral { litteral, pos } => f.debug_struct("Litteral").field("litteral", litteral).field("pos", pos).finish(),
			Self::Token(_) => f.debug_tuple("Token").finish(),
		}
	}
}
impl PartialEq for Arg{fn eq(&self, other: &Self) -> bool {match (self, other) {
	(Self::Litteral {
		litteral: lhs_litteral,
		pos: lhs_pos
	}, Self::Litteral {
		litteral: rhs_litteral,
		pos: rhs_pos
	}) => lhs_litteral == rhs_litteral && lhs_pos == rhs_pos,
	(Self::Token(lhs), Self::Token(rhs)) => *lhs.read().unwrap() == *rhs.read().unwrap(),
	_ => false,
}}}


#[derive(Clone)]
pub struct TypedToken{
	pub kind: TokenKind,
	pub args: Vec<Arg>,
	pub parent: Option<Arc<RwLock<Self>>>,
	pub pos: Pos,
	pub recurse: bool,
}

impl std::fmt::Debug for TypedToken{fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
	f.debug_struct("TypedToken")
	.field("kind", &self.kind)
	.field("args", &"[]")
	.field("parent", &"..") // this prevents a recursion
	.field("pos", &self.pos)
	.field("recurse", &self.recurse)
	.finish()
}}
impl Into<Arg> for TypedToken{fn into(self) -> Arg {Arg::Token(RwLock::new(self).into())}}
impl PartialEq for TypedToken{fn eq(&self, other: &Self) -> bool {
	self.kind == other.kind && self.args == other.args && match (&self.parent, &other.parent){
		(Some(lhs), Some(rhs)) => *lhs.read().unwrap() == *rhs.read().unwrap(),
		(None, None) => true,
		_ => false,
	} && self.pos == other.pos && self.recurse == other.recurse
}}
enum PieceRefState{
	None,
	Open,
	Full,
}


macro_rules! tokeniser_assert {($self: ident, $expr: expr, $message: expr, $pos: expr) => {if !$expr{
	return Err(format!("Tokeniser Error: {} at {}",$message,$self.pretty_print_pos($pos)))
}};}
macro_rules! close_token {($self: ident, $curr_tt: ident, $expr: expr, $message: expr, $pos: expr) => {{
	tokeniser_assert!($self, $expr, $message, $pos);
	let tmp = $curr_tt.read().unwrap().parent.as_ref().unwrap().clone();
	$curr_tt = tmp;
}};}
macro_rules! push_token {($curr_tt: ident, $name: expr, $pos: expr) => {$curr_tt.write().unwrap().args.push(TypedToken{
	kind:$name,
	args:Vec::new().into(),
	parent:Some($curr_tt.clone()),
	pos:$pos,
	recurse:false
}.into())};}
macro_rules! open_token {($curr_tt: ident, $name: expr, $pos: expr) => {{
	push_token!($curr_tt, $name, $pos);
	let tmp = if let Arg::Token(arg) = $curr_tt.read().unwrap().args.last().unwrap().clone(){arg}
	else{todo!("i havent thought this case through properly yet")};
	$curr_tt = tmp;
	$curr_tt.write().unwrap().recurse = true;
}};}

fn parser(){
	while let Some(token) = token_gen.peek(){
		root.push(OuterDef::Peice::parse())
	}

	self.type_tokens=Some(RwLock::new(TypedToken{
		kind: TokenKind::Root,
		args:Vec::new().into(),
		parent: None,
		pos: Pos{line: 1,chr: 0,},
		recurse: false
	}).into());
	let mut curr_tt:Arc<RwLock<TypedToken>>=self.type_tokens.as_ref().unwrap().clone();
	let mut piece_ref_state=PieceRefState::None;
	for token in self.tokens.as_ref().unwrap(){
		match piece_ref_state{
			PieceRefState::Open =>{
				(*curr_tt).write().unwrap().args.push(Arg::Litteral{
					litteral:token.content.clone().into(),
					pos:token.pos,
				});
				piece_ref_state = PieceRefState::Full;
				continue;
			},
			PieceRefState::Full => {
				close_token!(self, curr_tt, token.content=="\"", "only one token alowed in a Piece Reference", &token.pos);
				piece_ref_state = PieceRefState::None;
				continue;
			},
			PieceRefState::None => {}
		}
		match token.content.as_str() {
			"@" => {
				tokeniser_assert!(
					self,
					curr_tt.read().unwrap().kind==TokenKind::OuterDef,
					"'@' must be the first token in a 'OuterDef'",
					&token.pos
				);
				curr_tt.write().unwrap().kind=TokenKind::OuterDefPiece;
			},
			"#" => {
				tokeniser_assert!(
					self,
					curr_tt.read().unwrap().kind==TokenKind::OuterDef,
					"'#' must be the first token in a 'OuterDef'",
					&token.pos
				);
				curr_tt.write().unwrap().kind=TokenKind::OuterDefColour;
			},
			"<" => open_token!(curr_tt, TokenKind::OuterDef, token.pos),
			">" => close_token!(
				self,
				curr_tt,
				curr_tt.read().unwrap().kind == TokenKind::OuterDef ||
				curr_tt.read().unwrap().kind == TokenKind::OuterDefPiece ||
				curr_tt.read().unwrap().kind == TokenKind::OuterDefColour,
				"unmatched '>'",
				&token.pos
			),
			"-" => push_token!(curr_tt, TokenKind::Sub,token.pos),
			"[" => open_token!(curr_tt, TokenKind::List, token.pos),
			"]" => close_token!(self,curr_tt,curr_tt.read().unwrap().kind==TokenKind::List,"unmatched ']'",&token.pos),
			"$" => push_token!(curr_tt, TokenKind::Event,token.pos),
			"£" => push_token!(curr_tt, TokenKind::Var,token.pos),
			"+" => push_token!(curr_tt, TokenKind::Add,token.pos),
			"." => push_token!(curr_tt, TokenKind::Seperator,token.pos),
			"*" => push_token!(curr_tt, TokenKind::Mul,token.pos),
			"{" => open_token!(curr_tt, TokenKind::Data, token.pos),
			"}" => close_token!(self,curr_tt,curr_tt.read().unwrap().kind==TokenKind::Data,"unmatched '}'",&token.pos),
			"(" => open_token!(curr_tt, TokenKind::Statement, token.pos),
			")" => close_token!(self,curr_tt,curr_tt.read().unwrap().kind==TokenKind::Statement,"unmatched ')'",&token.pos),
			":" => push_token!(curr_tt, TokenKind::Condition,token.pos),
			"/" => push_token!(curr_tt, TokenKind::Div,token.pos),
			"," => push_token!(curr_tt, TokenKind::Sep,token.pos),
			"!" => push_token!(curr_tt, TokenKind::Inv,token.pos),
			"\"" => {
				open_token!(curr_tt, TokenKind::PieceRef, token.pos);
				piece_ref_state=PieceRefState::Open;
				curr_tt.write().unwrap().recurse = false;
			},
			"%" => push_token!(curr_tt, TokenKind::Mod,token.pos),
			"Σ" => push_token!(curr_tt, TokenKind::Sum,token.pos),
			"→" => push_token!(curr_tt, TokenKind::Map,token.pos),
			"=" => push_token!(curr_tt, TokenKind::Set,token.pos),
			"&&" => push_token!(curr_tt, TokenKind::And,token.pos),
			"||" => push_token!(curr_tt, TokenKind::Or,token.pos),
			"==" => push_token!(curr_tt, TokenKind::Eq,token.pos),
			"<<" => push_token!(curr_tt, TokenKind::Lt,token.pos),
			">>" => push_token!(curr_tt, TokenKind::Gt,token.pos),
			"!=" => push_token!(curr_tt, TokenKind::Neq,token.pos),
			"<=" => push_token!(curr_tt, TokenKind::Leq,token.pos),
			">=" => push_token!(curr_tt, TokenKind::Geq,token.pos),

			_ => curr_tt.write().unwrap().args.push(Arg::Litteral{
				litteral:token.content.clone(),
				pos:token.pos,
			}),
		}
}
}