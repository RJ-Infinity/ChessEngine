
window.monacoEditors = {
	getEditor : (el)=>monacoEditors.editors.filter((ed)=>ed._domElement === el)[0],
	getElement : (ed)=>ed._domElement,
	add : (ed)=>monacoEditors.editors.push(ed),
	editors : [],
}
document.querySelectorAll("div[type=monaco]").forEach(el=>{
	window.monacoEditors.add(monaco.editor.create(el, {
		value: el.getAttribute("value") || "",
		language: el.getAttribute("lan"),
		renderWhitespace: "all",
		bracketPairColorization: {enabled:true},
		colorDecorators: true,
		dragAndDrop:true,
		// overflowWidgetsDomNode: el.parentElement,
		fixedOverflowWidgets:true,
		useShadowDOM:true,
	}));

	new ResizeObserver((e)=>{
		if(e[0].target===undefined)return
		// console.log(e[0])
		// window.monacoEditors.getEditor(e[0].target).layout();
	}).observe(el)
})
