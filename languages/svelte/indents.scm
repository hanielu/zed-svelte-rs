; Element children, between the start and end tags
(element
  (start_tag) @start
  [
    (end_tag)
    (erroneous_end_tag)
  ]? @end) @indent

; Attributes of a start tag that spans several lines
(start_tag
  ">" @end) @indent

(self_closing_tag
  "/>" @end) @indent

; Block bodies, from the `}` of the opening tag to {/...}
(if_block
  (block_close) @start
  (block_end)? @end) @indent

(each_block
  (block_close) @start
  (block_end)? @end) @indent

(await_block
  (block_close) @start
  (block_end)? @end) @indent

(key_block
  (block_close) @start
  (block_end)? @end) @indent

(snippet_block
  (block_close) @start
  (block_end)? @end) @indent

; {:else}, {:else if}, {:then}, {:catch} line up with the opening tag
(if_block
  [
    (else_if_clause)
    (else_clause)
  ] @outdent)

(each_block
  (else_clause) @outdent)

(await_block
  (await_branch) @outdent)

(else_if_clause
  (block_close) @start) @indent

(else_clause
  (block_close) @start) @indent

(await_branch
  branch: (block_close) @start) @indent
