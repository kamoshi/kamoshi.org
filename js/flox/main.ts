import { EditorState } from "@codemirror/state";
import { EditorView, keymap, lineNumbers } from "@codemirror/view";
import { defaultKeymap } from "@codemirror/commands";
import { evaluate } from "./wasm";

const doc = `
let n1 = 2;
let n2 = 5;

let add a b = a + b;

2
  |> add 2
  |> add 1
  |> fn n -> n * -1
`;

const htmlEditor = document.getElementById("editor")!;
const htmlOutput = document.getElementById("output")!;
const htmlRun = document.getElementById("run")!;

const state = EditorState.create({
  doc,
  extensions: [keymap.of(defaultKeymap), lineNumbers()],
});

const view = new EditorView({ state, parent: htmlEditor });

htmlRun.addEventListener("click", () => {
  htmlOutput.textContent = evaluate(view.state.doc.toString());
});
