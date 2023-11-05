import "@citation-js/plugin-bibtex";
// @ts-ignore
import Cite from "citation-js";
import { visit, EXIT, CONTINUE } from "unist-util-visit";
import { toString } from "mdast-util-to-string";
import { Maybe } from "purify-ts";


function locateBibliography(tree: any) {
  let bibliography: Maybe<Cite> = Maybe.empty();

  visit(tree, 'containerDirective', (node, index, parent) => {
    if (node.name !== 'bibtex')
      return CONTINUE;

    const data = new Cite(toString(node));
    const html = data.format("bibliography", {
      format: "html",
      type: "string",
      template: "apa",
      lang: "en-US"
    });

    parent.children.splice(index, 1, {
      type: "html",
      value: html,
      position: node.position,
    });
    bibliography = Maybe.of(data);
    return EXIT;
  });

  return bibliography;
}

function convertCitations(tree: any, data: Cite) {
  visit(tree, "textDirective", (node, index, parent) => {
    if (node.name !== "cite")
      return CONTINUE;

    const refs = new Set(toString(node).split(','));
    const cite = new Cite(data.data.filter((el: any) => refs.has(el.id)));

    const html = cite.format("citation", {
      format: "html",
      template: "apa",
    })

    parent.children.splice(index, 1, {
      type: "html",
      value: `<cite>${html}</cite>`,
      position: node.position,
    });
  })
}


export default function remarkBibliography(options?: any) {
  return (tree: any) => {
    locateBibliography(tree)
      .ifJust(data => convertCitations(tree, data));
  }
}
