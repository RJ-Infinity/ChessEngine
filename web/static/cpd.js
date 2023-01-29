const doubleChars = ["&","|","=","<",">"];
const equalTypes = ["!","<",">"];
const exprChars = [
	"@","<",">","-","[","]","$","£","+",".","*","{","}","(",")",":","/",",","!","\"","%","Σ","→","="
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
			if (char === "\n"){
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
				this.Tokens.at(-1)?.content === "/" &&
				char === "*" &&
				this.Tokens.at(-1)?.pos.line === line &&
				this.Tokens.at(-1)?.pos.char === character-1
			){
				inInlineComment = true;
				this.Tokens.pop();
				currExpr="";
				currExprPos = null;
			}else if (
				this.Tokens.at(-1)?.content === "/" &&
				char === "/" &&
				this.Tokens.at(-1)?.pos.line === line &&
				this.Tokens.at(-1)?.pos.char === character-1
			){
				inComment = true;
				this.Tokens.pop();
				currExpr="";
				currExprPos = null;
			}else if(exprChars.includes(char)){
				if (currExpr.length>0){addToken(currExpr,currExprPos);}
				if(
					doubleChars.includes(char) &&
					this.Tokens.at(-1)?.content === char &&
					this.Tokens.at(-1)?.pos.line === line &&
					this.Tokens.at(-1)?.pos.char === character-1
				){this.Tokens.at(-1).content = char+char;}
				else if(
					char=="=" &&
					equalTypes.includes(this.Tokens.at(-1)?.content) &&
					this.Tokens.at(-1)?.pos.line === line &&
					this.Tokens.at(-1)?.pos.char === character-1
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
							pos:token.pos,
							recurse:false
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
					currTT.recurse = false;
				}break;
				case "%":{pushToken("mod",token.pos);}break;
				case "Σ":{pushToken("sum",token.pos);}break;
				case "→":{pushToken("map",token.pos);}break;
				case "=":{pushToken("set",token.pos);}break;
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
						pos:token.pos,
						recurse:false
					});
				}break;
			}
		});
	}
	static tokenCloner=(token)=>({
		type:token.type,
		args:Array.isArray(token.args)?token.args.map(CPD.tokenCloner):token.args,
		pos:{line:token.pos.line,char:token.pos.char},
		recurse:token.recurse
	})
	Parser(){
		if(this.TypeTokens === null){
			throw "Parser Error: no type tokens, TypeTokens must be added or generated before parsing can start";
		}
		const assert=(expr,message,pos)=>{if (!expr){
			throw "Parser Error: "+message+" at "+this.prettyPrintPos(pos)
		}}
		this.SyntaxTree=[];
		//clone the typeTokens & remove parent
		var newTokens = this.TypeTokens.args.map(CPD.tokenCloner);
		// set up recursive funcs
		const joinUnaryOp=(token, index, type, tokenRepr, validTypes, allowedTypes)=>{
			assert(
				validTypes.includes(token.args[index+1].type),
				`can only use a '${tokenRepr}' on a ${allowedTypes}`,
				token.args[index].pos
			);
			token.args[index] = {
				type:type,
				pos:token.args[index].pos,
				recurse:false,
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
		const joinBinaryOp=(
			token,
			index,
			newType,
			tokenRepr,
			validTypes,
			allowedTypes,
			tryUnary=false
		)=>{
			if (index===0){
				assert(
					tryUnary,
					`there must be a value before a '${tokenRepr}'`,
					token.args[index].pos
				);
				joinUnaryOp(token, index, newType, tokenRepr, validTypes, allowedTypes)
			}else{
				assert(
					validTypes.includes(token.args[index-1].type) &&
					validTypes.includes(token.args[index+1]?.type),
					`can only use a '${tokenRepr}' on a ${allowedTypes}`,
					token.args[index].pos
				);
				token.args[index-1] = {
					type:newType,
					pos:token.args[index-1].pos,
					recurse:false,
					args:[ParseArgs(token.args[index-1]),ParseArgs(token.args[index+1])]
				};
				token.args.splice(index, 2);
			}
		}
		const joinBinaryOps=(
			token,
			newType,
			oldType,
			tokenRepr,
			validTypes,
			allowedTypes,
			tryUnary=false
		)=>{
			var index = token.args.findIndex(token=>token.type==oldType);
			while (token.args[index]!==undefined){
				joinBinaryOp(
					token,
					index,
					newType,
					tokenRepr,
					validTypes,
					allowedTypes,
					tryUnary
				);
				index = token.args.findIndex(token=>token.type==oldType);
			}
		}
		const joinMultipleBinaryOps=(
			token,
			oldNewMatch,
			validTypes,
			allowedTypes,
			tryUnary = false
		)=>{
			var index = token.args.findIndex(token=>Object.keys(oldNewMatch).includes(token.type));
			while (token.args[index]!==undefined){
				joinBinaryOp(
					token,
					index,
					oldNewMatch[token.args[index].type][0],
					oldNewMatch[token.args[index].type][1],
					validTypes,
					allowedTypes,
					tryUnary
				);
				index = token.args.findIndex(token=>Object.keys(oldNewMatch).includes(token.type));
			}
		}
		const ParseArgs=(token)=>{
			if (!token.recurse){return token;}
			var treeRoot = {};
			treeRoot.type=token.type;
			treeRoot.pos=token.pos;
			treeRoot.recurse=token.recurse;
			// pieceref
			token.args = token.args.map(t=>{
				if (t.type !== "PieceRef"){return t;}
				assert(
					t.args.length === 1 && t.args[0].type=="text",
					"a piece ref must contain a single text node",
					t.pos
				);
				return {
					args: t.args[0].args,
					type: "PieceReference",
					recurse: false,
					pos: t.pos
				}
			});
			// split text into number and str
			token.args = token.args.map(t=>{
				if (t.type !== "text"){return t;}
				if (t.args.split("").map(c=>letters.includes(c)?"":c).join("").length>0){
					assert(
						t.args.split("").map(c=>numbers.includes(c)?"":c).join("").length==0,
						"text can only contain letters or numbers",
						t.pos
					);
					t.type = "int";
					return t;
				}
				t.type = "str";
				return t;
			});
			// function call
			var index = token.args.findIndex(
				(t,i)=>t.type==="str" &&
				token.args[i+1]?.type==="Statement"
			);
			while (token.args[index]!==undefined){
				var functionStatement = ParseArgs(token.args[index+1]);
				token.args[index] = {
					type:"Function",
					pos:token.args[index].pos,
					recurse:false,
					name:token.args[index].args,
					args: functionStatement.args
				};
				token.args.splice(index+1, 1);
				index = token.args.findIndex(
					(t,i)=>t.type==="str" &&
					token.args[i+1]?.type==="Statement"
				);
			}
			// conditionalExpr
			joinUnaryOps(token,"condExpr","condition",":",["Expresion"],"expresion");
			token
			.args
			.map((t,i)=>t.type==="condExpr"?i:null)
			.filter(i=>i!==null)
			.forEach(i=>token.args[i].args=token.args[i].args.args);
			token.args.filter(t=>t.type==="condExpr").forEach(t=>t.recurse = true);
			token.args = token.args.map(t=>t.type==="condExpr"?ParseArgs(t):t);
			// event
			index = token.args.findIndex(token=>token.type=="event");
			while (token.args[index]!==undefined){
				if (token.args[index+1].type==="str"){
					token.args[index] = {
						type: "namedEvent",
						name: token.args.splice(index+1, 1)[0].args,
						pos:token.args[index].pos,
						recurse:false
					};
					if (token.args[index+1].type === "condExpr"){
						token.args[index].conditionalExpresion = ParseArgs(token.args.splice(index+1, 1)[0]);
					}
					assert(
						token.args[index+1].type === "Expresion",
						"A named event must be followed by an optional conditional expresion then an expresion block",
						token.args[index].pos
					);
					token.args[index].Expresion = ParseArgs(token.args.splice(index+1, 1)[0]);
				}else{
					assert(
						token.args[index+1].type==="Expresion",
						"can only use a '$' on a string or expression",
						token.args[index].pos
					);
					token.args[index] = {
						type:"implicitEvent",
						pos:token.args[index].pos,
						recurse:false,
						args:ParseArgs(token.args[index+1])
					};
					token.args.splice(index+1, 1);
				}
				index = token.args.findIndex(token=>token.type=="event");
			}
			// property
			index = token.args.findIndex(token=>token.type=="seperator");
			while (token.args[index]!==undefined){
				assert(
					index>0,
					"there must be a property before a '.'",
					token.args[index].pos
				);
				assert(
					token.args[index-1].type==="str" ||
					token.args[index-1].type==="property" ||
					token.args[index-1].type==="Function",
					"can only use a '.' on a string node",
					token.args[index].pos
				);
				if (token.args[index-1].type==="str"){
					token.args[index-1] = {
						type:"property",
						pos:token.args[index-1].pos,
						recurse:false,
						args:[token.args[index-1].args]
					}
				}
				if (token.args[index-1].type==="Function"){
					token.args[index-1] = {
						type:"property",
						pos:token.args[index-1].pos,
						recurse:false,
						args:[token.args[index-1]]
					}
				}
				assert(
					token.args[index-1].type==="property",
					"FATAL UNREACHABLE near",
					token.args[index-1].pos||token.args[index].pos
				);
				assert(
					token.args[index+1].type==="str" ||
					token.args[index+1].type==="Function",
					"can only use a '.' on a string node",
					token.args[index+1].pos
				);
				//remove the "."
				token.args.splice(index, 1);
				//add the next part
				if (token.args[index].type==="str"){
					token.args[index-1].args.push(token.args.splice(index, 1)[0].args);
				}else if (token.args[index].type==="Function"){
					token.args[index-1].args.push(token.args.splice(index, 1)[0]);
				}else{
					assert(false,"FATAL UNREACHABLE near",token.args[index].pos||token.args[index-1].pos);
				}
				index = token.args.findIndex(token=>token.type=="seperator");
			}
			// variables
			joinUnaryOps(token,"variable","var","£",["property","str"],"property");
			//not operator
			joinUnaryOps(token,"inversion","inv","!",["variable","property","str"],"not");
			// bidmas
			// brackets are allready handled by tokeniser
			// indicies dont yet exist
			// division
			// multiplication
			// modulus is like division
			var oldNewMatch = {
				"div":["division","/"],
				"mul":["multiplication","*"],
				"mod":["modulus","%"],
			}
			var oldNewMatchSecondary = {
				"add":["addition","+"],
				"sub":["subtraction","-"]
			}
			index = token.args.findIndex(token=>Object.keys(oldNewMatch).includes(token.type));
			var validTypes = ["property","str","variable","int","Statement","division","multiplication","modulus","addition","subtraction"];
			while (token.args[index]!==undefined){
				assert(
					index!==0,
					`there must be a value before a '${oldNewMatch[token.args[index].type][1]}'`,
					token.args[index].pos
				);
				// allow to operate on unary + or - i.e. (8*-9)
				if (Object.keys(oldNewMatchSecondary).includes(token.args[index+1]?.type)){
					joinUnaryOp(
						token,
						index+1,
						oldNewMatchSecondary[token.args[index+1].type][0],
						oldNewMatchSecondary[token.args[index+1].type][1],
						validTypes,
						"property, int, statement if proceded by a '"+oldNewMatch[token.args[index].type][1]+"'"
					);
				}
				assert(
					validTypes.includes(token.args[index-1].type) &&
					validTypes.includes(token.args[index+1]?.type),
					`can only use a '${oldNewMatch[token.args[index].type][1]}' on a property, int or statement`,
					token.args[index].pos
				);
				token.args[index-1] = {
					type:oldNewMatch[token.args[index].type][0],
					pos:token.args[index-1].pos,
					recurse:false,
					args:[ParseArgs(token.args[index-1]),ParseArgs(token.args[index+1])]
				};
				token.args.splice(index, 2);
				index = token.args.findIndex(token=>Object.keys(oldNewMatch).includes(token.type));
			}
			// addition
			// subtraction
			joinMultipleBinaryOps(
				token,
				oldNewMatchSecondary,
				["property","str","variable","int","Statement","division","multiplication","modulus","addition","subtraction","List"],
				"property, int, statement or List",
				true
			);
			// comparisons
			joinMultipleBinaryOps(
				token,
				{
					"gt":["greaterThan",">>"],
					"lt":["lessThan","<<"],
					"geq":["greaterThanEq",">="],
					"leq":["lessThanEq","<="]
				},
				[
					"property","str","variable","int","Statement",
					"division","multiplication","modulus",
					"addition","subtraction","List",
					"greaterThan","lessThan","greaterThanEq","lessThanEq"
				],
				"property, int, statement, bool or List"
			);
			// equality
			joinMultipleBinaryOps(
				token,
				{
					"eq":["Equal","=="],
					"neq":["NotEqual","!="],
				},
				[
					"inversion","PieceReference",
					"property","str","variable","int","Statement",
					"division","multiplication","modulus",
					"addition","subtraction","List",
					"greaterThan","lessThan","greaterThanEq","lessThanEq",
					"Equal", "NotEqual"
				],
				"property, int, statement or bool"
			);
			joinBinaryOps(token,"logicalAnd","and","and",[
				"inversion",
				"property","str","variable","int","Statement",
				"division","multiplication","modulus",
				"addition","subtraction","List",
				"greaterThan","lessThan","greaterThanEq","lessThanEq",
				"Equal", "NotEqual",
				"logicalAnd"
			],"property, int, statement or bool");

			joinBinaryOps(token,"logicalOr","or","or",[
				"inversion",
				"property","str","variable","int","Statement",
				"division","multiplication","modulus",
				"addition","subtraction","List",
				"greaterThan","lessThan","greaterThanEq","lessThanEq",
				"Equal", "NotEqual",
				"logicalAnd", "logicalOr"
			],"property, int, statement or bool");

			//↑↑↑↑↑↑↑↑↑↑↑↑↑↑ this is the parsing of the args
			//↓↓↓↓↓↓↓↓↓↓↓↓↓↓ this is the creation of the treeRoot

			// variable mapping

			index = token.args.findIndex(token=>token.type==="map");
			if(token.args[index]!==undefined){
				assert(
					index>0 &&
					token.args[index-1].type === "str",
					"a mapping must be proceded by a variable mapping",
					token.args[index].pos
				);
				assert(
					token.type === "Statement" &&
					index === 1 &&
					token.args.length === 3,
					"a maping must by the only item in a statement",
					token.args[index].pos
				);
				assert(
					token.args[index+1].type === "Statement",
					"a mapping's map must be enclosed in a statement",
					token.args[index].pos
				);
				treeRoot = {
					type: "mapping",
					pos: token.args[1].pos,
					recurse: false,
					variable: token.args[0].args,
					map: ParseArgs(token.args[2])
				}
			}
			
			// sigma notation

			index = token.args.findIndex(token=>token.type==="sum");
			if (token.args[index]!==undefined){
				//this dosent need to be while as sigma must be the only item in a statement
				assert(
					index>1 &&
					token.args[index-1].type === "Statement" &&
					token.args[index-2].type === "Statement",
					"sigma notation must have a proceding variable and initial value each in brackets '()'",
					token.args[index].pos
				);
				assert(
					token.type === "Statement" && index === 2,
					"sigma notation must the the only item in a statement",
					token.args[index].pos
				)
				var sigmaVariable=ParseArgs({
					type:token.args[0].type,
					args:token.args[0].args.map(CPD.tokenCloner),
					pos:{line:token.args[0].pos.line,char:token.args[0].pos.char},
					recurse:token.args[0].recurse
				});
				assert(
					(
						sigmaVariable.type === "Statement" &&
						sigmaVariable.args.length === 1 &&
						sigmaVariable.args[0].type === "str"
					) || sigmaVariable.type === "mapping",
					"sigma notation must have a variable or mapping of a variable as its -2 argument",
					token.args[2].pos
				);
				token.args.splice(0,1);
				treeRoot = {
					type:"Sigma",
					variable:sigmaVariable.type === "mapping"?sigmaVariable:sigmaVariable.args[0],
					pos:token.args[1].pos,
					recurse:false
				}
				treeRoot.initialVal = ParseArgs(token.args.splice(0,1)[0]);
				treeRoot.finalVal = ParseArgs(token.args.splice(1,1)[0]);
				if (token.args[1].type === "condExpr"){
					treeRoot.condExpr = ParseArgs(token.args.splice(1,1)[0]);
				}
				assert(
					token.args[1].type === "Expresion",
					"a sigma notation must have a value to append to the list",
					token.args[0].pos
				);
				treeRoot.Expr = ParseArgs(token.args.splice(1,1)[0]);
				assert(
					token.args.length === 1 || (
						token.args.length === 2 &&
						token.args[1].type === "condExpr"
					),
					"sigma notation must the the only thing in a statement",
					token.args[0].pos
				);
				if (token.args.length === 2){
					treeRoot.ExprCondition = ParseArgs(token.args.splice(1,1)[0]);
				}
			}

			index = 0;
			if (treeRoot.type==="OuterDefPiece"){
				assert(token.args[index]?.type === "str","piece definition has no name",token.pos);
				treeRoot.name = token.args[index].args;
				treeRoot.type = "PieceDefinition"
				treeRoot.properties = [];
				index++;
				var name;
				while (token.args[index] !== undefined){
					name = token.args[index];
					assert(name.type==="str","named property key must be a text Node",name.pos);
					index++;
					assert(token.args[index]?.type==="set","named property name must be followed by '='",token.args[index]?.pos||token.args[index-1].pos);
					index++;
					assert(token.args[index] !== undefined,"named property must have a value",token.args[index-1].pos);
					treeRoot.properties.push({
						type:"namedProperty",
						key:name.args,
						value:ParseArgs(token.args[index]),
						pos: name.pos
					});
					index++;
				}
			}
			if (
				treeRoot.type === "Statement" ||
				treeRoot.type === "condExpr" ||
				treeRoot.type === "Expresion"
			){
				treeRoot.args=token.args.map(arg=>ParseArgs(arg));
			}
			if (treeRoot.type === "List"){
				treeRoot.args=token.args.reduce((acc,arg)=>{
					if(arg.type==="sep"){
						acc.push([]);
					}else{
						acc.at(-1).push(ParseArgs(arg));
					}
					return acc;
				},[[]]);
			}
			if (treeRoot.type==="OuterDef"){
				treeRoot.args=token.args;
			}
			return treeRoot;
		}
		// apply modifications
		newTokens.forEach(token=>this.SyntaxTree.push(ParseArgs(token)));
	}
	prettyPrintPos=pos=>`${this.Filename}:${pos.line}:${pos.char}`
}

window.CPD = CPD;