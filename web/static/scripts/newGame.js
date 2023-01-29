
		function updateatr(json,att,val){
			json[att]=val;
			return json;
		}
		function upadteEditor(editor,att,val){
			try{
				json = JSON.parse(monacoEditors.getEditor(editor).getValue());
			}catch{
				json = {};
			}
			monacoEditors.getEditor(editor).getModel().setValue(
				JSON.stringify(
					updateatr(
						json,
						att,
						val
					),
					null,
					"\t"
				)
			);
		}

		var ErrorModal = document.getElementById("ErrorDialog");
		ErrorModal.addEventListener("click",e=>{
			rect = e.target.getBoundingClientRect();
			if (
				e.clientX < rect.left ||
				e.clientX > rect.right ||
				e.clientY < rect.top ||
				e.clientY > rect.bottom
			){
				e.target.close();
			}
		})
		var goBtn = document.getElementById("StartGame");
		goBtn.addEventListener("click",()=>{
			let xhr = new XMLHttpRequest();
			xhr.open("POST", "/createGame");
			
			xhr.setRequestHeader("Accept", "application/json");
			xhr.setRequestHeader("Content-Type", "application/json");

			xhr.onload = () => {
				console.log(xhr.responseText)
				data = JSON.parse(xhr.responseText);
				if ("error" in data){
					ErrorModal.textContent = "ERROR: " + data["error"];
					ErrorModal.showModal();
					return;
				}
				window.location.href = `/Game/${data.pin}`
			}

			// console.log(monacoEditors.getEditor(monacoJsonContainer).getValue())

			xhr.send(monacoEditors.getEditor(monacoJsonContainer).getValue());
		});


		var chessBoard = document.getElementById("ChessBoard");

		var monacoJsonContainer = document.querySelector(".setting.json div[type=monaco][lan=json]");

		var chessBoardWidthSlider = document.getElementById("chessBoardWidthSlider");
		var chessBoardWidth = document.getElementById("chessBoardWidth");

		chessBoardWidthSlider.value = 8;
		chessBoardWidth.value = 8;

		const chessBoardWidthInput = e=>{
			chessBoardWidth.value = e.target.value;
			chessBoardWidthSlider.value = e.target.value;
			chessBoard.setAttribute("size",e.target.value);
			chessBoard.refreshBoard();
			upadteEditor(monacoJsonContainer,"size",e.target.value);
		}
		chessBoardWidthSlider.addEventListener("input",chessBoardWidthInput);
		chessBoardWidth.addEventListener("input",chessBoardWidthInput);

		monacoJsonContainer.addEventListener("keydown",e=>{
			if (e.key == "Escape"){
				if(e.shiftKey){
					document.getElementById("jsonMoacoFullCheck").focus();
				}else{
					console.log(document.getElementById("escapeJsonMonaco"))
					document.getElementById("escapeJsonMonaco").focus();
				}
			}
		});

		var disabledPieces = document.getElementById("disabledPieces");
		var disablePieces = document.getElementById("disablePieces");
		var removeBtn = document.getElementById("removeBtn");

		disabledPieces.value = "";
		disablePieces.checked = false;
		var chessDisabledPieces = []
		
		disabledPieces.addEventListener("input",e => {
			chessDisabledPieces = e.target.value.split(" ").filter(el=>el!=="");
			highlightDisabledPieces();
		});
		const highlightDisabledPieces = ()=>{
			disabledPieces.parentNode.classList.remove("error");
			for (let i = 0; i < chessBoard.size; i++) {
				for (let j = 0; j < chessBoard.size; j++) {
					chessBoard.highlightPiece(chessBoard.toCoord({x:i,y:j}),"red",true);
				}
			}
			chessDisabledPieces.forEach(el => {
				try{
					if (!chessBoard.disabled.includes(chessBoard.toIndex(el))){
						chessBoard.highlightPiece(el,"red")
					}
				}catch (error){
					disabledPieces.parentElement.classList.add("error")
				}
			});
			chessBoard.disabled.forEach(el => {
				if (!chessDisabledPieces.includes(chessBoard.toCoord(el))){
					try{
						chessBoard.highlightPiece(chessBoard.toCoord(el),"red")
					}catch (error){
						disabledPieces.parentElement.classList.add("error")
					}
				}
			})
		}
		chessBoard.addEventListener("chessrefreshed",highlightDisabledPieces);
		chessBoard.addEventListener("chessclick",e=>{
			if(disablePieces.checked){
				if (chessDisabledPieces.includes(e.detail.pos)){
					disabledPieces.value = chessDisabledPieces.filter(el=>e.detail.pos!==el).join(" ");
				}else{
					disabledPieces.value = [...chessDisabledPieces, e.detail.pos].join(" ");
				}
				chessDisabledPieces = disabledPieces.value.split(" ").filter(el=>el!=="")
				highlightDisabledPieces();
			}
		})
		removeBtn.addEventListener("click",e=>{
			if (!disabledPieces.parentElement.classList.contains("error")){
				chessBoard.setAttribute("disabled",disabledPieces.value);
				chessBoard.refreshBoard();
				upadteEditor(monacoJsonContainer,"disabled",disabledPieces.value);
			}
		});
		
		monacoEditors.getEditor(monacoJsonContainer).getModel().onDidChangeContent((event) => {
			// console.log(event)
		});
	