; Tag delimiters and quotes match as pairs but stay out of rainbow bracket
; colorization, as in Zed's HTML extension.
(("<" @open ">" @close) (#set! rainbow.exclude))
(("<" @open "/>" @close) (#set! rainbow.exclude))
(("</" @open ">" @close) (#set! rainbow.exclude))
(("\"" @open "\"" @close) (#set! rainbow.exclude))
(("'" @open "'" @close) (#set! rainbow.exclude))

; {expression}, {...spread}, snippet parameters and each keys
("{" @open "}" @close)
("(" @open ")" @close)

; {#if ...}, {:else}, {/if}, {@render ...}: each of these nodes has exactly one
; block_open and one closing `}` of its own. Only the brace of the block_open is
; a bracket; its sigil (`#`, `:`, `@`, `/`) is highlighted with the keyword.
(_
  (block_open
    "{" @open)
  (block_close) @close)

((element
  (start_tag) @open
  [
    (end_tag)
    (erroneous_end_tag)
  ] @close)
  (#set! newline.only)
  (#set! rainbow.exclude))
