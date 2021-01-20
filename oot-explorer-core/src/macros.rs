macro_rules! declare_pointer_descriptor {
    ($type:ident) => {
        ::paste::paste! {
            pub const [<$type:snake:upper _PTR_DESC>]: crate::reflect::type_::TypeDescriptor =
                crate::reflect::type_::TypeDescriptor::Pointer(
                    &crate::reflect::pointer::PointerDescriptor {
                        name: concat!(stringify!($type), "*"),
                        underlying: [<$type:snake:upper _DESC>],
                    },
                );
        }
    };
}

macro_rules! compile_interfaces {
    // Parse the end of input.
    (@parse Init $ignored_state:tt /* empty */) => {};

    // Parse a size attribute.
    (
        @parse Init { size: $ignored_size:tt is_end: $is_end:tt }
        #[size($size:literal)]
        $($tail:tt)*
    ) => {
        compile_interfaces! {
            // Record the newly defined size.
            @parse Init { size: (Some($size)) is_end: $is_end }
            $($tail)*
        }
    };

    // Parse an is_end attribute.
    (
        @parse Init { size: $size:tt is_end: $ignored_is_end:tt }
        #[is_end($is_end:expr)]
        $($tail:tt)*
    ) => {
        compile_interfaces! {
            // Record the newly defined is_end function.
            @parse Init { size: $size is_end: (Some($is_end)) }
            $($tail)*
        }
    };

    // Parse the start of a struct.
    (
        @parse Init { size: $size:tt is_end: $is_end:tt }
        struct $name:ident {
            $($body:tt)*
        }
        $($tail:tt)*
    ) => {
        compile_interfaces! {
            // We are now parsing a struct.
            @parse Struct { name: $name size: $size is_end: $is_end fields: [] }
            { $($body)* }
            $($tail)*
        }
    };

    // Parse a struct field declaration of scalar type.
    (
        @parse Struct {
            name: $name:ident
            size: $size:tt
            is_end: $is_end:tt
            fields: [$($field:tt)*]
        }
        {
            $field_type:ident $field_name:ident @$field_offset:literal;
            $($body:tt)*
        }
        $($tail:tt)*
    ) => {
        compile_interfaces! {
            @parse Struct {
                name: $name
                size: $size
                is_end: $is_end
                fields: [
                    $($field)*
                    // New field.
                    { name: $field_name type: (scalar $field_type) location: (simple $field_offset)  }
                ]
            }
            { $($body)* }
            $($tail)*
        }
    };

    // Parse a struct field declaration of aggregate type.
    (
        @parse Struct {
            name: $name:ident
            size: $size:tt
            is_end: $is_end:tt
            fields: [$($field:tt)*]
        }
        {
            struct $field_type:ident $field_name:ident @$field_offset:literal;
            $($body:tt)*
        }
        $($tail:tt)*
    ) => {
        compile_interfaces! {
            @parse Struct {
                name: $name
                size: $size
                is_end: $is_end
                fields: [
                    $($field)*
                    // New field.
                    {
                        name: $field_name
                        type: (aggregate $field_type)
                        location: (simple $field_offset)
                    }
                ]
            }
            { $($body)* }
            $($tail)*
        }
    };

    // Parse a struct field declaration of pointer type.
    (
        @parse Struct {
            name: $name:ident
            size: $size:tt
            is_end: $is_end:tt
            fields: [$($field:tt)*]
        }
        {
            struct $field_type:ident *$field_name:ident @$field_offset:literal;
            $($body:tt)*
        }
        $($tail:tt)*
    ) => {
        compile_interfaces! {
            @parse Struct {
                name: $name
                size: $size
                is_end: $is_end
                fields: [
                    $($field)*
                    // New field.
                    {
                        name: $field_name
                        type: (ptr-aggregate $field_type)
                        location: (simple $field_offset)
                    }
                ]
            }
            { $($body)* }
            $($tail)*
        }
    };

    // Parse a struct field declaration of slice type.
    (
        @parse Struct {
            name: $name:ident
            size: $size:tt
            is_end: $is_end:tt
            fields: [$($field:tt)*]
        }
        {
            struct $type:ident[$count_type:ident @$count_offset:literal]* $field_name:ident
                @$ptr_offset:literal;
            $($body:tt)*
        }
        $($tail:tt)*
    ) => {
        compile_interfaces! {
            @parse Struct {
                name: $name
                size: $size
                is_end: $is_end
                fields: [
                    $($field)*
                    // New field.
                    {
                        name: $field_name
                        type: (aggregate $type)
                        location: (slice $count_type $count_offset $ptr_offset)
                    }
                ]
            }
            { $($body)* }
            $($tail)*
        }
    };

    // Parse a struct field declaration of inline delimited list type.
    (
        @parse Struct {
            name: $name:ident
            size: $size:tt
            is_end: $is_end:tt
            fields: [$($field:tt)*] }
        {
            struct $type:ident[..] $field_name:ident @$offset:literal;
            $($body:tt)*
        }
        $($tail:tt)*
    ) => {
        compile_interfaces! {
            @parse Struct {
                name: $name
                size: $size
                is_end: $is_end
                fields: [
                    $($field)*
                    // New field.
                    {
                        name: $field_name
                        type: (aggregate $type)
                        location: (inline_delimited_list $offset)
                    }
                ]
            }
            { $($body)* }
            $($tail)*
        }
    };

    // Parse the end of a struct.
    (
        @parse Struct {
            name: $name:ident
            size: $size:tt
            is_end: $is_end:tt
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
                        name: stringify!($name),
                        size: $size,
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

        impl<'scope> crate::reflect::instantiate::Instantiate<'scope> for $name<'scope> {
            fn new(data: &'scope [u8]) -> Self {
                Self { data }
            }
        }

        compile_interfaces!(@emit_reflect_sized_impl $name $size);

        compile_interfaces! { $($tail)* }
    };

    // Emit a Rust `FieldDescriptor` literal.
    (@emit_field_descriptor {
        name: $name:ident
        type: (scalar $type:ident)
        location: (simple $offset:literal)
    }) => {
        compile_interfaces!(@emit_nonptr_simple_field_descriptor $name $type $offset)
    };
    (@emit_field_descriptor {
        name: $name:ident
        type: (aggregate $type:ident)
        location: (simple $offset:literal)
    }) => {
        compile_interfaces!(@emit_nonptr_simple_field_descriptor $name $type $offset)
    };
    (@emit_field_descriptor {
        name: $name:ident
        type: (ptr-aggregate $type:ident)
        location: (simple $offset:literal)
    }) => {
        ::paste::paste! {
            crate::reflect::struct_::FieldDescriptor {
                name: stringify!($name),
                location: crate::reflect::struct_::StructFieldLocation::Simple { offset: $offset },
                desc: [<$type:snake:upper _PTR_DESC>],
            }
        }
    };
    (@emit_field_descriptor {
        name: $name:ident
        type: (aggregate $ptr_type:ident)
        location: (slice $count_type:ident $count_offset:literal $ptr_offset:literal)
    }) => {
        ::paste::paste! {
            crate::reflect::struct_::FieldDescriptor {
                name: stringify!($name),
                location: crate::reflect::struct_::StructFieldLocation::Slice {
                    count_offset: $count_offset,
                    count_desc: compile_interfaces!(@primitive_type_literal $count_type),
                    ptr_offset: $ptr_offset,
                },
                desc: [<$ptr_type:snake:upper _DESC>],
            }
        }
    };
    (@emit_field_descriptor {
        name: $name:ident
        type: (aggregate $type:ident)
        location: (inline_delimited_list $offset:literal)
    }) => {
        ::paste::paste! {
            crate::reflect::struct_::FieldDescriptor {
                name: stringify!($name),
                location: crate::reflect::struct_::StructFieldLocation::InlineDelimitedList {
                    offset: $offset,
                },
                desc: [<$type:snake:upper _DESC>],
            }
        }
    };

    // Emit a Rust `FieldDescriptor` literal for a field that has a non-pointer type and simple
    // location.
    (@emit_nonptr_simple_field_descriptor $name:ident $type:ident $offset:literal) => {
        ::paste::paste! {
            crate::reflect::struct_::FieldDescriptor {
                name: stringify!($name),
                location: crate::reflect::struct_::StructFieldLocation::Simple { offset: $offset },
                desc: [<$type:snake:upper _DESC>],
            }
        }
    };

    // Emit a Rust method to access a field.
    (@emit_field_accessor {
        name: $name:ident
        type: (scalar $type:ident)
        location: (simple $offset:literal)
    }) => {
        compile_interfaces!(@emit_scalar_simple_field_accessor $name $type $offset);
    };
    (@emit_field_accessor {
        name: $name:ident
        type: (aggregate $type:ident)
        location: (simple $offset:literal)
    }) => {
        compile_interfaces!(@emit_aggregate_simple_field_accessor $name $type $offset);
    };
    (@emit_field_accessor {
        name: $name:ident
        type: (ptr-aggregate $type:ident)
        location: (simple $offset:literal)
    }) => {
        pub fn $name(self, segment_ctx: &crate::segment::SegmentCtx<'scope>) -> $type<'scope> {
            let ptr = crate::segment::SegmentAddr(
                compile_interfaces!(@read_simple_field self u32 $offset),
            );
            let data = segment_ctx.resolve(ptr).unwrap();
            <$type as crate::reflect::instantiate::Instantiate>::new(data)
        }
    };
    (@emit_field_accessor {
        name: $name:ident
        type: (aggregate $type:ident)
        location: (slice $count_type:ident $count_offset:literal $ptr_offset:literal)
    }) => {
        pub fn $name(
            self,
            segment_ctx: &crate::segment::SegmentCtx<'scope>,
        ) -> crate::slice::Slice<'scope, $type<'scope>> {
            let ptr = crate::segment::SegmentAddr(
                compile_interfaces!(@read_simple_field self u32 $ptr_offset),
            );
            let count = compile_interfaces!(@read_simple_field self $count_type $count_offset);
            crate::slice::Slice::new(segment_ctx.resolve(ptr).unwrap(), count as usize)
        }
    };
    (@emit_field_accessor {
        name: $name:ident
        type: (aggregate $type:ident)
        location: (inline_delimited_list $offset:literal)
    }) => {
        pub fn $name(self) -> impl ::std::iter::Iterator<Item = $type<'scope>> {
            crate::delimited::Iter::<$type<'scope>>::new(&self.data[$offset..])
        }
    };

    // Emit a Rust method to access a field that has a scalar type and simple location.
    (@emit_scalar_simple_field_accessor $name:ident $type:ident $offset:literal) => {
        pub fn $name(self) -> $type {
            compile_interfaces!(@read_simple_field self $type $offset)
        }
    };

    // Emit a Rust method to access a field that has an aggregate type and simple location.
    (@emit_aggregate_simple_field_accessor $name:ident $type:ident $offset:literal) => {
        pub fn $name(self) -> $type<'scope> {
            compile_interfaces!(@read_simple_field self $type $offset)
        }
    };

    // Emit a Rust expression to access a field with simple location.
    (@read_simple_field $self:ident $type:ident $offset:literal) => {
        <$type as crate::reflect::instantiate::Instantiate<'scope>>::new(&$self.data[$offset..])
    };

    // Emit a Rust impl for the `ReflectSized` trait.
    (@emit_reflect_sized_impl $name:ident (None)) => {};
    (@emit_reflect_sized_impl $name:ident (Some($size:literal))) => {
        impl<'scope> crate::reflect::sized::ReflectSized for $name<'scope> {
            const SIZE: usize = $size;
        }
    };

    // Parse the start of a union.
    (
        @parse Init { size: $size:tt is_end: $is_end:tt }
        union $name:ident: $discriminant_type:ident @$discriminant_offset:literal {
            $($body:tt)*
        }
        $($tail:tt)*
    ) => {
        compile_interfaces! {
            // We are now parsing a union.
            @parse Union {
                name: $name
                size: $size
                is_end: $is_end
                discriminant_offset: $discriminant_offset
                discriminant_type: $discriminant_type
                variants: []
            }
            { $($body)* }
            $($tail)*
        }
    };

    // Parse a union variant of aggregate type.
    (
        @parse Union {
            name: $name:ident
            size: $size:tt
            is_end: $is_end:tt
            discriminant_offset: $discriminant_offset:literal
            discriminant_type: $discriminant_type:ident
            variants: [$($variant:tt)*]
        }
        {
            struct $field_type:ident $field_name:ident #$field_discriminant:expr;
            $($body:tt)*
        }
        $($tail:tt)*
    ) => {
        compile_interfaces! {
            @parse Union {
                name: $name
                size: $size
                is_end: $is_end
                discriminant_offset: $discriminant_offset
                discriminant_type: $discriminant_type
                    variants: [
                    $($variant)*
                    // New variant.
                    {
                        name: $field_name
                        discriminant: ($field_discriminant)
                        type: (aggregate $field_type)
                    }
                ]
            }
            { $($body)* }
            $($tail)*
        }
    };

    // Parse the end of a union.
    (
        @parse Union {
            name: $name:ident
            size: $size:tt
            is_end: ($is_end:expr)
            discriminant_offset: $discriminant_offset:literal
            discriminant_type: $discriminant_type:ident
            variants: [$({
                name: $variant_name:ident
                discriminant: ($variant_discriminant:expr)
                type: (aggregate $variant_type:ident)
            })*]
        }
        { /* empty */ }
        $($tail:tt)*
    ) => {
        // Generate the reflection table.
        ::paste::paste! {
            pub const [<$name:snake:upper _DESC>]: crate::reflect::type_::TypeDescriptor =
                crate::reflect::type_::TypeDescriptor::Union(
                    &crate::reflect::struct_::UnionDescriptor {
                        name: stringify!($name),
                        size: $size,
                        // TODO: Provide a way to specify this!
                        is_end: $is_end,
                        discriminant_offset: $discriminant_offset,
                        discriminant_desc: [<$discriminant_type:snake:upper _DESC>],
                        variants: &[$(
                            compile_interfaces!(@emit_variant_entry {
                                name: $variant_name
                                discriminant: ($variant_discriminant)
                                type: $variant_type
                            }),
                        )*],
                    },
                );

            // Generate the reader type.
            #[derive(Clone, Copy)]
            pub struct $name<'scope> {
                data: &'scope [u8],
            }

            impl<'scope> $name<'scope> {
                pub fn new(data: &'scope [u8]) -> Self {
                    Self { data }
                }

                pub fn discriminant(self) -> $discriminant_type {
                    compile_interfaces!(
                        @read_simple_field self $discriminant_type $discriminant_offset
                    )
                }

                pub fn variant(self) -> [<$name Variant>]<'scope> {
                    match self.discriminant() {
                        $(
                            $variant_discriminant => [<$name Variant>]::[<$variant_name:camel>](
                                <$variant_type as crate::reflect::instantiate::Instantiate<'scope>>
                                    ::new(self.data)),
                        )*
                        discriminant => panic!(
                            concat!(
                                "unexpected union discriminant for ",
                                stringify!($name),
                                ": {}",
                            ),
                            discriminant.to_u32(),
                        ),
                    }
                }
            }

            impl<'scope> crate::reflect::instantiate::Instantiate<'scope> for $name<'scope> {
                fn new(data: &'scope [u8]) -> Self {
                    Self { data }
                }
            }

            compile_interfaces!(@emit_reflect_sized_impl $name $size);

            // Generate the variant enum.
            #[derive(Clone, Copy)]
            pub enum [<$name Variant>]<'scope> {$(
                // TODO: Just tacking <'scope> on the type is not going to be valid if union
                // variants are expanded beyond just structs.
                [<$variant_name:camel>]($variant_type<'scope>),
            )*}
        }

        compile_interfaces! { $($tail)* }
    };

    // Emit a Rust expression for an entry in a `UnionDescriptor.variants` list.
    (@emit_variant_entry {
        name: $name:ident
        discriminant: ($discriminant:expr)
        type: $type:ident
    }) => {
        ::paste::paste! { ($discriminant.to_u32(),[<$type:snake:upper _DESC>]) }
    };

    // Emit a Rust enum declaration for a union.
    (@emit_union_variant_enum $name:ident [$(
        { name: $variant_name:ident discriminant: $_:tt type: $variant_type:ident }
    )*]) => {
    };

    // Map primitive types to `PrimitiveType` literals.
    (@primitive_type_literal u8) => { crate::reflect::primitive::PrimitiveType::U8 };
    (@primitive_type_literal i8) => { crate::reflect::primitive::PrimitiveType::I8 };
    (@primitive_type_literal u16) => { crate::reflect::primitive::PrimitiveType::U16 };
    (@primitive_type_literal i16) => { crate::reflect::primitive::PrimitiveType::I16 };
    (@primitive_type_literal u32) => { crate::reflect::primitive::PrimitiveType::U32 };
    (@primitive_type_literal i32) => { crate::reflect::primitive::PrimitiveType::I32 };

    // Catch-all handler for input that didn't match anything.
    (@ $($args:tt)*) => {
        compile_error!(concat!(
            "a match fell through: @",
            $(stringify!($args), " ",)*
        ));
    };

    // Handle the initial invocation.
    ($($args:tt)*) => {
        compile_interfaces! {
            @parse Init { size: (None) is_end: (None) }
            $($args)*
        }
    };
}
