macro_rules! declare_pointer_descriptor {
    ($type:ident) => {
        ::paste::paste! {
            pub const [<$type:snake:upper _PTR_DESC>]: ::oot_explorer_reflect::TypeDescriptor =
                ::oot_explorer_reflect::TypeDescriptor::Pointer(
                    &::oot_explorer_reflect::PointerDescriptor {
                        name: concat!(stringify!($type), "*"),
                        target: [<$type:snake:upper _DESC>],
                    },
                );
        }
    };
}

macro_rules! compile_interfaces {
    // Parse the end of input.
    (@parse Init $ignored_state:tt /* empty */) => {};

    // Parse an is_end attribute.
    (
        // Parse state.
        @parse Init { is_end: $ignored_is_end:tt }

        // Item to parse.
        #[is_end($is_end:expr)]

        // Remainder of input.
        $($tail:tt)*
    ) => {
        compile_interfaces! {
            // Record the newly defined is_end function.
            @parse Init { is_end: (Some($is_end)) }
            $($tail)*
        }
    };

    // Parse the start of a struct.
    (
        // Parse state.
        @parse Init { is_end: $is_end:tt }

        // Item to parse.
        #[layout(size = $size:literal, align_bits = $align_bits:literal)]
        struct $name:ident {
            $($body:tt)*
        }

        // Remainder of input.
        $($tail:tt)*
    ) => {
        compile_interfaces! {
            // We are now parsing a struct.
            @parse Struct {
                name: $name
                size: $size
                align_bits: $align_bits
                is_end: $is_end
                prev_field: (None)
                fields: []
            }
            { $($body)* }
            $($tail)*
        }
    };

    // Parse a struct field declaration of scalar type.
    (
        // Parse state.
        @parse Struct {
            name: $name:ident
            size: $size:literal
            align_bits: $align_bits:literal
            is_end: $is_end:tt
            prev_field: $prev_field:tt
            fields: [$($field:tt)*]
        }

        // Item to parse.
        {
            $field_type:ident $field_name:ident @$field_offset:literal;
            $($body:tt)*
        }

        // Remainder of input.
        $($tail:tt)*
    ) => {
        compile_interfaces! {
            @parse Struct {
                name: $name
                size: $size
                align_bits: $align_bits
                is_end: $is_end
                prev_field: (Some($field_name))
                fields: [
                    $($field)*
                    // New field.
                    {
                        name: $field_name
                        type: (scalar $field_type)
                        location: (simple $field_offset)
                        prev: $prev_field
                    }
                ]
            }
            { $($body)* }
            $($tail)*
        }
    };

    // Parse a struct field declaration of aggregate type.
    (
        // Parse state.
        @parse Struct {
            name: $name:ident
            size: $size:literal
            align_bits: $align_bits:literal
            is_end: $is_end:tt
            prev_field: $prev_field:tt
            fields: [$($field:tt)*]
        }

        // Item to parse.
        {
            struct $field_type:ident $field_name:ident @$field_offset:literal;
            $($body:tt)*
        }

        // Remainder of input.
        $($tail:tt)*
    ) => {
        compile_interfaces! {
            @parse Struct {
                name: $name
                size: $size
                align_bits: $align_bits
                is_end: $is_end
                prev_field: (Some($field_name))
                fields: [
                    $($field)*
                    // New field.
                    {
                        name: $field_name
                        type: (aggregate $field_type)
                        location: (simple $field_offset)
                        prev: $prev_field
                    }
                ]
            }
            { $($body)* }
            $($tail)*
        }
    };

    // Parse a struct field declaration of pointer type.
    (
        // Parse state.
        @parse Struct {
            name: $name:ident
            size: $size:literal
            align_bits: $align_bits:literal
            is_end: $is_end:tt
            prev_field: $prev_field:tt
            fields: [$($field:tt)*]
        }

        // Item to parse.
        {
            struct $field_type:ident *$field_name:ident @$field_offset:literal;
            $($body:tt)*
        }

        // Remainder of input.
        $($tail:tt)*
    ) => {
        compile_interfaces! {
            @parse Struct {
                name: $name
                size: $size
                align_bits: $align_bits
                is_end: $is_end
                prev_field: (Some($field_name))
                fields: [
                    $($field)*
                    // New field.
                    {
                        name: $field_name
                        type: (ptr-aggregate $field_type)
                        location: (simple $field_offset)
                        prev: $prev_field
                    }
                ]
            }
            { $($body)* }
            $($tail)*
        }
    };

    // Parse a struct field declaration of slice type pointing to a scalar type.
    (
        // Parse state.
        @parse Struct {
            name: $name:ident
            size: $size:literal
            align_bits: $align_bits:literal
            is_end: $is_end:tt
            prev_field: $prev_field:tt
            fields: [$($field:tt)*]
        }

        // Item to parse.
        {
            $type:ident[$count_type:ident @$count_offset:literal]* $field_name:ident
                @$ptr_offset:literal;
            $($body:tt)*
        }

        // Remainder of input.
        $($tail:tt)*
    ) => {
        compile_interfaces! {
            @parse Struct {
                name: $name
                size: $size
                align_bits: $align_bits
                is_end: $is_end
                prev_field: (Some($field_name))
                fields: [
                    $($field)*
                    // New field.
                    {
                        name: $field_name
                        type: (scalar $type)
                        location: (slice $count_type $count_offset $ptr_offset)
                        prev: $prev_field
                    }
                ]
            }
            { $($body)* }
            $($tail)*
        }
    };

    // Parse a struct field declaration of slice type pointing to an aggregate type.
    (
        // Parse state.
        @parse Struct {
            name: $name:ident
            size: $size:literal
            align_bits: $align_bits:literal
            is_end: $is_end:tt
            prev_field: $prev_field:tt
            fields: [$($field:tt)*]
        }

        // Item to parse.
        {
            struct $type:ident[$count_type:ident @$count_offset:literal]* $field_name:ident
                @$ptr_offset:literal;
            $($body:tt)*
        }

        // Remainder of input.
        $($tail:tt)*
    ) => {
        compile_interfaces! {
            @parse Struct {
                name: $name
                size: $size
                align_bits: $align_bits
                is_end: $is_end
                prev_field: (Some($field_name))
                fields: [
                    $($field)*
                    // New field.
                    {
                        name: $field_name
                        type: (aggregate $type)
                        location: (slice $count_type $count_offset $ptr_offset)
                        prev: $prev_field
                    }
                ]
            }
            { $($body)* }
            $($tail)*
        }
    };

    // Parse a struct field declaration of inline delimited list type.
    (
        // Parse state.
        @parse Struct {
            name: $name:ident
            size: $size:literal
            align_bits: $align_bits:literal
            is_end: $is_end:tt
            prev_field: $prev_field:tt
            fields: [$($field:tt)*]
        }

        // Item to parse.
        {
            struct $type:ident[..] $field_name:ident @$offset:literal;
            $($body:tt)*
        }

        // Remainder of input.
        $($tail:tt)*
    ) => {
        compile_interfaces! {
            @parse Struct {
                name: $name
                size: $size
                align_bits: $align_bits
                is_end: $is_end
                prev_field: (Some($field_name))
                fields: [
                    $($field)*
                    // New field.
                    {
                        name: $field_name
                        type: (aggregate $type)
                        location: (inline_delimited_list $offset)
                        prev: $prev_field
                    }
                ]
            }
            { $($body)* }
            $($tail)*
        }
    };

    // Parse the end of a struct.
    (
        // Parse state.
        @parse Struct {
            name: $name:ident
            size: $size:literal
            align_bits: $align_bits:literal
            is_end: $is_end:tt
            prev_field: $prev_field:tt
            fields: [$($field:tt)*]
        }

        // Item to parse.
        { /* empty */ }

        // Remainder of input.
        $($tail:tt)*
    ) => {
        // Generate the reflection table.
        ::paste::paste! {
            pub const [<$name:snake:upper _DESC>]: ::oot_explorer_reflect::TypeDescriptor =
                ::oot_explorer_reflect::TypeDescriptor::Struct(
                    &::oot_explorer_reflect::StructDescriptor {
                        name: stringify!($name),
                        size: Some($size),
                        is_end: None,
                        fields: &[$(
                            compile_interfaces!(@emit_field_descriptor $name $field),
                        )*],
                    },
                );
        }

        // Generate the reader type.

        #[derive(Clone, Copy)]
        pub struct $name {
            addr: oot_explorer_vrom::VromAddr,
        }

        ::paste::paste! {
            #[allow(dead_code)]
            impl $name {
                $(
                    compile_interfaces!(@emit_field_accessor $field);
                )*
            }
        }

        impl ::oot_explorer_read::FromVrom for $name {
            fn from_vrom(
                vrom: ::oot_explorer_vrom::Vrom<'_>,
                addr: ::oot_explorer_vrom::VromAddr,
            ) -> ::std::result::Result<Self, ::oot_explorer_read::ReadError> {
                ::oot_explorer_read::aligned_data::<Self>(vrom, addr)?;
                Ok(Self { addr })
            }
        }

        impl ::oot_explorer_read::VromProxy for $name {
            fn addr(&self) -> ::oot_explorer_vrom::VromAddr {
                self.addr
            }
        }

        impl ::oot_explorer_read::Layout for $name {
            const SIZE: u32 = $size;
            const ALIGN_BITS: u32 = $align_bits;
        }

        compile_interfaces! { $($tail)* }
    };

    // Emit a Rust `FieldDescriptor` literal.
    (@emit_field_descriptor $struct_name:ident {
        name: $name:ident
        type: (scalar $type:ident)
        location: (simple $offset:literal)
        prev: $_:tt
    }) => {
        compile_interfaces!(@emit_nonptr_simple_field_descriptor $name $type $offset)
    };
    (@emit_field_descriptor $struct_name:ident {
        name: $name:ident
        type: (aggregate $type:ident)
        location: (simple $offset:literal)
        prev: $_:tt
    }) => {
        compile_interfaces!(@emit_nonptr_simple_field_descriptor $name $type $offset)
    };
    (@emit_field_descriptor $struct_name:ident {
        name: $name:ident
        type: (ptr-aggregate $type:ident)
        location: (simple $offset:literal)
        prev: $_:tt
    }) => {
        ::paste::paste! {
            ::oot_explorer_reflect::FieldDescriptor {
                name: stringify!($name),
                location: ::oot_explorer_reflect::StructFieldLocation::Simple { offset: $offset },
                desc: [<$type:snake:upper _PTR_DESC>],
            }
        }
    };
    (@emit_field_descriptor $struct_name:ident {
        name: $name:ident
        type: (scalar $ptr_type:ident)
        location: (slice $count_type:ident $count_offset:literal $ptr_offset:literal)
        prev: $_:tt
    }) => {
        ::paste::paste! {
            ::oot_explorer_reflect::FieldDescriptor {
                name: stringify!($name),
                location: ::oot_explorer_reflect::StructFieldLocation::Slice {
                    count_offset: $count_offset,
                    count_desc: compile_interfaces!(@primitive_type_literal $count_type),
                    ptr_offset: $ptr_offset,
                },
                desc: [<$ptr_type:snake:upper _DESC>],
            }
        }
    };
    (@emit_field_descriptor $struct_name:ident {
        name: $name:ident
        type: (aggregate $ptr_type:ident)
        location: (slice $count_type:ident $count_offset:literal $ptr_offset:literal)
        prev: $_:tt
    }) => {
        ::paste::paste! {
            ::oot_explorer_reflect::FieldDescriptor {
                name: stringify!($name),
                location: ::oot_explorer_reflect::StructFieldLocation::Slice {
                    count_offset: $count_offset,
                    count_desc: compile_interfaces!(@primitive_type_literal $count_type),
                    ptr_offset: $ptr_offset,
                },
                desc: [<$ptr_type:snake:upper _DESC>],
            }
        }
    };
    (@emit_field_descriptor $struct_name:ident {
        name: $name:ident
        type: (aggregate $type:ident)
        location: (inline_delimited_list $offset:literal)
        prev: $_:tt
    }) => {
        ::paste::paste! {
            ::oot_explorer_reflect::FieldDescriptor {
                name: stringify!($name),
                location: ::oot_explorer_reflect::StructFieldLocation::InlineDelimitedList {
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
            ::oot_explorer_reflect::FieldDescriptor {
                name: stringify!($name),
                location: ::oot_explorer_reflect::StructFieldLocation::Simple { offset: $offset },
                desc: [<$type:snake:upper _DESC>],
            }
        }
    };

    // Emit a Rust method to access a field.
    (@emit_field_accessor {
        name: $name:ident
        type: (scalar $type:ident)
        location: (simple $offset:literal)
        prev: $_:tt
    }) => {
        compile_interfaces!(@emit_scalar_simple_field_accessor $name $type $offset);
    };
    (@emit_field_accessor {
        name: $name:ident
        type: (aggregate $type:ident)
        location: (simple $offset:literal)
        prev: $_:tt
    }) => {
        compile_interfaces!(@emit_aggregate_simple_field_accessor $name $type $offset);
    };
    (@emit_field_accessor {
        name: $name:ident
        type: (ptr-aggregate $type:ident)
        location: (simple $offset:literal)
        prev: $_:tt
    }) => {
        pub fn $name(
            self,
            vrom: ::oot_explorer_vrom::Vrom<'_>,
            segment_table: &::oot_explorer_segment::SegmentTable,
        ) -> ::std::result::Result<$type, ::oot_explorer_segment::SegmentError> {
            let segment_addr = ::oot_explorer_segment::SegmentAddr(
                compile_interfaces!(@read_simple_field self vrom u32 $offset),
            );
            let vrom_addr = segment_table.resolve(segment_addr)?;
            // Unwrap because struct size and alignment have already been checked.
            Ok(<$type as ::oot_explorer_read::FromVrom>::from_vrom(vrom, vrom_addr).unwrap())
        }
    };
    (@emit_field_accessor {
        name: $name:ident
        type: (scalar $type:ident)
        location: (slice $count_type:ident $count_offset:literal $ptr_offset:literal)
        prev: $_:tt
    }) => {
        pub fn $name(
            self,
            vrom: ::oot_explorer_vrom::Vrom<'_>,
            segment_table: &::oot_explorer_segment::SegmentTable,
        ) -> ::std::result::Result<
            ::oot_explorer_read::Slice<$type>,
            ::oot_explorer_segment::SegmentError,
        > {
            let segment_addr = ::oot_explorer_segment::SegmentAddr(
                compile_interfaces!(@read_simple_field self vrom u32 $ptr_offset),
            );
            let vrom_addr = segment_table.resolve(segment_addr)?;
            let count = compile_interfaces!(@read_simple_field self vrom $count_type $count_offset);
            Ok(::oot_explorer_read::Slice::new(vrom_addr, count as u32))
        }
    };
    (@emit_field_accessor {
        name: $name:ident
        type: (aggregate $type:ident)
        location: (slice $count_type:ident $count_offset:literal $ptr_offset:literal)
        prev: $_:tt
    }) => {
        pub fn $name(
            self,
            vrom: ::oot_explorer_vrom::Vrom<'_>,
            segment_table: &::oot_explorer_segment::SegmentTable,
        ) -> ::std::result::Result<
            ::oot_explorer_read::Slice<$type>,
            ::oot_explorer_segment::SegmentError,
        > {
            let segment_addr = ::oot_explorer_segment::SegmentAddr(
                compile_interfaces!(@read_simple_field self vrom u32 $ptr_offset),
            );
            let vrom_addr = segment_table.resolve(segment_addr)?;
            let count = compile_interfaces!(@read_simple_field self vrom $count_type $count_offset);
            Ok(::oot_explorer_read::Slice::new(vrom_addr, count as u32))
        }
    };
    (@emit_field_accessor {
        name: $name:ident
        type: (aggregate $type:ident)
        location: (inline_delimited_list $offset:literal)
        prev: $_:tt
    }) => {
        pub fn $name(
            self,
            vrom: ::oot_explorer_vrom::Vrom<'_>,
        ) -> ::oot_explorer_read::SentinelIter<'_, $type> {
            ::oot_explorer_read::SentinelIter::new(vrom, self.addr + $offset)
        }
    };

    // Emit a Rust method to access a field that has a scalar type and simple location.
    (@emit_scalar_simple_field_accessor $name:ident $type:ident $offset:literal) => {
        pub fn $name(self, vrom: ::oot_explorer_vrom::Vrom<'_>) -> $type {
            compile_interfaces!(@read_simple_field self vrom $type $offset)
        }
    };

    // Emit a Rust method to access a field that has an aggregate type and simple location.
    (@emit_aggregate_simple_field_accessor $name:ident $type:ident $offset:literal) => {
        pub fn $name(self, vrom: ::oot_explorer_vrom::Vrom<'_>) -> $type {
            compile_interfaces!(@read_simple_field self vrom $type $offset)
        }
    };

    // Emit a Rust expression to access a field with simple location.
    (@read_simple_field $self:ident $vrom:ident $type:ident $offset:expr) => {
        // Unwrap because struct size and alignment have already been checked.
        <$type as ::oot_explorer_read::FromVrom>::from_vrom($vrom, $self.addr + $offset).unwrap()
    };

    // Parse the start of a union.
    (
        // Parse state.
        @parse Init { is_end: $is_end:tt }

        // Item to parse.
        #[layout(size = $size:literal, align_bits = $align_bits:literal)]
        union $name:ident: $discriminant_type:ident @$discriminant_offset:literal {
            $($body:tt)*
        }

        // Remainder of input.
        $($tail:tt)*
    ) => {
        compile_interfaces! {
            // We are now parsing a union.
            @parse Union {
                name: $name
                size: $size
                align_bits: $align_bits
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
        // Parse state.
        @parse Union {
            name: $name:ident
            size: $size:literal
            align_bits: $align_bits:literal
            is_end: $is_end:tt
            discriminant_offset: $discriminant_offset:literal
            discriminant_type: $discriminant_type:ident
            variants: [$($variant:tt)*]
        }

        // Item to parse.
        {
            struct $field_type:ident $field_name:ident #$field_discriminant:expr;
            $($body:tt)*
        }

        // Remainder of input.
        $($tail:tt)*
    ) => {
        compile_interfaces! {
            @parse Union {
                name: $name
                size: $size
                align_bits: $align_bits
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
        // Parse state.
        @parse Union {
            name: $name:ident
            size: $size:literal
            align_bits: $align_bits:literal
            is_end: ($is_end:expr)
            discriminant_offset: $discriminant_offset:literal
            discriminant_type: $discriminant_type:ident
            variants: [$({
                name: $variant_name:ident
                discriminant: ($variant_discriminant:expr)
                type: (aggregate $variant_type:ident)
            })*]
        }

        // Item to parse.
        { /* empty */ }

        // Remainder of input.
        $($tail:tt)*
    ) => {
        // Generate the reflection table.
        ::paste::paste! {
            pub const [<$name:snake:upper _DESC>]: ::oot_explorer_reflect::TypeDescriptor =
                ::oot_explorer_reflect::TypeDescriptor::Union(
                    &::oot_explorer_reflect::UnionDescriptor {
                        name: stringify!($name),
                        size: Some($size),
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
            pub struct $name {
                addr: VromAddr,
            }

            impl $name {
                pub fn discriminant(
                    self,
                    vrom: ::oot_explorer_vrom::Vrom<'_>,
                ) -> $discriminant_type {
                    compile_interfaces!(
                        @read_simple_field self vrom  $discriminant_type ($discriminant_offset)
                    )
                }

                pub fn variant(
                    self,
                    vrom: ::oot_explorer_vrom::Vrom<'_>,
                ) -> [<$name Variant>] {
                    match self.discriminant(vrom) {
                        $(
                            $variant_discriminant => [<$name Variant>]::[<$variant_name:camel>](
                                // Unwrap because struct size and alignment have already been
                                // checked.
                                <$variant_type as ::oot_explorer_read::FromVrom>
                                    ::from_vrom(vrom, self.addr).unwrap()
                            ),
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

            impl ::oot_explorer_read::FromVrom for $name {
                fn from_vrom(
                    vrom: ::oot_explorer_vrom::Vrom<'_>,
                    addr: ::oot_explorer_vrom::VromAddr,
                ) -> ::std::result::Result<Self, ::oot_explorer_read::ReadError> {
                    ::oot_explorer_read::aligned_data::<Self>(vrom, addr)?;
                    Ok(Self { addr })
                }
            }

            impl ::oot_explorer_read::VromProxy for $name {
                fn addr(&self) -> ::oot_explorer_vrom::VromAddr {
                    self.addr
                }
            }

            impl ::oot_explorer_read::Layout for $name {
                const SIZE: u32 = $size;
                const ALIGN_BITS: u32 = $align_bits;
            }

            // Generate the variant enum.
            #[derive(Clone, Copy)]
            pub enum [<$name Variant>] {$(
                // TODO: Just tacking <'scope> on the type is not going to be valid if union
                // variants are expanded beyond just structs.
                [<$variant_name:camel>]($variant_type),
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
    (@primitive_type_literal u8) => { ::oot_explorer_reflect::PrimitiveType::U8 };
    (@primitive_type_literal i8) => { ::oot_explorer_reflect::PrimitiveType::I8 };
    (@primitive_type_literal u16) => { ::oot_explorer_reflect::PrimitiveType::U16 };
    (@primitive_type_literal i16) => { ::oot_explorer_reflect::PrimitiveType::I16 };
    (@primitive_type_literal u32) => { ::oot_explorer_reflect::PrimitiveType::U32 };
    (@primitive_type_literal i32) => { ::oot_explorer_reflect::PrimitiveType::I32 };

    // Parse the start of an enum.
    (
        // Parse state.
        @parse Init { is_end: (None) }

        // Item to parse.
        enum $name:ident: $underlying:ident {
            $($body:tt)*
        }

        // Remainder of input.
        $($tail:tt)*
    ) => {
        compile_interfaces! {
            @parse Enum { name: $name underlying: $underlying entries: [] }
            { $($body)* }
            $($tail)*
        }
    };

    // Parse an enum entry.
    (
        // Parse state.
        @parse Enum { name: $name:ident underlying: $underlying:ident entries: [$($entry:tt)*] }

        // Item to parse.
        {
            $entry_name:ident = $entry_value:expr;
            $($body:tt)*
        }

        // Remainder of input.
        $($tail:tt)*
    ) => {
        compile_interfaces! {
            @parse Enum {
                name: $name
                underlying: $underlying
                entries: [
                    $($entry)*
                    // New entry.
                    { name: $entry_name value: $entry_value }
                ]
            }
            { $($body)* }
            $($tail)*
        }
    };

    // Parse the end of an enum.
    (
        // Parse state.
        @parse Enum {
            name: $name:ident
            underlying: $underlying:ident
            entries: [$({ name: $entry_name:ident value: $entry_value:expr })*]
        }

        // Item to parse.
        { /* empty */ }

        // Remainder of input.
        $($tail:tt)*
    ) => {
        // Generate the reflection table.
        ::paste::paste! {
            pub const [<$name:snake:upper _DESC>]: ::oot_explorer_reflect::TypeDescriptor =
            ::oot_explorer_reflect::TypeDescriptor::Enum(
                    &::oot_explorer_reflect::EnumDescriptor {
                        name: stringify!($name),
                        underlying: compile_interfaces!(@primitive_type_literal $underlying),
                        values: &[$(
                            ($entry_value, stringify!($entry_name)),
                        )*],
                    });
        }

        // Generate the Rust type.

        #[derive(Clone, Copy, Eq, PartialEq)]
        pub struct $name(pub $underlying);

        impl $name {
            $(
                pub const $entry_name: $name = $name($entry_value);
            )*

            pub const fn to_u32(self) -> u32 {
                self.0 as u32
            }
        }

        impl ::oot_explorer_read::FromVrom for $name {
            fn from_vrom(
                vrom: ::oot_explorer_vrom::Vrom<'_>,
                addr: ::oot_explorer_vrom::VromAddr,
            ) -> ::std::result::Result<Self, ::oot_explorer_read::ReadError> {
                ::oot_explorer_read::aligned_data::<Self>(vrom, addr)?;
                Ok(Self(<$underlying as ::oot_explorer_read::FromVrom>::from_vrom(vrom, addr)?))
            }
        }

        impl ::oot_explorer_read::Layout for $name {
            const SIZE: u32 = <$underlying as ::oot_explorer_read::Layout>::SIZE;
            const ALIGN_BITS: u32 = <$underlying as ::oot_explorer_read::Layout>::ALIGN_BITS;
        }

        compile_interfaces! { $($tail)* }
    };

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
            @parse Init { is_end: (None) }
            $($args)*
        }
    };
}
