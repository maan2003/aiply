( [ (attribute_item) (line_comment) ]* .
(struct_item
    (visibility_modifier)? @context
    "struct" @context
    name: (_) @name)
) @item

( [ (attribute_item) (line_comment) ]* .
(enum_item
    (visibility_modifier)? @context
    "enum" @context
    name: (_) @name)
) @item

( [ (attribute_item) (line_comment) ]* .
(impl_item
    "impl" @context
    trait: (_)? @name
    "for"? @context
    type: (_) @name
    body: (_ "{" @open (_)* "}" @close))
) @item

( [ (attribute_item) (line_comment) ]* .
(trait_item
    (visibility_modifier)? @context
    "trait" @context
    name: (_) @name)
) @item

( [ (attribute_item) (line_comment) ]* .
(function_item
    (visibility_modifier)? @context
    (function_modifiers)? @context
    "fn" @context
    name: (_) @name)
) @item

( [ (attribute_item) (line_comment) ]* .
(function_signature_item
    (visibility_modifier)? @context
    (function_modifiers)? @context
    "fn" @context
    name: (_) @name)
) @item

( [ (attribute_item) (line_comment) ]* .
(macro_definition
    . "macro_rules!" @context
    name: (_) @name)
) @item

( [ (attribute_item) (line_comment) ]* .
(mod_item
    (visibility_modifier)? @context
    "mod" @context
    name: (_) @name)
) @item

( [ (attribute_item) (line_comment) ]* .
(type_item
    (visibility_modifier)? @context
    "type" @context
    name: (_) @name)
) @item

( [ (attribute_item) (line_comment) ]* .
(associated_type
    "type" @context
    name: (_) @name)
) @item

( [ (attribute_item) (line_comment) ]* .
(const_item
    (visibility_modifier)? @context
    "const" @context
    name: (_) @name)
) @item

( [ (attribute_item) (line_comment) ]* .
(static_item
    (visibility_modifier)? @context
    "static" @context
    name: (_) @name)
) @item
