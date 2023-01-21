const doubleChars = ["&","|","="];
const exprChars = [
	"@","<",">","-","[","]","$","£","+",".","*","{","}","(",")",":","/",",","!","\"","%","Σ","→"
].concat(doubleChars);
const whiteSpaceChars = [" ","\t","\n"];
export class CPD{
	Data = null;
	Tokens = null;
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
					this.Tokens.at(-1)?.content == "!" &&
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
		this.Tokens.forEach(token=>{

		});
	}
	static prettyPrintPos=pos=>`${Filename}:${pos.line},${pos.char}`
}

window.CPD = CPD;