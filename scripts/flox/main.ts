import { EditorState } from "@codemirror/state";
import { EditorView, keymap, lineNumbers } from "@codemirror/view";
import { defaultKeymap } from "@codemirror/commands";
import { init, evaluate } from "./wasm";

function instantiate(element: HTMLElement): EditorView {
  const divEditor = document.createElement("div");
  const divOutput = document.createElement("pre");
  const initState = element.innerText.trim();

  element.innerText = "";
  element.appendChild(divEditor);
  element.appendChild(divOutput);
  divEditor.className = "editor";
  divOutput.className = "output";
  init.then((_) => (divOutput.innerText = evaluate(initState)));

  const state = EditorState.create({
    doc: initState,
    extensions: [
      keymap.of(defaultKeymap),
      lineNumbers(),
      EditorView.updateListener.of((update) => {
        if (update.docChanged) {
          divOutput.innerText = evaluate(update.state.doc.toString());
        }
      }),
    ],
  });

  return new EditorView({ state, parent: divEditor });
}

for (const el of document.getElementsByClassName("flox-eval")) {
  const editor = instantiate(el as HTMLElement);
}
