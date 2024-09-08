import * as wasm from "./pkg/flox_wasm";

const PATH = "/static/flox_wasm_bg.wasm";
const INIT = WebAssembly.compileStreaming(fetch(PATH)).then(wasm.initSync);

export const init = INIT;

export function evaluate(source: string): string {
  return wasm.eval_expression(source);
}
