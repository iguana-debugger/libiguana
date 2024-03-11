use kmdparse::line::Line;

use super::word::KmdparseWord;

#[derive(Clone, Debug, PartialEq, Eq, uniffi::Record)]
pub struct KmdparseLine {
    pub memory_address: Option<u32>,
    pub word: Option<KmdparseWord>,
    pub comment: String,
}

impl From<Line> for KmdparseLine {
    fn from(value: Line) -> Self {
        Self {
            memory_address: value.memory_address,
            word: match value.word {
                Some(word) => Some(KmdparseWord::from(word)),
                None => None,
            },
            comment: value.comment,
        }
    }
}
