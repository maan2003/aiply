([ (comment) ]* .
(internal_module
    "namespace" @context
    name: (_) @name)) @item

([ (comment) ]* .
(enum_declaration
    "enum" @context
    name: (_) @name)) @item

([ (comment) ]* .
(type_alias_declaration
    "type" @context
    name: (_) @name)) @item

([ (comment) ]* .
(export_statement
    "export" @context
    .
    (type_alias_declaration
        "type" @context
        name: (_) @name))) @item
([ (comment) ]* .
(function_declaration
    "async"? @context
    "function" @context
    name: (_) @name
    parameters: (formal_parameters
      "(" @context
      ")" @context))) @item

([ (comment) ]* .
(interface_declaration
    "interface" @context
    name: (_) @name)) @item

([ (comment) ]* .
(export_statement
    "export" @context
    .
    (lexical_declaration
        ["let" "const"] @context
        (variable_declarator
            name: (_) @name)))) @item

([ (comment) ]* .
(program
    (lexical_declaration
        ["let" "const"] @context
        (variable_declarator
            name: (_) @name)))) @item

([ (comment) ]* .
(class_declaration
    "class" @context
    name: (_) @name)) @item

([ (comment) ]* .
(method_definition
    [
        "get"
        "set"
        "async"
        "*"
        "readonly"
        "static"
        (override_modifier)
        (accessibility_modifier)
    ]* @context
    name: (_) @name
    parameters: (formal_parameters
      "(" @context
      ")" @context))) @item

([ (comment) ]* .
(public_field_definition
    [
        "declare"
        "readonly"
        "abstract"
        "static"
        (accessibility_modifier)
    ]* @context
    name: (_) @name)) @item

([ (comment) ]* .
(call_expression
    function: (_) @context
    (#any-of? @context "it" "test" "describe")
    arguments: (
        arguments . (string
            (string_fragment) @name
        )
    ))) @item
