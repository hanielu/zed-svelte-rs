; Zed gives later patterns precedence: general captures first, specific ones after.

; Markup

(tag_name) @tag

(erroneous_end_tag_name) @tag

(doctype) @tag.doctype

(comment) @comment

(tag_comment) @comment

(entity) @string.special

; Components, including member components such as <Chart.Tooltip />
((tag_name) @type
  (#match? @type "^[A-Z]"))

(tag_name
  object: (tag_member) @type)

(tag_name
  property: (tag_member) @type)

; Attributes. Directive scopes follow the Svelte VS Code grammar.

(attribute_name) @attribute

; transition:, in:, out:, animate:, let:
(attribute_directive) @keyword

; on:, use:, bind:
((attribute_directive) @keyword.control
  (#any-of? @keyword.control "on" "use" "bind"))

; class:name and style:name are names, like attributes
((attribute_directive) @attribute
  (#any-of? @attribute "class" "style"))

; use:action, transition:fade, in:fly, out:fade, animate:flip
(attribute_name
  (attribute_directive) @_directive
  (attribute_identifier) @function
  (#any-of? @_directive "use" "transition" "in" "out" "animate"))

; on:click
(attribute_name
  (attribute_directive) @_directive
  (attribute_identifier) @type
  (#eq? @_directive "on"))

; bind:this
(attribute_name
  (attribute_directive) @_directive
  (attribute_identifier) @variable.special
  (#eq? @_directive "bind")
  (#eq? @variable.special "this"))

; in:fly|global, on:click|once
(attribute_modifier) @function

[
  "\""
  "'"
  (attribute_value)
] @string

; Svelte blocks and tags: {#if}, {:else}, {/if}, {@render}, ...
; Block keywords are control flow, tag keywords are plain keywords.

[
  "html"
  "debug"
  "render"
  "attach"
  "const"
  (block_sigil)
] @keyword

(declaration_kind) @keyword.declaration

[
  "if"
  "each"
  "await"
  "key"
  "snippet"
  "else"
  "as"
  (block_keyword)
  (branch_kind)
  (shorthand_kind)
] @keyword.control

(snippet_name) @function

; Punctuation

[
  "<"
  ">"
  "</"
  "/>"
  "<!"
] @punctuation.bracket

[
  "{"
  "}"
  "("
  ")"
  (block_close)
] @punctuation.bracket

[
  "="
  ":"
  "|"
  "."
  ","
] @punctuation.delimiter

; The sigil of {#if}, {:else}, {@render}, {/if}
(block_open
  [
    "#"
    ":"
    "@"
    "/"
  ] @punctuation.delimiter)

; Namespaced tags read as one tag name
(tag_name
  ":" @tag)

; svelte:head, svelte:element: the svelte namespace is a keyword
(tag_name
  namespace: (tag_namespace) @keyword.control
  ":" @punctuation.delimiter
  (#eq? @keyword.control "svelte"))
