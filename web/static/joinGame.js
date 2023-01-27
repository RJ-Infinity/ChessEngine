pinInput = document.getElementById("JoinPin");
pinInput.addEventListener("keydown",e=>{if (event.key == "Enter"){Join();}});
function Join(){window.location.href = `/Game/${pinInput.value}`}
document.getElementById("JoinButton").addEventListener("click",Join);