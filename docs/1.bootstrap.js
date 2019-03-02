(window["webpackJsonp"] = window["webpackJsonp"] || []).push([[1],{

/***/ "./index.js":
/*!******************!*\
  !*** ./index.js ***!
  \******************/
/*! no static exports found */
/***/ (function(module, exports, __webpack_require__) {

eval("const agb = __webpack_require__.e(/*! import() */ 0).then(__webpack_require__.bind(null, /*! agb-web */ \"../pkg/agb_web.js\"));\n\nconst FRAMERATE = 59.7;\nlet canvas = document.getElementById(\"agb-canvas\");\nconst KEY_UP = 0;\nconst KEY_DOWN = 1;\nconst KEY_LEFT = 2;\nconst KEY_RIGHT = 3;\nconst KEY_B = 4;\nconst KEY_A = 5;\nconst KEY_SELECT = 6;\nconst KEY_START = 7;\n\nlet keyBindings = {\n\t\"ArrowUp\": KEY_UP,\n\t\"ArrowDown\": KEY_DOWN,\n\t\"ArrowLeft\": KEY_LEFT,\n\t\"ArrowRight\": KEY_RIGHT,\n\t\"z\": KEY_B,\n\t\"x\": KEY_A,\n\t\"c\": KEY_SELECT,\n\t\"v\": KEY_START\n};\n\nagb.then(agb => {\n\tlet canvas = document.getElementById(\"agb-canvas\");\n\n\tlet romInput = document.getElementById(\"rom\");\n\tromInput.addEventListener(\"change\", onRomUpload, false);\n\tfunction onRomUpload() {\n\t\tlet files = romInput.files;\n\t\tif(files.length > 0) {\n\t\t\tlet rom = files[0];\n\t\t\tlet fileReader = new FileReader();\n\t\t\tfileReader.onload = function() {\n\t\t\t\tlet data = fileReader.result;\n\t\t\t\tlet array = new Uint8Array(data);\n\t\t\t\tagb.load_rom(array);\n\t\t\t\tcanvas.focus();\n\t\t\t}\n\t\t\tfileReader.readAsArrayBuffer(rom);\n\t\t}\n\t}\n\n\tcanvas.addEventListener(\"keydown\", function(event) {\n\t\tlet key = keyBindings[event.key];\n\t\tif(key !== null && key !== undefined) {\n\t\t\tagb.keydown(key);\n\t\t}\n\t});\n\tcanvas.addEventListener(\"keyup\", function(event) {\n\t\tlet key = keyBindings[event.key];\n\t\tif(key !== null && key !== undefined) {\n\t\t\tagb.keyup(key);\n\t\t}\n\t});\n\n\tfunction emulateFrame() {\n\t\tlet milliseconds = Math.trunc(1000 / FRAMERATE);\n\t\tlet start = new Date().getTime();\n\t\tagb.emulate(canvas.getContext(\"2d\"), milliseconds);\n\t\tlet end = new Date().getTime();\n\t}\n\n\tsetInterval(emulateFrame, Math.trunc(1000/FRAMERATE));\n});\n\n\n//# sourceURL=webpack:///./index.js?");

/***/ })

}]);