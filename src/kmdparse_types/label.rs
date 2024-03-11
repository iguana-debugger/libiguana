use kmdparse::label::Label;

#[derive(Clone, Debug, PartialEq, uniffi::Record)]
pub struct KmdparseLabel {
    /// The name of the label
    pub name: String,

    /// The associated memory address of the label
    pub memory_address: u32,

    /// Whether or not the label is global (true for global, false for local)
    pub is_exported: bool,

    /// Whether or not the label points to a Thumb instruction
    pub is_thumb: bool,
}

// #[cfg_attr(feature = "uniffi", uniffi::export)]
impl From<Label> for KmdparseLabel {
    fn from(value: Label) -> Self {
        KmdparseLabel {
            name: value.name,
            memory_address: value.memory_address,
            is_exported: value.is_exported,
            is_thumb: value.is_thumb,
        }
    }
}
