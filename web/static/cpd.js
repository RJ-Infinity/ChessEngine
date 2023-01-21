const doubleChars = ["&","|","=","<",">"];
const equalTypes = ["!","<",">"];
const exprChars = [
	"@","<",">","-","[","]","$","£","+",".","*","{","}","(",")",":","/",",","!","\"","%","Σ","→","=",";"
].concat(doubleChars);
const whiteSpaceChars = [" ","\t","\n"];
export class CPD{
	Data = null;
	Tokens = null;
	TypeTokens = null;
	Filename;
	constructor(filename){
		this.Filename = filename;
	}
	Lexer(){
		if(this.Data === null){
			throw "Lexer Error: no data, Data must be added before lexing can start";
		}
		this.Tokens = [];
		var line = 1;
		var character = 0;
		var currExpr = "";
		var currExprPos = null;
		var inInlineComment = false;
		var inComment = false;
		const addToken = (data,pos) => {
			this.Tokens.push({content:data,pos:pos});
			currExpr="";
			currExprPos = null;
		};
		this.Data.split("").forEach(char => {
			character++;
			if (char == "\n"){
				line++;
				character = 0;
				if (inComment){
					inComment = false;
					currExpr="";
					currExprPos = null;
				}
			}
			if(inInlineComment && currExpr.substring(currExpr.length-2)=="*/"){
				inInlineComment=false;
				currExpr="";
				currExprPos = null;
			}

			if (inComment || inInlineComment){
				currExpr+=char;
				return;
			}

			if (
				this.Tokens.at(-1)?.content == "/" &&
				char == "*" &&
				this.Tokens.at(-1)?.pos.line == line &&
				this.Tokens.at(-1)?.pos.char == character-1
			){
				inInlineComment = true;
				this.Tokens.pop();
				currExpr="";
				currExprPos = null;
			}else if (
				this.Tokens.at(-1)?.content == "/" &&
				char == "/" &&
				this.Tokens.at(-1)?.pos.line == line &&
				this.Tokens.at(-1)?.pos.char == character-1
			){
				inComment = true;
				this.Tokens.pop();
				currExpr="";
				currExprPos = null;
			}else if(exprChars.includes(char)){
				if (currExpr.length>0){addToken(currExpr,currExprPos);}
				if(
					doubleChars.includes(char) &&
					this.Tokens.at(-1)?.content == char &&
					this.Tokens.at(-1)?.pos.line == line &&
					this.Tokens.at(-1)?.pos.char == character-1
				){this.Tokens.at(-1).content = char+char;}
				else if(
					char=="=" &&
					equalTypes.includes(this.Tokens.at(-1)?.content) &&
					this.Tokens.at(-1)?.pos.line == line &&
					this.Tokens.at(-1)?.pos.char == character-1
				){this.Tokens.at(-1).content += char;}
				else{addToken(char,{line:line,char:character});}
			}else if(whiteSpaceChars.includes(char)){
				if (currExpr.length>0){addToken(currExpr,currExprPos);}
			}else{
				if (currExprPos==null){
					currExprPos={line:line,char:character};
				}
				currExpr+=char
			}
		});
	}
	Parser(){
		if(this.Tokens === null){
			throw "Parser Error: no tokens, Tokens must be added or generated before parsing can start";
		}
		this.TypeTokens={args:[]};
		var currTT=this.TypeTokens;
		const assert=(expr,message,pos)=>{if (!expr){
			console.log(this.TypeTokens);
			throw "Parser Error: "+message+" at "+this.prettyPrintPos(pos)
		}}
		const addToken=(name,pos)=>{
			pushToken(name,pos);
			currTT=currTT.args.at(-1);
		}
		const pushToken=(name,pos)=>{
			currTT.args.push({type:name,args:[],parent:currTT,pos:pos});
		}
		const closeToken=(expr,message,pos)=>{
			assert(expr,message,pos);
			currTT=currTT.parent;
		}
		var peiceRefState="";
		this.Tokens.forEach(token=>{
			if (peiceRefState!==""){
				switch (peiceRefState){
					case "open":{
						currTT.args.push({
							type:"text",
							args:token.content,
							parent:currTT,
							pos:token.pos
						});
						peiceRefState = "full";
					}break;
					case "full":{
						closeToken(
							token.content==="\"",
							"only one token alowed in a Peice Reference",
							token.pos
						);
						peiceRefState = "";
					}
				}
				return;
			}
			switch (token.content) {
				case "@":{
					assert(currTT.type==="OuterDef","'@' must be the first token in a 'OuterDef'",token.pos);
					currTT.type="OuterDefPeice";
				}break;
				case "<":{addToken("OuterDef",token.pos);}break;
				case ">":{closeToken(currTT.type.startsWith("OuterDef"),"unmatched '>'",token.pos);}break;
				case "-":{pushToken("sub",token.pos);}break;
				case "[":{addToken("List",token.pos);}break;
				case "]":{closeToken(currTT.type=="List","unmatched ']'",token.pos);}break;
				case "$":{pushToken("event",token.pos);}break;
				case "£":{pushToken("var",token.pos);}break;
				case "+":{pushToken("add",token.pos);}break;
				case ".":{pushToken("seperator",token.pos);}break;
				case "*":{pushToken("mul",token.pos);}break;
				case "{":{addToken("Expresion",token.pos);}break;
				case "}":{closeToken(currTT.type=="Expresion","unmatched '}'",token.pos);}break;
				case "(":{addToken("Statement",token.pos);}break;
				case ")":{closeToken(currTT.type=="Statement","unmatched ')'",token.pos);}break;
				case ":":{pushToken("condition",token.pos);}break;
				case "/":{pushToken("div",token.pos);}break;
				case ",":{pushToken("sep",token.pos);}break;
				case "!":{pushToken("inv",token.pos);}break;
				case "\"":{
					addToken("PeiceRef",token.pos);
					peiceRefState="open";
				}break;
				case "%":{pushToken("mod",token.pos);}break;
				case "Σ":{pushToken("sum",token.pos);}break;
				case "→":{pushToken("map",token.pos);}break;
				case "=":{pushToken("set",token.pos);}break;
				case ";":{pushToken("ins",token.pos);}break;
				case "&&":{pushToken("and",token.pos);}break;
				case "||":{pushToken("or",token.pos);}break;
				case "==":{pushToken("eq",token.pos);}break;
				case "<<":{pushToken("lt",token.pos);}break;
				case ">>":{pushToken("gt",token.pos);}break;
				case "!=":{pushToken("neq",token.pos);}break;
				case "<=":{pushToken("leq",token.pos);}break;
				case ">=":{pushToken("geq",token.pos);}break;

				default:{
					console.log(token.content)
					currTT.args.push({
						type:"text",
						args:token.content,
						parent:currTT,
						pos:token.pos
					});
				}break;
			}
		});
		
	}
	prettyPrintPos=pos=>`${this.Filename}:${pos.line},${pos.char}`
}

window.CPD = CPD;