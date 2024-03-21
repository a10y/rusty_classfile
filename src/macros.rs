/// Helper macro to create a "reversible enum". Reversible enums are
/// * Field-less
/// * Contain only a discriminant
/// * Have a primitive representation
/// * Implement the `TryFrom<$ty>` trait to allow for easy conversions from the primitive type
macro_rules! reversible_enum {
    ($name:ident as $ty:ty, {
        $($key:ident = $val:literal,)*
    }) => {
        #[repr($ty)]
        pub enum $name {
            $($key = $val),*
        }

        impl TryFrom<$ty> for $name {
            type Error = Error;
            fn try_from(value: $ty) -> Result<Self, Self::Error> {
                match value {
                    $($val => Ok($name::$key),)*
                    _ => Err(Self::Error::InvalidConstantPoolItemTag(value)),
                }
            }
        }
    };
}

macro_rules! read_bytes {
    ($self:expr, $ty:ty, $N:expr) => {{
        let mut buf = [0u8; $N];
        $self.read_exact(&mut buf)?;

        Ok(<$ty>::from_be_bytes(buf))
    }};
}
