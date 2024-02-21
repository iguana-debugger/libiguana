use crate::UniffiCustomTypeConverter;

// uniffi::custom_type! hates giving [u8; 4] directly for some reason so I made this type
type U84Arr = [u8; 4];

uniffi::custom_type!(U84Arr, u32);

impl UniffiCustomTypeConverter for U84Arr {
    type Builtin = u32;

    fn into_custom(val: Self::Builtin) -> uniffi::Result<Self> {
        Ok(val.to_le_bytes())
    }

    fn from_custom(obj: Self) -> Self::Builtin {
        u32::from_le_bytes(obj)
    }
}
