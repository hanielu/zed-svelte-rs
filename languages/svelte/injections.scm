((comment) @injection.content
  (#set! injection.language "comment"))

; <script lang="ts">, <style lang="scss">: the lang attribute names the language
(element
  (start_tag
    (tag_name) @_tag
    (attribute
      (attribute_name) @_attr
      (quoted_attribute_value
        (attribute_value) @injection.language)))
  (raw_text) @injection.content
  (#any-of? @_tag "script" "style")
  (#eq? @_attr "lang"))

; <script> without a lang attribute
(element
  (start_tag
    (tag_name) @_tag
    (attribute
      (attribute_name) @_attr)*)
  (raw_text) @injection.content
  (#eq? @_tag "script")
  (#not-any-of? @_attr "lang")
  (#set! injection.language "javascript"))

; <style> without a lang attribute
(element
  (start_tag
    (tag_name) @_tag
    (attribute
      (attribute_name) @_attr)*)
  (raw_text) @injection.content
  (#eq? @_tag "style")
  (#not-any-of? @_attr "lang")
  (#set! injection.language "css"))

; Template expressions, patterns and tag arguments: the grammar makes the
; content of each one a js or ts node, from the instance script's lang.
((ts) @injection.content
  (#set! injection.language "typescript"))

((js) @injection.content
  (#set! injection.language "javascript"))

; {#snippet name<T>(...)}
((snippet_type_parameters) @injection.content
  (#set! injection.language "typescript"))

; style="color: red"
(attribute
  (attribute_name) @_attr
  (quoted_attribute_value
    (attribute_value) @injection.content)
  (#eq? @_attr "style")
  (#set! injection.language "css"))
