var nameDialog = document.getElementById("nameDialog");
var userNameTextBox = document.getElementById("userNameTextBox");
var chessBoard = document.getElementById("ChessBoard");
var ErrorModal = document.getElementById("ErrorModal");
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
nameDialog.showModal();
nameDialog.addEventListener('cancel', e => e.preventDefault());
nameDialog.addEventListener('close', e => {
	let xhr = new XMLHttpRequest();
	xhr.open("POST", "/joinGame");
	
	xhr.setRequestHeader("Accept", "application/json");
	xhr.setRequestHeader("Content-Type", "application/json");

	xhr.onload = () => {
		data = JSON.parse(xhr.responseText);
		if ("error" in data){
			ErrorModal.textContent = "ERROR: " + data["error"];
			ErrorModal.showModal();
			return;
		}
		console.log(data)
	}
	xhr.send({
		userName: userNameTextBox.value,
		pin: pin,
	});

	xhr = new XMLHttpRequest();
	xhr.open("GET", `/getGameObject/${pin}`);
	
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
		chessBoard.setAttribute("size",data["size"]);
		chessBoard.refreshBoard();
	}
	xhr.send();

});
