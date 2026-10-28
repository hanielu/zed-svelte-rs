(comment) @annotation

(element
  (start_tag) @name) @item

(element
  (self_closing_tag) @name) @item

(if_block
  "if" @context
  expression: (_)? @name) @item

(else_if_clause
  (branch_kind) @context
  expression: (_) @name) @item

(else_clause
  "else" @name) @item

(each_block
  "each" @context
  expression: (_)? @name) @item

(await_block
  "await" @context
  expression: (_)? @name) @item

(await_branch
  (branch_kind) @context
  (pattern)? @name) @item

(key_block
  "key" @context
  expression: (_)? @name) @item

(snippet_block
  "snippet" @context
  name: (snippet_name) @name) @item

(render_tag
  "render" @context
  expression: (_) @name) @item

(html_tag
  "html" @context
  expression: (_) @name) @item

(const_tag
  "const" @context
  expression: (_) @name) @item
