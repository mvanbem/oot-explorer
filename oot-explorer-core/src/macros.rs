macro_rules! compile_interfaces {
    // Parse the end of input.
    (@parse: Init $ignored_state:tt) => {};

    // Parse a size attribute.
    (
        @parse: Init { size: $ignored_size:tt }
        #[size($size:literal)]
        $($tail:tt)*
    ) => {
        compile_interfaces! {
            // Record the newly defined size.
            @parse: Init { size: (present $size) }
            $($tail)*
        }
    };

    // Parse the start of a struct.
    (
        @parse: Init { size: $size:tt }
        struct $name:ident {
            $($struct_tail:tt)*
        }
        $($tail:tt)*
    ) => {
        compile_interfaces! {
            // We are now parsing a struct.
            @parse: Struct { name: $name size: $size fields: [] }
            { $($struct_tail)* }
            $($tail)*
        }
    };

    // Parse a struct field declaration.
    (
        @parse: Struct { name: $name:ident size: $size:tt fields: [$($field:tt)*] }
        {
            $field_type:ident $field_name:ident @ $field_offset:literal;
            $($struct_tail:tt)*
        }
        $($tail:tt)*
    ) => {
        compile_interfaces! {
            @parse: Struct {
                name: $name
                size: $size
                fields: [
                    $($field)*
                    // New field.
                    { name: $field_name offset: $field_offset type: $field_type }
                ]
            }
            { $($struct_tail)* }
            $($tail)*
        }
    };

    // Parse the end of a struct.
    (
        @parse: Struct {
            name: $name:ident
            size: $size:tt
            fields: [$($field:tt)*]
        }
        { /* empty */ }
        $($tail:tt)*
    ) => {
        // Generate the reflection table.
        ::paste::paste! {
            pub const [<$name:snake:upper _DESC>]: crate::reflect::type_::TypeDescriptor =
                crate::reflect::type_::TypeDescriptor::Struct(
                    &crate::reflect::struct_::StructDescriptor {
                        name: "",
                        size: compile_interfaces!(@option_to_rust $size),
                        is_end: None,
                        fields: &[$(
                            compile_interfaces!(@emit_field_descriptor $field),
                        )*],
                    },
                );
        }

        // Generate the reader type.

        #[derive(Clone, Copy)]
        pub struct $name<'scope> {
            data: &'scope [u8],
        }

        impl<'scope> $name<'scope> {
            pub fn new(data: &'scope [u8]) -> Self {
                Self { data }
            }

            pub fn data(self) -> &'scope [u8] {
                self.data
            }

            $(
                compile_interfaces!(@emit_field_accessor $field);
            )*
        }

        compile_interfaces!(@emit_struct_reader_impl $name $size);

        compile_interfaces! {
            // We are back in the initial state.
            @parse: Init { size: (absent) }
            $($tail)*
        }
    };

    // Emit a Rust `FieldDescriptor` literal.
    (@emit_field_descriptor { name: $name:ident offset: $offset:literal type: u16 }) => {
        crate::reflect::struct_::FieldDescriptor {
            name: stringify!($name),
            location: crate::reflect::struct_::StructFieldLocation::Simple { offset: $offset },
            desc: crate::reflect::type_::TypeDescriptor::Primitive(
                crate::reflect::primitive::PrimitiveType::U16,
            ),
        }
    };
    (@emit_field_descriptor { name: $name:ident offset: $offset:literal type: i16 }) => {
        crate::reflect::struct_::FieldDescriptor {
            name: stringify!($name),
            location: crate::reflect::struct_::StructFieldLocation::Simple { offset: $offset },
            desc: crate::reflect::type_::TypeDescriptor::Primitive(
                crate::reflect::primitive::PrimitiveType::I16,
            ),
        }
    };

    // Emit a Rust method to access a field.
    (@emit_field_accessor { name: $name:ident offset: $offset:literal type: u16 }) => {
        pub fn $name(self) -> u16 {
            ::byteorder::ReadBytesExt::read_u16::<::byteorder::BigEndian>(
                &mut &self.data[$offset..$offset + 2],
            )
            .unwrap()
        }
    };
    (@emit_field_accessor { name: $name:ident offset: $offset:literal type: i16 }) => {
        pub fn $name(self) -> i16 {
            ::byteorder::ReadBytesExt::read_i16::<::byteorder::BigEndian>(
                &mut &self.data[$offset..$offset + 2],
            )
            .unwrap()
        }
    };

    // Emit a Rust impl for the StructReader trait.
    (@emit_struct_reader_impl $name:ident (absent)) => {};
    (@emit_struct_reader_impl $name:ident (present $size:literal)) => {
        impl<'scope> crate::slice::StructReader<'scope> for $name<'scope> {
            const SIZE: usize = $size;

            fn new(data: &'scope [u8]) -> Self {
                Self { data }
            }
        }
    };

    // Rust-style `Some(value)` is two tt tokens, while `None` is only one tt token. To keep things
    // easy to parse, this macro uses `(present value)`/`(absent)` instead. These two expressions
    // convert the present/absent form to a Rust option literal.
    (@option_to_rust (absent)) => {
        None
    };
    (@option_to_rust (present $size:literal)) => {
        Some($size)
    };

    // Catch-all for the initial invocation.
    ($($args:tt)*) => {
        compile_interfaces! {
            @parse: Init { size: (absent) }
            $($args)*
        }
    };
}
