( [ (attribute_item)+ @item (line_comment)+ @item ]* .
(struct_item
    (visibility_modifier)? @context
    "struct" @context
    name: (_) @name)
) @item

( [ (attribute_item)+ @item (line_comment)+ @item ]* .
(enum_item
    (visibility_modifier)? @context
    "enum" @context
    name: (_) @name) @item
) @item

( [ (attribute_item)+ @item (line_comment)+ @item ]* .
(impl_item
    "impl" @context
    trait: (_)? @name
    "for"? @context
    type: (_) @name
    body: (_ "{" @open (_)* "}" @close))
) @item
( [ (attribute_item)+ @item (line_comment)+ @item ]* .
(trait_item
    (visibility_modifier)? @context
    "trait" @context
    name: (_) @name)
) @item

( [ (attribute_item)+ @item (line_comment)+ @item ]* .
(function_item
    (visibility_modifier)? @context
    (function_modifiers)? @context
    "fn" @context
    name: (_) @name)
) @item

( [ (attribute_item)+ @item (line_comment)+ @item ]* .
(function_signature_item
    (visibility_modifier)? @context
    (function_modifiers)? @context
    "fn" @context
    name: (_) @name)
) @item

( [ (attribute_item)+ @item (line_comment)+ @item ]* .
(macro_definition
    . "macro_rules!" @context
    name: (_) @name)
) @item

( [ (attribute_item)+ @item (line_comment)+ @item ]* .
(mod_item
    (visibility_modifier)? @context
    "mod" @context
    name: (_) @name)
) @item

( [ (attribute_item)+ @item (line_comment)+ @item ]* .
(type_item
    (visibility_modifier)? @context
    "type" @context
    name: (_) @name)
) @item

( [ (attribute_item)+ @item (line_comment)+ @item ]* .
(associated_type
    "type" @context
    name: (_) @name)
) @item

( [ (attribute_item)+ @item (line_comment)+ @item ]* .
(const_item
    (visibility_modifier)? @context
    "const" @context
    name: (_) @name)
) @item

( [ (attribute_item)+ @item (line_comment)+ @item ]* .
(static_item
    (visibility_modifier)? @context
    "static" @context
    name: (_) @name)
) @item
