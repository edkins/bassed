use std::fs::File;
use symphonia::core::audio::AudioBufferRef;
use symphonia::core::codecs::DecoderOptions;
use symphonia::core::formats::{FormatOptions, SeekMode, SeekTo};
use symphonia::core::io::{MediaSourceStream, MediaSourceStreamOptions};
use symphonia::core::meta::MetadataOptions;
use symphonia::core::probe::Hint;

#[derive(Debug)]
pub enum AudioError {
    BadProject,
    NoAudioFile,
    FileError,
    ProbeError,
    AmbiguousTrack,
    ChannelsError,
    RegistryError(String),
    SeekError,
    ReadPacketError,
    DecodeError,
    ChannelMismatch,
}

fn get_buffer_data(num_channels: usize, buffer: AudioBufferRef) -> Result<Vec<Vec<f32>>, AudioError> {
    if buffer.spec().channels.count() != num_channels {
        return Err(AudioError::ChannelMismatch);
    }
    match buffer {
        AudioBufferRef::U8(_) => println!("U8"),
        AudioBufferRef::U16(_) => println!("U16"),
        AudioBufferRef::U24(_) => println!("U24"),
        AudioBufferRef::U32(_) => println!("U32"),
        AudioBufferRef::S8(_) => println!("S8"),
        AudioBufferRef::S16(_) => println!("S16"),
        AudioBufferRef::S24(_) => println!("S24"),
        AudioBufferRef::S32(_) => println!("S32"),
        AudioBufferRef::F32(_) => println!("F32"),
        AudioBufferRef::F64(_) => println!("F64"),
    }
    Ok(vec![vec![]; num_channels])
}

pub fn get_audio_data(file: &str, start: u64, len: u64) -> Result<Vec<Vec<f32>>, AudioError> {
    let codec_registry = symphonia::default::get_codecs();
    let probe = symphonia::default::get_probe();
    let file = File::open(file).map_err(|_|AudioError::FileError)?;
    let media_source_stream = MediaSourceStream::new(Box::new(file), MediaSourceStreamOptions{ buffer_len: 1024*1024 });
    let mut probe_result = probe.format(
        &Hint::default(),
        media_source_stream,
        &FormatOptions::default(),
        &MetadataOptions::default(),
    ).map_err(|_|AudioError::ProbeError)?;
    let tracks = probe_result.format.tracks();
    if tracks.len() != 1 {
        return Err(AudioError::AmbiguousTrack);
    }
    let num_channels = tracks[0].codec_params.channels.ok_or(AudioError::ChannelsError)?.count();
    let mut decoder = codec_registry.make(&tracks[0].codec_params, &DecoderOptions::default()).map_err(|e|AudioError::RegistryError(format!("{:?}", e)))?;
    probe_result.format.seek(SeekMode::Accurate, SeekTo::TimeStamp{ ts: start, track_id: tracks[0].id }).map_err(|_|AudioError::SeekError)?;
    let mut result = vec![vec![];num_channels];
    loop {
        let packet = probe_result.format.next_packet().map_err(|_|AudioError::ReadPacketError)?;
        let audio_buffer = decoder.decode(&packet).map_err(|_|AudioError::DecodeError)?;
        let data = get_buffer_data(num_channels, audio_buffer)?;
        for i in 0..num_channels {
            result[i].extend_from_slice(&data[i]);
        }
        if packet.ts + packet.dur >= start + len {
            break;
        }
    }

    Ok(result)
}
