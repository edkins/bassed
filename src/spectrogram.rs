use ndarray::{s, Array, ArrayView, ArrayViewMut};
use realfft::RealFftPlanner;
use std::fs::File;
use std::io::{Seek, SeekFrom};
use byteorder::{LittleEndian, ReadBytesExt};
use crate::project::{ProjectAudio, ProjectSpectrogram};

pub fn get_spectrogram(info: &ProjectAudio, spec: &ProjectSpectrogram, index: usize) -> Option<Vec<u8>> {
    let filename = format!("projects/{}", info.file);
    let num_steps = spec.height;
    let num_freqs = spec.width;
    let num_samples = num_steps * spec.samples_per_step + (spec.samples_per_fft - spec.samples_per_step);
    let start_pos = info.channels * 4 * num_steps * spec.samples_per_step * index;
    let len = info.channels * 4 * num_samples;
    let mut file = File::open(filename).ok()?;
    file.seek(SeekFrom::Start(start_pos as u64)).ok()?;
    let mut f32_vec:Vec<f32> = vec![0.0; len / 4];
    file.read_f32_into::<LittleEndian>(&mut f32_vec).ok()?;
    let array = Array::from_vec(f32_vec).into_shape((num_samples, info.channels)).ok()?;

    let fft = RealFftPlanner::<f32>::new().plan_fft_forward(spec.samples_per_fft);
    let mut fft_input = fft.make_input_vec();
    let mut fft_output = fft.make_output_vec();
    assert_eq!(fft_input.len(), spec.samples_per_fft);
    assert!(fft_output.len() >= num_freqs);

    let mut result:Array<u8,_> = Array::zeros((info.channels, spec.height, spec.width, 4));

    for i in 0..info.channels {
        for j in 0..num_steps {
            let offset = j * spec.samples_per_step;
            let mut input_view = ArrayViewMut::from_shape((spec.samples_per_fft,), &mut fft_input).expect("ArrayViewMut failure");
            input_view.assign(&array.slice(s![offset..offset+spec.samples_per_fft, i]));
            fft.process(&mut fft_input, &mut fft_output).expect("FFT failed for some reason");
            let output_view = ArrayView::from_shape((fft_output.len(),), &fft_output).expect("ArrayView failure");
            let output_view = output_view.slice(s![..num_freqs]);

            let output_cx = output_view.split_complex();
            let output_re = output_cx.re.into_owned();
            let output_im = output_cx.im.into_owned();
            let values = (output_re.clone() * output_re + output_im.clone() * output_im).mapv(|e|e as u8);

            result.slice_mut(s![i, j, .., 0]).assign(&values);
            result.slice_mut(s![i, j, .., 1]).assign(&values);
            result.slice_mut(s![i, j, .., 2]).assign(&values);
            result.slice_mut(s![i, j, .., 3]).fill(255);
        }
    }
    Some(result.into_shape((info.channels * spec.height * spec.width * 4,)).expect("into_shape failure").to_vec())
}
