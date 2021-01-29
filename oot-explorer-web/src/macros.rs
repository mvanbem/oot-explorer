macro_rules! html_template {
    (
        @parse $_:tt
        /* empty */
    ) => {};

    // Parse a bound element.
    (
        @parse {
            document: ($document:expr)
            parent: $parent:tt
        }
        let $binding:ident = $tag:ident[$($attr_name:ident = $attr_value:literal)*] {
            $($body:tt)*
        }
        $($tail:tt)*
    ) => {
        let $binding: ::web_sys::HtmlElement = ::wasm_bindgen::JsCast::unchecked_into(
            ::wasm_bindgen::UnwrapThrowExt::unwrap_throw(
                ::web_sys::Document::create_element(&$document, stringify!($tag)),
            ),
        );
        $(html_template! { @attr binding: $binding name: $attr_name value: $attr_value })*
        html_template! {
            @parse {
                document: ($document)
                parent: (Some($binding))
            }
            $($body)*
        }
        html_template! { @append_child parent: $parent child: $binding }

        html_template! { @parse { document: ($document) parent: $parent } $($tail)* }
    };

    // Parse an unbound element.
    (
        @parse {
            document: ($document:expr)
            parent: $parent:tt
        }
        $tag:ident[$($attr_name:ident = $attr_value:literal)*] {
            $($body:tt)*
        }
        $($tail:tt)*
    ) => {
        let element: ::web_sys::HtmlElement = ::wasm_bindgen::JsCast::unchecked_into(
            ::wasm_bindgen::UnwrapThrowExt::unwrap_throw(
                ::web_sys::Document::create_element(&$document, stringify!($tag)),
            ),
        );
        $(html_template! { @attr binding: element name: $attr_name value: $attr_value })*
        html_template! {
            @parse {
                document: ($document)
                parent: (Some(element))
            }
            $($body)*
        }
        html_template! { @append_child parent: $parent child: element }

        html_template! { @parse { document: ($document) parent: $parent } $($tail)* }
    };

    // Parse a text node.
    (
        @parse {
            document: ($document:expr)
            parent: $parent:tt
        }
        text($text:expr)
        $($tail:tt)*
    ) => {
        let text_node = ::web_sys::Document::create_text_node($document, $text);
        html_template! { @append_child parent: $parent child: text_node }

        html_template! { @parse { document: ($document) parent: $parent } $($tail)* }
    };

    // Set an attribute or pseudo-attribute.
    (@attr binding: $binding:ident name: class value: $value:literal) => {
        ::web_sys::Element::set_class_name(&$binding, $value);
    };
    (@attr binding: $binding:ident name: $name:ident value: $value:literal) => {
        ::web_sys::Element::set_attribute(&$binding, stringify!($name), $value);
    };

    // Append a node to a parent node, if given.
    (@append_child parent: (None) child: $_:ident) => {};
    (@append_child parent: (Some($parent:ident)) child: $child:ident) => {
        ::wasm_bindgen::UnwrapThrowExt::unwrap_throw(
            ::web_sys::Node::append_child(&$parent, &$child),
        );
    };

    // Catch-all handler for input that didn't match anything.
    (@ $($args:tt)*) => {
        compile_error!(concat!(
            "a match fell through: @",
            $(stringify!($args), " ",)*
        ));
    };

    // Handle the initial invocation when requested to yield a value.
    ($document:expr, return $($args:tt)*) => {
        {
            html_template! { @parse { document: ($document) parent: (None) } let temp = $($args)* }
            temp
        }
    };

    // Handle the initial invocation when requested to append to an external parent.
    ($document:expr, in $parent:ident: $($args:tt)*) => {
        html_template! { @parse { document: ($document) parent: (Some($parent)) } $($args)* }
    };

    // Handle the initial invocation when not requested to yield a value.
    ($document:expr, $($args:tt)*) => {
        html_template! { @parse { document: ($document) parent: (None) } $($args)* }
    };
}
