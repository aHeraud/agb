console.log("Rustboy init");

var Gameboy = {
	lib: {
		init: Module.cwrap("rustboy_init", null, ["number", "number", "number", "number"]),
		step_frame: Module.cwrap("rustboy_step_frame", null, []),
		get_framebuffer: Module.cwrap("rustboy_get_framebuffer", "number", []),
		keydown: Module.cwrap("rustboy_keydown", null, ["number"]),
		keyup: Module.cwrap("rustboy_keyup", null, ["number"]),
	},

	//TODO: export constants from emscripten
	keys: {
		up: 0,
		down: 1,
		left: 2,
		right: 3,
		b: 4,
		a: 5,
		select: 6,
		start: 7,
	},

	callbacks: {
		keydown: function(key) {
			let code = Gameboy.getKey(key);
			if(code != -1) {
				Gameboy.lib.keydown(code);
			}
		},

		keyup: function(key) {
			let code = Gameboy.getKey(key);
			if(code != -1) {
				Gameboy.lib.keyup(code);
			}
		},
	},

	rom_ptr: null,
	rom_size: -1,
	ram_ptr: null,
	ram_size: -1,

	//TODO: allow remapping of keys
	getKey: function(key) {
		switch(key) {
			case "ArrowUp":
				return Gameboy.keys.up;
			case "ArrowDown":
				return Gameboy.keys.down;
			case "ArrowLeft":
				return Gameboy.keys.left;
			case "ArrowRight":
				return Gameboy.keys.right;
			case "z":
				return Gameboy.keys.b;
			case "x":
				return Gameboy.keys.a;
			case "c":
				return Gameboy.keys.select;
			case "v":
				return Gameboy.keys.start;
			default:
				return undefined;
		}
	},

	reset: function() {
		if(Gameboy.rom_ptr === null) {
			console.error("Gameboy needs a rom to run.");
			return;
		}

		if(Gameboy.ram_ptr === null) {
			//Generate empty ram
			//Eventually i'll rewrite the Gameboy constructor to accept a null pointer and autogenerate it's own ram
			const size = 4096;
			let buffer = Module._malloc(size);
			Gameboy.ram_ptr = buffer;
			Gameboy.ram_size = size;
		}

		/* rustboy_init(rom_ptr, rom_size, ram_ptr, ram_size) */
		Gameboy.lib.init(Gameboy.rom_ptr, Gameboy.rom_size, Gameboy.ram_ptr, Gameboy.ram_size);

		Gameboy.start();
	},

	draw_frame: function() {
		let framebuffer_ptr = Gameboy.lib.get_framebuffer();
		if(framebuffer_ptr === null) {
			throw "Null framebuffer";
		}
		let heap = Module.HEAP8.buffer;
		//let frame = new Uint8ClampedArray(heap, framebuffer_ptr, (160 * 144 * 4));

		//The frame is packed into u32 rgba pixel values, and as far as I can tell a canvas expects
		//rgba ordered bytes, so repack into an array of u8's
		let frame_u32 = new Uint32Array(heap, framebuffer_ptr, (160 * 144 * 4));
		let frame = new Uint8ClampedArray(160 * 144 * 4);
		for(var i = 0; i < (160 * 144); i += 1) {
			let r = (frame_u32[i] >> 24) & 255;
			let g = (frame_u32[i] >> 16) & 255;
			let b = (frame_u32[i] >> 8) & 255;
			let a = 255;

			frame[i * 4] = r;
			frame[(i * 4) + 1] = g;
			frame[(i * 4) + 2] = b;
			frame[(i * 4) + 3] = a;
		}

		let canvas = document.getElementById("screen");
		let ctx = canvas.getContext("2d");
		ctx.imagesmoothingenabled = false;

		let imagedata = new ImageData(frame, 160, 144);
		ctx.putImageData(imagedata, 0, 0);
	},

	step: function() {
		Gameboy.lib.step_frame();
		Gameboy.draw_frame();
	},

	start: function() {
		Gameboy.stop();
		Gameboy._intervalId = setInterval(Gameboy.step, 1000/60);
	},

	stop: function() {
		clearInterval(Gameboy._intervalId);
	},
}

//Move me
document.getElementById("reset").addEventListener("click", Gameboy.reset);
document.body.addEventListener("keydown", function(event) {
	Gameboy.callbacks.keydown(event.key);
});
document.body.addEventListener("keyup", function(event) {
	Gameboy.callbacks.keyup(event.key);
});
