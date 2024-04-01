use thiserror::Error;
use yaxpeax_arch::{Decoder, ReaderBuilder};
use yaxpeax_arm::armv7::{ARMv7, DecodeError};

/// A wrapper around yaxpeax's `DecoderError` so that it can be passed through uniffi
#[derive(Debug, Error, uniffi::Error)]
#[uniffi(flat_error)]
pub enum DecoderError {
    #[error("An error occured when decoding an instruction: {0}")]
    DecodeError(#[from] DecodeError),
}

/// Decodes the given word into an ARM instruction. I'd rather have this contained in a struct, but
/// uniffi doesn't like associated functions.
#[uniffi::export]
pub fn decode_instruction(word: u32) -> Result<String, DecoderError> {
    let word_bytes: &[u8] = &word.to_le_bytes();

    let mut reader = ReaderBuilder::<u32, u8>::read_from(word_bytes);

    let decoder = <ARMv7 as yaxpeax_arch::Arch>::Decoder::default(); // what a line

    let instruction_string = decoder
        .decode(&mut reader)
        .map(|instruction| instruction.to_string())?;

    Ok(instruction_string)
}
