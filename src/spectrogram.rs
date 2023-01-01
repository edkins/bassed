use crate::codecs::{self, AudioError};

const SPECTROGRAM_SIZE:u64 = 1024 * 1024;

pub fn get_spectrogram(file: &str, i: u64) -> Result<Vec<u8>, AudioError> {
    let data = codecs::get_audio_data(file, i * SPECTROGRAM_SIZE, SPECTROGRAM_SIZE)?;
    Ok(vec![])
}
