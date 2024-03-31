use kmdparse::word::Word;

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Enum)]
pub enum KmdparseWord {
    /// An instruction, represented as 4 bytes. kmdparse handles flipping the bytes, so that
    /// instructions are the right way around.
    Instruction {
        instruction: [u8; 4],
    },
    Data {
        data: Vec<u8>,
    },
}

impl From<Word> for KmdparseWord {
    fn from(value: Word) -> Self {
        match value {
            Word::Instruction(instruction) => Self::Instruction { instruction },
            Word::Data(data) => Self::Data { data },
        }
    }
}
