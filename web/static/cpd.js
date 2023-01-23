const doubleChars = ["&","|","=","<",">"];
const equalTypes = ["!","<",">"];
const exprChars = [
	"@","<",">","-","[","]","$","£","+",".","*","{","}","(",")",":","/",",","!","\"","%","Σ","→","=",";"
].concat(doubleChars).concat(equalTypes);
const whiteSpaceChars = [" ","\t","\n"];
const numbers = ["1","2","3","4","5","6","7","8","9","0"];
const letters = ["a","b","c","d","e","f","g","h","i","j","k","l","m","n","o","p","q","r","s","t","u","v","w","x","y","z"]
const allValidChars = exprChars.concat(whiteSpaceChars).concat(numbers).concat(letters);
export class CPD{
	// TODO: Add indicies suport exponent
	Data = null;
	Tokens = null;
	TypeTokens = null;
	SyntaxTree = null;
	Filename;
	constructor(filename){
		this.Filename = filename;
	}
	Lexer(){
		if(this.Data === null){
			throw "Lexer Error: no data, Data must be added before lexing can start";
		}
		var index = this.Data.indexOf("σ");
		if (index!==-1){
			var lines = this.Data.substring(0,index).split("\n")
			throw `Lexer Error: invalid char 'σ' at ${this.prettyPrintPos({
				line:lines.length,
				char:lines.at(-1).length+1
			})}`;
		}
		this.Data = this.Data.toLowerCase().replaceAll("σ","Σ");
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
			if (!allValidChars.includes(char)){
				throw `Lexer Error: invalid char '${char}' at ${this.prettyPrintPos({line:line,char:character})}`;
			}
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
	Tokeniser(){
		if(this.Tokens === null){
			throw "Tokeniser Error: no tokens, Tokens must be added or generated before tokenising can start";
		}
		const assert=(expr,message,pos)=>{if (!expr){
			throw "Tokeniser Error: "+message+" at "+this.prettyPrintPos(pos)
		}}
		this.TypeTokens={args:[]};
		var currTT=this.TypeTokens;
		const addToken=(name,pos)=>{
			pushToken(name,pos);
			currTT=currTT.args.at(-1);
			currTT.recurse = true;
		}
		const pushToken=(name,pos)=>{
			currTT.args.push({type:name,args:[],parent:currTT,pos:pos,recurse:false});
		}
		const closeToken=(expr,message,pos)=>{
			assert(expr,message,pos);
			currTT=currTT.parent;
		}
		var pieceRefState="";
		this.Tokens.forEach(token=>{
			if (pieceRefState!==""){
				switch (pieceRefState){
					case "open":{
						currTT.args.push({
							type:"text",
							args:token.content,
							parent:currTT,
							pos:token.pos
						});
						pieceRefState = "full";
					}break;
					case "full":{
						closeToken(
							token.content==="\"",
							"only one token alowed in a Piece Reference",
							token.pos
						);
						pieceRefState = "";
					}
				}
				return;
			}
			switch (token.content) {
				case "@":{
					assert(currTT.type==="OuterDef","'@' must be the first token in a 'OuterDef'",token.pos);
					currTT.type+="Piece";
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
					addToken("PieceRef",token.pos);
					pieceRefState="open";
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
	Parser(){
		if(this.TypeTokens === null){
			throw "Parser Error: no type tokens, TypeTokens must be added or generated before parsing can start";
		}
		const assert=(expr,message,pos)=>{if (!expr){
			throw "Parser Error: "+message+" at "+this.prettyPrintPos(pos)
		}}
		this.SyntaxTree=[];
		const joinUnaryOp=(token, index, type, tokenRepr, validTypes, allowedTypes)=>{
			assert(
				validTypes.includes(token.args[index+1].type),
				`can only use a '${tokenRepr}' on a ${allowedTypes}`,
				token.args[index].pos
			);
			token.args[index] = {
				type:type,
				pos:token.args[index].pos,
				recurse:true,
				args:token.args[index+1]
			};
			token.args.splice(index+1, 1);
		}
		const joinUnaryOps=(token, type, tokenType, tokenRepr, validTypes, allowedTypes)=>{
			var index = token.args.findIndex(token=>token.type==tokenType);
			while (token.args[index]!==undefined){
				joinUnaryOp(token, index, type, tokenRepr, validTypes, allowedTypes);
				index = token.args.findIndex(token=>token.type==tokenType);
			}
		}
		const joinBinaryOps=(
			token,
			type,
			tokenType,
			tokenRepr,
			validTypes,
			allowedTypes,
			tryUnary=false
		)=>{
			var index = token.args.findIndex(token=>token.type==tokenType);
			while (token.args[index]!==undefined){
				if (index===0){
					assert(
						tryUnary,
						`there must be a value before a '${tokenRepr}'`,
						token.args[index].pos
					);
					joinUnaryOp(token, index, type, tokenRepr, validTypes, allowedTypes)
				}else{
					assert(
						validTypes.includes(token.args[index-1].type) ||
						validTypes.includes(token.args[index+1].type),
						`can only use a '${tokenRepr}' on a ${allowedTypes}`,
						token.args[index].pos
					);
					token.args[index-1] = {
						type:type,
						pos:token.args[index-1].pos,
						recurse:true,
						args:[token.args[index-1],token.args[index+1]]
					};
					token.args.splice(index, 2);
				}
				index = token.args.findIndex(token=>token.type==tokenType);
			}
		}
		const ParseArgs=(token)=>{
			if (!token.recurse){return token;}
			var treeRoot = {};
			treeRoot.type=token.type;
			treeRoot.pos=token.pos;
			// conditionalExpr
			joinUnaryOps(token,"condExpr","condition",":",["Expresion"],"expresion");
			token
			.args
			.map((t,i)=>t.type==="condExpr"?i:null)
			.filter(i=>i!==null)
			.forEach(i=>token.args[i].args=token.args[i].args.args);
			// property
			var index = token.args.findIndex(token=>token.type=="seperator");
			while (token.args[index]!==undefined){
				assert(
					index>0,
					"there must be a property before a '.'",
					token.args[index].pos
				);
				assert(
					token.args[index-1].type==="text"||token.args[index-1].type==="property",
					"can only use a '.' on a text node",
					token.args[index].pos
				);
				if (token.args[index-1].type==="text"){
					token.args[index-1] = {
						type:"property",
						pos:token.args[index-1].pos,
						recurse:false,
						args:[token.args[index-1].args]
					}
				}
				assert(
					token.args[index-1].type==="property",
					"FATAL UNREACHABLE near",
					token.args[index-1].pos||token.args[index].pos
				);
				assert(
					token.args[index+1].type==="text",
					"can only use a '.' on a text node",
					token.args[index+1].pos
				);
				//remove the "."
				token.args.splice(index, 1);
				//add the next part
				token.args[index-1].args.push(token.args.splice(index, 1)[0].args);
				index = token.args.findIndex(token=>token.type=="seperator");
			}
			// variables
			joinUnaryOps(token,"variable","var","£",["property","text"],"property");
			// bidmas
			// brackets are allready handled by tokeniser
			// indicies dont yet exist
			// division
			joinBinaryOps(
				token,
				"division",
				"div",
				"/",
				["property","text","Statement","division"],
				"text node or statement"
			);
			// multiplication
			joinBinaryOps(
				token,
				"multiplication",
				"mul",
				"*",
				["property","text","Statement","division","multiplication"],
				"text node or statement"
			);
			// addition
			joinBinaryOps(
				token,
				"addition",
				"add",
				"+",
				["property","text","Statement","division","multiplication","addition","List"],
				"text node, statement or List",
				true
			);
			// subtraction
			joinBinaryOps(
				token,
				"subtraction",
				"sub",
				"-",
				["property","text","Statement","division","multiplication","addition","subtraction","List"],
				"text node, statement or List",
				true
			);
			index = 0;
			if (token.type==="OuterDefPiece"){
				assert(token.args[index]?.type === "text","piece definition has no name",token.pos);
				treeRoot.name = token.args[index].args;
				treeRoot.type = "PieceDefinition"
				treeRoot.properties = [];
				index++;
				var name;
				while (token.args[index] !== undefined){
					name = token.args[index];
					assert(name.type==="text","named property key must be a text Node",name.pos);
					index++;
					assert(token.args[index]?.type==="set","named property name must be followed by '='",token.args[index]?.pos||token.args[index-1].pos);
					index++;
					assert(token.args[index] !== undefined,"named property must have a value",token.args[index-1].pos);
					treeRoot.properties.push({type:"namedProperty",key:name.args,value:ParseArgs(token.args[index])});
					index++;
				}
			}
			if (token.type==="OuterDef"){
				treeRoot.args=token.args;
			}
			return treeRoot;
		}
		this.TypeTokens.args.forEach(
			token=>this.SyntaxTree.push(ParseArgs(token))
		);
	}
	prettyPrintPos=pos=>`${this.Filename}:${pos.line}:${pos.char}`
}

window.CPD = CPD;