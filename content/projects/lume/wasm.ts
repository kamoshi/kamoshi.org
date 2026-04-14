import {
  initSync,
  lint as _lint,
  to_js as _toJs,
  to_lua as _toLua,
  type_at as _typeAt,
} from './pkg/lume_wasm';

export let wasmReady = false;
export const wasmCallbacks: (() => void)[] = [];

WebAssembly.compileStreaming(fetch('/static/wasm/lume.wasm')).then((mod) => {
  initSync({ module: mod });
  wasmReady = true;
  wasmCallbacks.splice(0).forEach((cb) => cb());
});

export const wasmLint   = _lint;
export const wasmTypeAt = _typeAt;
export const wasmToJs   = _toJs;
export const wasmToLua  = _toLua;
