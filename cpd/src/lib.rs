use wasm_bindgen::prelude::*;

#[cfg(test)]
mod test;

fn is_double_char(chr: &char)->bool{ ['&','|','=','<','>'].contains(chr) }
fn is_equal_type(chr: &char)->bool{ ['!','<','>'].contains(chr) }
fn is_expr_char(chr: &char)->bool{['@','#','<','>','-','[',']','$','£','+','.','*','{','}','(',')',':','/',',','!','"','%','Σ','→','='].contains(chr) || is_double_char(chr) || is_equal_type(chr) }
fn is_white_space_char(chr: &char)->bool{ [' ','\t','\n'].contains(chr) }
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


#[derive(PartialEq, Clone, Debug)]
pub struct Token{
	pub content: String,
	pub pos: Pos,
}

#[wasm_bindgen]
#[derive(Debug)]
pub struct Interpreter{
	#[wasm_bindgen(skip)]
	pub data: Option<String>,
	tokens: Option<Vec<Token>>,
	// type_tokens: Option<>,
	// syntax_tree: Option<>,
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
		filename: filename,
	}}
	///preforms the lexing. Requires data to be not None
	pub fn lexer(&mut self) -> Result<(),String>{
		if self.data.is_none(){return Err("Lexer Error: no data, Data must be added before lexing can start".to_string());}
		
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
			if !is_all_valid_char(&chr){return Err(format!("Lexer Error: invalid char '{}' at {}", chr, self.pretty_print_pos(pos)));}
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
				if 
					is_double_char(&chr) &&
					tokens.last() == Some(&Token{
						content: chr.to_string(),
						pos: Pos::new(pos.line, pos.chr-1)
					})
				{tokens.last_mut().unwrap().content += &chr.to_string()}
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
				if let Some(first_chr) = curr_expr.chars().next(){
					if is_number(&first_chr) && is_letter(&chr){
						tokens.push(Token{content: curr_expr, pos: curr_expr_pos.unwrap(),});
						curr_expr=String::new();
						curr_expr_pos = None;
					}
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
	pub fn get_debug(&self)->String{format!("{:#?}",self)}
	fn pretty_print_pos(&self, pos: &Pos)->String{format!("{}:{}:{}", self.filename, pos.line, pos.chr)}
}