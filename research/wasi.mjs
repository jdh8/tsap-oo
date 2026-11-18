// Run a wasm32-wasip1 program under node's WASI, i.e. under V8, the wasm
// engine of Chrome: `node --no-warnings research/wasi.mjs prog.wasm [args]`.
import { readFile } from 'node:fs/promises';
import { WASI } from 'node:wasi';

const [path, ...args] = process.argv.slice(2);
const wasi = new WASI({ version: 'preview1', args: [path, ...args] });
const module = await WebAssembly.compile(await readFile(path));
wasi.start(await WebAssembly.instantiate(module, wasi.getImportObject()));
