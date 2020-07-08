(window["webpackJsonp"] = window["webpackJsonp"] || []).push([[1],{

/***/ "../pkg/earley.js":
/*!************************!*\
  !*** ../pkg/earley.js ***!
  \************************/
/*! exports provided: parse, __wbindgen_string_new, __wbindgen_rethrow */
/***/ (function(module, __webpack_exports__, __webpack_require__) {

"use strict";
eval("__webpack_require__.r(__webpack_exports__);\n/* harmony import */ var _earley_bg_wasm__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./earley_bg.wasm */ \"../pkg/earley_bg.wasm\");\n/* harmony import */ var _earley_bg_js__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! ./earley_bg.js */ \"../pkg/earley_bg.js\");\n/* harmony reexport (safe) */ __webpack_require__.d(__webpack_exports__, \"parse\", function() { return _earley_bg_js__WEBPACK_IMPORTED_MODULE_1__[\"parse\"]; });\n\n/* harmony reexport (safe) */ __webpack_require__.d(__webpack_exports__, \"__wbindgen_string_new\", function() { return _earley_bg_js__WEBPACK_IMPORTED_MODULE_1__[\"__wbindgen_string_new\"]; });\n\n/* harmony reexport (safe) */ __webpack_require__.d(__webpack_exports__, \"__wbindgen_rethrow\", function() { return _earley_bg_js__WEBPACK_IMPORTED_MODULE_1__[\"__wbindgen_rethrow\"]; });\n\n\n\n\n//# sourceURL=webpack:///../pkg/earley.js?");

/***/ }),

/***/ "../pkg/earley_bg.js":
/*!***************************!*\
  !*** ../pkg/earley_bg.js ***!
  \***************************/
/*! exports provided: parse, __wbindgen_string_new, __wbindgen_rethrow */
/***/ (function(module, __webpack_exports__, __webpack_require__) {

"use strict";
eval("__webpack_require__.r(__webpack_exports__);\n/* WEBPACK VAR INJECTION */(function(module) {/* harmony export (binding) */ __webpack_require__.d(__webpack_exports__, \"parse\", function() { return parse; });\n/* harmony export (binding) */ __webpack_require__.d(__webpack_exports__, \"__wbindgen_string_new\", function() { return __wbindgen_string_new; });\n/* harmony export (binding) */ __webpack_require__.d(__webpack_exports__, \"__wbindgen_rethrow\", function() { return __wbindgen_rethrow; });\n/* harmony import */ var _earley_bg_wasm__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! ./earley_bg.wasm */ \"../pkg/earley_bg.wasm\");\n\n\nconst lTextDecoder = typeof TextDecoder === 'undefined' ? (0, module.require)('util').TextDecoder : TextDecoder;\n\nlet cachedTextDecoder = new lTextDecoder('utf-8', { ignoreBOM: true, fatal: true });\n\ncachedTextDecoder.decode();\n\nlet cachegetUint8Memory0 = null;\nfunction getUint8Memory0() {\n    if (cachegetUint8Memory0 === null || cachegetUint8Memory0.buffer !== _earley_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"memory\"].buffer) {\n        cachegetUint8Memory0 = new Uint8Array(_earley_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"memory\"].buffer);\n    }\n    return cachegetUint8Memory0;\n}\n\nfunction getStringFromWasm0(ptr, len) {\n    return cachedTextDecoder.decode(getUint8Memory0().subarray(ptr, ptr + len));\n}\n\nconst heap = new Array(32).fill(undefined);\n\nheap.push(undefined, null, true, false);\n\nlet heap_next = heap.length;\n\nfunction addHeapObject(obj) {\n    if (heap_next === heap.length) heap.push(heap.length + 1);\n    const idx = heap_next;\n    heap_next = heap[idx];\n\n    heap[idx] = obj;\n    return idx;\n}\n\nfunction getObject(idx) { return heap[idx]; }\n\nfunction dropObject(idx) {\n    if (idx < 36) return;\n    heap[idx] = heap_next;\n    heap_next = idx;\n}\n\nfunction takeObject(idx) {\n    const ret = getObject(idx);\n    dropObject(idx);\n    return ret;\n}\n\nlet WASM_VECTOR_LEN = 0;\n\nconst lTextEncoder = typeof TextEncoder === 'undefined' ? (0, module.require)('util').TextEncoder : TextEncoder;\n\nlet cachedTextEncoder = new lTextEncoder('utf-8');\n\nconst encodeString = (typeof cachedTextEncoder.encodeInto === 'function'\n    ? function (arg, view) {\n    return cachedTextEncoder.encodeInto(arg, view);\n}\n    : function (arg, view) {\n    const buf = cachedTextEncoder.encode(arg);\n    view.set(buf);\n    return {\n        read: arg.length,\n        written: buf.length\n    };\n});\n\nfunction passStringToWasm0(arg, malloc, realloc) {\n\n    if (realloc === undefined) {\n        const buf = cachedTextEncoder.encode(arg);\n        const ptr = malloc(buf.length);\n        getUint8Memory0().subarray(ptr, ptr + buf.length).set(buf);\n        WASM_VECTOR_LEN = buf.length;\n        return ptr;\n    }\n\n    let len = arg.length;\n    let ptr = malloc(len);\n\n    const mem = getUint8Memory0();\n\n    let offset = 0;\n\n    for (; offset < len; offset++) {\n        const code = arg.charCodeAt(offset);\n        if (code > 0x7F) break;\n        mem[ptr + offset] = code;\n    }\n\n    if (offset !== len) {\n        if (offset !== 0) {\n            arg = arg.slice(offset);\n        }\n        ptr = realloc(ptr, len, len = offset + arg.length * 3);\n        const view = getUint8Memory0().subarray(ptr + offset, ptr + len);\n        const ret = encodeString(arg, view);\n\n        offset += ret.written;\n    }\n\n    WASM_VECTOR_LEN = offset;\n    return ptr;\n}\n\nlet cachegetInt32Memory0 = null;\nfunction getInt32Memory0() {\n    if (cachegetInt32Memory0 === null || cachegetInt32Memory0.buffer !== _earley_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"memory\"].buffer) {\n        cachegetInt32Memory0 = new Int32Array(_earley_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"memory\"].buffer);\n    }\n    return cachegetInt32Memory0;\n}\n/**\n* @param {string} rules\n* @param {string} input\n* @param {string} start\n* @param {string} kind\n* @param {number} n\n* @returns {string}\n*/\nfunction parse(rules, input, start, kind, n) {\n    try {\n        var ptr0 = passStringToWasm0(rules, _earley_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"__wbindgen_malloc\"], _earley_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"__wbindgen_realloc\"]);\n        var len0 = WASM_VECTOR_LEN;\n        var ptr1 = passStringToWasm0(input, _earley_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"__wbindgen_malloc\"], _earley_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"__wbindgen_realloc\"]);\n        var len1 = WASM_VECTOR_LEN;\n        var ptr2 = passStringToWasm0(start, _earley_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"__wbindgen_malloc\"], _earley_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"__wbindgen_realloc\"]);\n        var len2 = WASM_VECTOR_LEN;\n        var ptr3 = passStringToWasm0(kind, _earley_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"__wbindgen_malloc\"], _earley_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"__wbindgen_realloc\"]);\n        var len3 = WASM_VECTOR_LEN;\n        _earley_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"parse\"](8, ptr0, len0, ptr1, len1, ptr2, len2, ptr3, len3, n);\n        var r0 = getInt32Memory0()[8 / 4 + 0];\n        var r1 = getInt32Memory0()[8 / 4 + 1];\n        return getStringFromWasm0(r0, r1);\n    } finally {\n        _earley_bg_wasm__WEBPACK_IMPORTED_MODULE_0__[\"__wbindgen_free\"](r0, r1);\n    }\n}\n\nconst __wbindgen_string_new = function(arg0, arg1) {\n    var ret = getStringFromWasm0(arg0, arg1);\n    return addHeapObject(ret);\n};\n\nconst __wbindgen_rethrow = function(arg0) {\n    throw takeObject(arg0);\n};\n\n\n/* WEBPACK VAR INJECTION */}.call(this, __webpack_require__(/*! ./../www/node_modules/webpack/buildin/harmony-module.js */ \"./node_modules/webpack/buildin/harmony-module.js\")(module)))\n\n//# sourceURL=webpack:///../pkg/earley_bg.js?");

/***/ }),

/***/ "../pkg/earley_bg.wasm":
/*!*****************************!*\
  !*** ../pkg/earley_bg.wasm ***!
  \*****************************/
/*! exports provided: memory, parse, __wbindgen_malloc, __wbindgen_realloc, __wbindgen_free */
/***/ (function(module, exports, __webpack_require__) {

eval("\"use strict\";\n// Instantiate WebAssembly module\nvar wasmExports = __webpack_require__.w[module.i];\n__webpack_require__.r(exports);\n// export exports from WebAssembly module\nfor(var name in wasmExports) if(name != \"__webpack_init__\") exports[name] = wasmExports[name];\n// exec imports from WebAssembly module (for esm order)\n/* harmony import */ var m0 = __webpack_require__(/*! ./earley_bg.js */ \"../pkg/earley_bg.js\");\n\n\n// exec wasm module\nwasmExports[\"__webpack_init__\"]()\n\n//# sourceURL=webpack:///../pkg/earley_bg.wasm?");

/***/ }),

/***/ "./index.js":
/*!******************!*\
  !*** ./index.js ***!
  \******************/
/*! no exports provided */
/***/ (function(module, __webpack_exports__, __webpack_require__) {

"use strict";
eval("__webpack_require__.r(__webpack_exports__);\n/* harmony import */ var earley__WEBPACK_IMPORTED_MODULE_0__ = __webpack_require__(/*! earley */ \"../pkg/earley.js\");\n/* harmony import */ var svg_pan_zoom__WEBPACK_IMPORTED_MODULE_1__ = __webpack_require__(/*! svg-pan-zoom */ \"./node_modules/svg-pan-zoom/src/browserify.js\");\n/* harmony import */ var svg_pan_zoom__WEBPACK_IMPORTED_MODULE_1___default = /*#__PURE__*/__webpack_require__.n(svg_pan_zoom__WEBPACK_IMPORTED_MODULE_1__);\n/* harmony import */ var ace_builds__WEBPACK_IMPORTED_MODULE_2__ = __webpack_require__(/*! ace-builds */ \"./node_modules/ace-builds/src-noconflict/ace.js\");\n/* harmony import */ var ace_builds__WEBPACK_IMPORTED_MODULE_2___default = /*#__PURE__*/__webpack_require__.n(ace_builds__WEBPACK_IMPORTED_MODULE_2__);\n/* harmony import */ var split_js__WEBPACK_IMPORTED_MODULE_3__ = __webpack_require__(/*! split.js */ \"./node_modules/split.js/dist/split.es.js\");\n\n\n\n\n\nconst Viz = __webpack_require__(/*! viz.js */ \"./node_modules/viz.js/viz.js\");\n\nconst resizeEvent = new Event(\"paneresize\");\nObject(split_js__WEBPACK_IMPORTED_MODULE_3__[\"default\"])([\"#editor\", \"#graph\"], {\n  sizes: [25, 75],\n  onDragEnd: function () {\n    const svgOutput = document.getElementById(\"svg_output\");\n    if (svgOutput != null) {\n      svgOutput.dispatchEvent(resizeEvent);\n    }\n  }\n});\n\nconst editor = ace_builds__WEBPACK_IMPORTED_MODULE_2__[\"edit\"](\"editor\");\neditor.getSession().setMode(\"ace/mode/dot\");\n\nconst parser = new DOMParser();\n\nconst error = document.querySelector(\"#error\");\nconst kind = document.querySelector(\"#kind select\");\nconst input = document.querySelector(\"#input input\");\nconst start = document.querySelector(\"#start input\");\nconst ntrees = document.querySelector(\"#ntrees input\");\nconst format = document.querySelector(\"#format select\");\n\nfunction updateGraph() {\n  const output = document.querySelector(\"#output\");\n  output.classList.add(\"working\");\n  output.classList.remove(\"error\");\n  const rules = editor.getSession().getDocument().getValue();\n  const n = parseInt(ntrees.value);\n  let result;\n  try {\n    result = earley__WEBPACK_IMPORTED_MODULE_0__[\"parse\"](rules, input.value, start.value, kind.value, n);\n  } catch (msg) {\n    output.classList.add(\"error\");\n    while (error.firstChild) {\n      error.removeChild(error.firstChild);\n    }\n    error.appendChild(document.createTextNode(msg));\n  }\n  output.classList.remove(\"working\");\n\n  let svg = output.querySelector(\"svg\");\n  if (svg) output.removeChild(svg);\n  let text = output.querySelector(\"#text\");\n  if (text) output.removeChild(text);\n  let img = output.querySelector(\"img\");\n  if (img) output.removeChild(img);\n\n  if (!result) return;\n  if (kind.value === \"chart\") { // render text\n    const text = document.createElement(\"div\");\n    text.id = \"text\";\n    text.appendChild(document.createTextNode(result));\n    output.appendChild(text);\n  } else if (format.value === \"svg\") { // render svg\n    const svg = parser.parseFromString(Viz(result), \"image/svg+xml\").documentElement;\n    svg.id = \"svg_output\";\n    output.appendChild(svg);\n    const panZoom = svg_pan_zoom__WEBPACK_IMPORTED_MODULE_1__(svg, {\n      zoomEnabled: true,\n      controlIconsEnabled: true,\n      fit: true,\n      center: true,\n      minZoom: 0.1\n    });\n    svg.addEventListener(\"paneresize\", _ => panZoom.resize(), false);\n    window.addEventListener(\"resize\", _ => panZoom.resize());\n  } else { // render png\n    output.appendChild(Viz.svgXmlToPngImageElement(Viz(result)));\n  }\n}\n\neditor.on(\"change\", () => updateGraph());\nkind.addEventListener(\"change\", () => updateGraph());\ninput.addEventListener(\"input\", () => updateGraph());\nstart.addEventListener(\"input\", () => updateGraph());\nntrees.addEventListener(\"input\", () => updateGraph());\nformat.addEventListener(\"change\", () => updateGraph());\n\nupdateGraph();\n\n//# sourceURL=webpack:///./index.js?");

/***/ }),

/***/ 0:
/*!********************!*\
  !*** fs (ignored) ***!
  \********************/
/*! no static exports found */
/***/ (function(module, exports) {

eval("/* (ignored) */\n\n//# sourceURL=webpack:///fs_(ignored)?");

/***/ }),

/***/ 1:
/*!**********************!*\
  !*** path (ignored) ***!
  \**********************/
/*! no static exports found */
/***/ (function(module, exports) {

eval("/* (ignored) */\n\n//# sourceURL=webpack:///path_(ignored)?");

/***/ }),

/***/ 2:
/*!************************!*\
  !*** crypto (ignored) ***!
  \************************/
/*! no static exports found */
/***/ (function(module, exports) {

eval("/* (ignored) */\n\n//# sourceURL=webpack:///crypto_(ignored)?");

/***/ })

}]);