const agb = import('./agb_web');

const FRAMERATE = 59.7;
let canvas = document.getElementById("agb-canvas");
const KEY_UP = 0;
const KEY_DOWN = 1;
const KEY_LEFT = 2;
const KEY_RIGHT = 3;
const KEY_B = 4;
const KEY_A = 5;
const KEY_SELECT = 6;
const KEY_START = 7;

let keyBindings = {
	"ArrowUp": KEY_UP,
	"ArrowDown": KEY_DOWN,
	"ArrowLeft": KEY_LEFT,
	"ArrowRight": KEY_RIGHT,
	"z": KEY_B,
	"x": KEY_A,
	"c": KEY_SELECT,
	"v": KEY_START
};

agb.then(agb => {
	let romInput = document.getElementById("rom");
	romInput.addEventListener("change", onRomUpload, false);
	function onRomUpload() {
		let files = romInput.files;
		if(files.length > 0) {
			let rom = files[0];
			let fileReader = new FileReader();
			fileReader.onload = function() {
				let data = fileReader.result;
				let array = new Uint8Array(data);
				agb.load_rom(array);
			}
			fileReader.readAsArrayBuffer(rom);
		}
	}

	let canvasContainer = document.getElementById("agb-canvas-container");
	canvasContainer.addEventListener("keydown", function(event) {
		let key = keyBindings[event.key];
		if(key !== null && key !== undefined) {
			agb.keydown(key);
		}
	});
	canvasContainer.addEventListener("keyup", function(event) {
		let key = keyBindings[event.key];
		if(key !== null && key !== undefined) {
			agb.keyup(key);
		}
	});

	function emulateFrame() {
		let milliseconds = Math.trunc(1000 / FRAMERATE);
		let start = new Date().getTime();
		agb.emulate(milliseconds);
		let end = new Date().getTime();
	}

	setInterval(emulateFrame, Math.trunc(1000/FRAMERATE));
});

let imageData = null;
let imageBuffer = null;
let imageBufferU8 = null;
let imageBufferU32 = null;

/**
* draw pixels to the canvas
* assumes that pixels are in RGBA format
* @param {Number} width
* @param {Number} height
* @param {Uint32Array} pixels
*/
export function draw(width, height, pixels) {
	let ctx = canvas.getContext("2d");
	if(imageData === null || imageData.width != width || imageData.height != height) {
		imageData = ctx.createImageData(width, height);
		imageBuffer = new ArrayBuffer(imageData.data.length);
		imageBufferU8 = new Uint8ClampedArray(imageBuffer);
		imageBufferU32 = new Uint32Array(imageBuffer);
	}

	for(let i = 0; i < Math.min(imageBufferU32.length, pixels.length); i += 1) {
		//convert to ARGB
		imageBufferU32[i] = (pixels[i] >> 8) | 0xFF000000;
	}

	imageData.data.set(imageBufferU8);
	ctx.putImageData(imageData, 0, 0);
}
