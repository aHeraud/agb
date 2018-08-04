/* tslint:disable */
import * as wasm from './agb_web_bg';
import { draw } from './index';

const __wbg_log_b6b99bda6f2d2a73_target = console.log;

const TextDecoder = typeof self === 'object' && self.TextDecoder
    ? self.TextDecoder
    : require('util').TextDecoder;

let cachedDecoder = new TextDecoder('utf-8');

let cachegetUint8Memory = null;
function getUint8Memory() {
    if (cachegetUint8Memory === null || cachegetUint8Memory.buffer !== wasm.memory.buffer) {
        cachegetUint8Memory = new Uint8Array(wasm.memory.buffer);
    }
    return cachegetUint8Memory;
}

function getStringFromWasm(ptr, len) {
    return cachedDecoder.decode(getUint8Memory().subarray(ptr, ptr + len));
}

export function __wbg_log_b6b99bda6f2d2a73(arg0, arg1) {
    let varg0 = getStringFromWasm(arg0, arg1);
    __wbg_log_b6b99bda6f2d2a73_target(varg0);
}

const __wbg_error_c027e244c353f04e_target = console.error;

export function __wbg_error_c027e244c353f04e(arg0, arg1) {
    let varg0 = getStringFromWasm(arg0, arg1);
    __wbg_error_c027e244c353f04e_target(varg0);
}

export function __wbg_alert_8dc787c1a93118ac(arg0, arg1) {
    let varg0 = getStringFromWasm(arg0, arg1);
    alert(varg0);
}

let cachegetUint32Memory = null;
function getUint32Memory() {
    if (cachegetUint32Memory === null || cachegetUint32Memory.buffer !== wasm.memory.buffer) {
        cachegetUint32Memory = new Uint32Array(wasm.memory.buffer);
    }
    return cachegetUint32Memory;
}

function getArrayU32FromWasm(ptr, len) {
    return getUint32Memory().subarray(ptr / 4, ptr / 4 + len);
}

export function __wbg_draw_8067c5a3a0ae46f3(arg0, arg1, arg2, arg3) {
    let varg2 = getArrayU32FromWasm(arg2, arg3);
    draw(arg0, arg1, varg2);
}

function passArray8ToWasm(arg) {
    const ptr = wasm.__wbindgen_malloc(arg.length * 1);
    getUint8Memory().set(arg, ptr / 1);
    return [ptr, arg.length];
}
/**
* Loads a rom + an optional save file.
* This creates a new Gameboy object.
* This can fail: if the rom has an invalid header an alert will be displayed  and an error message will be printed to the console
* @param {Uint8Array} arg0
* @returns {void}
*/
export function load_rom(arg0) {
    const [ptr0, len0] = passArray8ToWasm(arg0);
    try {
        return wasm.load_rom(ptr0, len0);
        
    } finally {
        wasm.__wbindgen_free(ptr0, len0 * 1);
        
    }
    
}

/**
* @param {number} arg0
* @returns {void}
*/
export function keydown(arg0) {
    return wasm.keydown(arg0);
}

/**
* @param {number} arg0
* @returns {void}
*/
export function keyup(arg0) {
    return wasm.keyup(arg0);
}

/**
* Emulate the gameboy for a specific number of milliseconds
* @param {number} arg0
* @returns {void}
*/
export function emulate(arg0) {
    return wasm.emulate(arg0);
}

export function __wbindgen_throw(ptr, len) {
    throw new Error(getStringFromWasm(ptr, len));
}

