use kmdparse::token::Token;

use super::{label::KmdparseLabel, line::KmdparseLine};

#[derive(Clone, Debug, PartialEq, uniffi::Enum)]
pub enum KmdparseToken {
    Tag,
    Line { line: KmdparseLine },
    Label { label: KmdparseLabel },
}

impl From<Token> for KmdparseToken {
    fn from(value: Token) -> Self {
        match value {
            Token::Tag => Self::Tag,
            Token::Line { line } => Self::Line {
                line: KmdparseLine::from(line),
            },
            Token::Label { label } => Self::Label {
                label: KmdparseLabel::from(label),
            },
        }
    }
}
