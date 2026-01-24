---
title: Todo
---

1. directives

The old implementation had a robust custom directive system. The new
implementation currently relies on standard Markdown or shortcodes, but lacks
the custom handlers.

- [ ] Port sidenote container: The `<aside class="marginnote">` logic is missing.
- [ ] Port youtube block: The iframe injection logic is missing.
- [ ] Port icon inline directive: The `<img class="inline-icon">` logic is missing.
- [ ] Port cite inline directive: The old code hijacked InlineMath for
  citations; this logic is absent.

2. bibliography and citations (hayagriva)

The entire bibliography subsystem is currently missing from the comrak
implementation.

- [ ] Integrate hayagriva library: Pass the Library struct to the parser.
- [ ] Citation scanning: Implement a pass (likely AST traversal) to identify
  citation keys (e.g., [@key] or the old cite directive).
- [ ] Bibliography Rendering: Generate the HTML bibliography list and append it
  to the document.

3. outline

The old parser returned an Article struct containing the TOC.

- [ ] Extract Table of Contents

4. special code block handling

- [ ] Restore typst support: The old code checked for
  CodeBlockKind::Fenced("typst svg") and rendered it using typst::render_typst.
  The new code currently sends everything to the syntax highlighter.
