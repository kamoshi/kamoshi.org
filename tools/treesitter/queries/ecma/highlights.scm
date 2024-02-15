; Keywords
;----------
[
  "if"
  "else"
  "switch"
  "case"
] @keyword.conditional

[
  "import"
  "from"
] @keyword.import

(export_specifier
  "as" @keyword.import)

(import_specifier
  "as" @keyword.import)

(namespace_export
  "as" @keyword.import)

(namespace_import
  "as" @keyword.import)

[
  "for"
  "of"
  "do"
  "while"
  "continue"
] @keyword.repeat

[
  "break"
  "class"
  "const"
  "debugger"
  "export"
  "extends"
  "get"
  "let"
  "set"
  "static"
  "target"
  "var"
  "with"
] @keyword

[
  "async"
  "await"
] @keyword.coroutine

[
  "return"
  "yield"
] @keyword.return

"function" @keyword.function

[
  "new"
  "delete"
  "in"
  "instanceof"
  "typeof"
] @keyword.operator

[
  "throw"
  "try"
  "catch"
  "finally"
] @keyword.exception

(export_statement
  "default" @keyword)

(switch_default
  "default" @keyword.conditional)
