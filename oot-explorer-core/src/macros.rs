macro_rules! gen_types {
    ($($args:tt)*) => {
        __impl_gen_types! {
            state: Init {
                size: false 0,
            },
            $($args)*
        }
    };
}

macro_rules! __impl_gen_types {
    // End of input.
    (
        state: Init {$($ignored_state:tt)*},
    ) => {};

    // An attribute.
    (
        state: Init {
            size: $size_is_some:ident $size:literal,
        },
        #[size($new_size:literal)]
        $($tail:tt)*
    ) => {
        __impl_gen_types! {
            state: Init {
                // Record the newly defined size.
                size: true $size,
            },
            $($tail)*
        }
    };

    // Start of struct.
    (
        state: Init {
            size: $size_is_some:ident $size:literal,
        },
        struct $name:ident {
            $($struct_tail:tt)*
        }
        $($tail:tt)*
    ) => {
        __impl_gen_types! {
            // We are now in struct state.
            state: Struct {
                name: $name,
                size: $size_is_some $size,
                fields: [],
            },
            { $($struct_tail)* }
            $($tail)*
        }
    };

    // Struct field declaration.
    (
        state: Struct {
            name: $name:ident,
            size: $size_is_some:ident $size:literal,
            fields: [
                $({
                    name: $field_name:ident,
                    offset: $field_offset:literal,
                    type: $field_type:ident,
                },)*
            ],
        },
        {
            $new_field_type:ident $new_field_name:ident @ $new_field_offset:literal;
            $($struct_tail:tt)*
        }
        $($tail:tt)*
    ) => {
        __impl_gen_types! {
            state: Struct {
                name: $name,
                size: $size_is_some $size,
                fields: [
                    $({
                        name: $field_name,
                        offset: $field_offset,
                        type: $field_type,
                    },)*
                    // New field.
                    {
                        name: $new_field_name,
                        offset: $new_field_offset,
                        type: $new_field_type,
                    },
                ],
            },
            { $($struct_tail)* }
            $($tail)*
        }
    };

    // End of struct.
    (
        state: Struct {
            name: $name:ident,
            size: $size_is_some:ident $size:literal,
            fields: [
                $({
                    name: $field_name:ident,
                    offset: $field_offset:literal,
                    type: $field_type:ident,
                },)*
            ],
        },
        { /* empty */ }
        $($tail:tt)*
    ) => {
        // Generate the reflection table.
        ::paste::paste! {
            pub const [<$name:snake:upper _DESC>]: crate::reflect::type_::TypeDescriptor =
                crate::reflect::type_::TypeDescriptor::Struct(
                    &crate::reflect::struct_::StructDescriptor {
                        name: "",
                        size: __impl_reflect_size_field!($size_is_some $size),
                        is_end: None,
                        fields: &[$(
                            __impl_reflect_field!($field_type $field_name $field_offset),
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
                __impl_reader_field!($field_type $field_name $field_offset);
            )*
        }

        __impl_struct_reader!($name $size_is_some $size);

        __impl_gen_types! {
            // We are back in the initial state.
            state: Init {
                size: false 0,
            },
            $($tail)*
        }
    };
}

macro_rules! __impl_reflect_size_field {
    (false $size:literal) => {
        None
    };
    (true $size:literal) => {
        Some($size)
    };
}

macro_rules! __impl_reflect_field {
    (u16 $name:ident $offset:literal) => {
        crate::reflect::struct_::FieldDescriptor {
            name: stringify!($name),
            location: crate::reflect::struct_::StructFieldLocation::Simple { offset: $offset },
            desc: crate::reflect::type_::TypeDescriptor::Primitive(
                crate::reflect::primitive::PrimitiveType::U16,
            ),
        }
    };
    (i16 $name:ident $offset:literal) => {
        crate::reflect::struct_::FieldDescriptor {
            name: stringify!($name),
            location: crate::reflect::struct_::StructFieldLocation::Simple { offset: $offset },
            desc: crate::reflect::type_::TypeDescriptor::Primitive(
                crate::reflect::primitive::PrimitiveType::I16,
            ),
        }
    };
}

macro_rules! __impl_reader_field {
    (u16 $name:ident $offset:literal) => {
        pub fn $name(self) -> u16 {
            ::byteorder::ReadBytesExt::read_u16::<::byteorder::BigEndian>(
                &mut &self.data[$offset..$offset + 2],
            )
            .unwrap()
        }
    };
    (i16 $name:ident $offset:literal) => {
        pub fn $name(self) -> i16 {
            ::byteorder::ReadBytesExt::read_i16::<::byteorder::BigEndian>(
                &mut &self.data[$offset..$offset + 2],
            )
            .unwrap()
        }
    };
}

macro_rules! __impl_struct_reader {
    ($name:ident false $size:literal) => {};
    ($name:ident true $size:literal) => {
        impl<'scope> crate::slice::StructReader<'scope> for $name<'scope> {
            const SIZE: usize = $size;

            fn new(data: &'scope [u8]) -> Self {
                Self { data }
            }
        }
    };
}
