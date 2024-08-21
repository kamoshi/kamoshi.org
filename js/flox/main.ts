import { EditorState } from '@codemirror/state';
import { EditorView, keymap, lineNumbers } from '@codemirror/view';
import { defaultKeymap } from '@codemirror/commands';
import * as wasm from './pkg/flox_wasm';

const doc = `
let n1 = 2;
let n2 = 5;

let add a b = a + b;

2
  |> add 2
  |> add 1
  |> fn n -> n * -1
`;

const htmlEditor = document.getElementById('editor')!;
const htmlOutput = document.getElementById('output')!;
const htmlRun = document.getElementById('run')!;

const state = EditorState.create({
  doc,
  extensions: [
    keymap.of(defaultKeymap),
    lineNumbers()
  ],
});

const view = new EditorView({ state, parent: htmlEditor });

WebAssembly.compileStreaming(fetch('flox_wasm_bg.wasm')).then((asm) => wasm.initSync(asm));

function run(code: string) {
  return wasm.eval_expression(code);
}

htmlRun.addEventListener('click', () => {
  const code = view.state.doc.toString();
  htmlOutput.textContent = run(code);
});
