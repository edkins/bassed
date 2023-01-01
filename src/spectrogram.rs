use ndarray::{Array, Axis};
use std::fs::File;
use std::io::{Seek, SeekFrom};
use byteorder::{LittleEndian, ReadBytesExt};
use crate::project::ProjectAudio;

const SPECTROGRAM_SIZE:usize = 1024 * 1024;

pub fn get_spectrogram(info: &ProjectAudio, i: usize) -> Option<Vec<u8>> {
    let filename = format!("projects/{}", info.file);
    let start_pos = info.channels * 4 * SPECTROGRAM_SIZE * i;
    let len = info.channels * 4 * SPECTROGRAM_SIZE;
    let mut file = File::open(filename).ok()?;
    file.seek(SeekFrom::Start(start_pos as u64)).ok()?;
    let mut f32_vec:Vec<f32> = vec![0.0; len / 4];
    file.read_f32_into::<LittleEndian>(&mut f32_vec).ok()?;
    let array = Array::from_vec(f32_vec).into_shape((info.channels, SPECTROGRAM_SIZE)).ok()?;
    for i in 0..info.channels {
        let channel_vec = array.index_axis(Axis(0), i).to_vec();
    }
    Some(vec![])
}
