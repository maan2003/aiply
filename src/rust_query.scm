(function_item name: (identifier) @function)
(impl_item type: (type_identifier) @impl)
(struct_item name: (type_identifier) @struct)
(trait_item name: (type_identifier) @trait)
(impl_item
    type: (type_identifier) @impl_container
    body: (declaration_list
        (function_item name: (identifier) @impl_method)))
(trait_item
    name: (type_identifier) @trait_container
    body: (declaration_list
        (function_item name: (identifier) @trait_method)))
