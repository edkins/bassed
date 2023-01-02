use ndarray::{s, Array, ArrayView};
use realfft::RealFftPlanner;
use std::fs::File;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::project::{ProjectAudio, ProjectSpectrogram};

pub fn get_spectrogram(info: &ProjectAudio, spec: &ProjectSpectrogram) -> Option<Vec<u8>> {
    let filename = format!("projects/{}", info.file);
    let num_steps = spec.height.unwrap();
    let num_freqs = spec.width;
    let num_samples = info.length.unwrap();
    let len = info.channels * 4 * num_samples;
    let mut file = File::open(filename).ok()?;
    let mut f32_vec:Vec<f32> = vec![0.0; len / 4];
    file.read_f32_into::<LittleEndian>(&mut f32_vec).ok()?;
    let array = Array::from_vec(f32_vec).into_shape((num_samples, info.channels)).ok()?;

    let fft = RealFftPlanner::<f32>::new().plan_fft_forward(spec.samples_per_fft);
    let mut fft_output = fft.make_output_vec();
    assert!(fft_output.len() >= num_freqs);

    let wanted_channels = 1;
    let mut result:Array<u8,_> = Array::zeros((wanted_channels, spec.height.unwrap(), spec.width, 4));

    let mut gaussian:Array<f32,_> = Array::zeros((spec.samples_per_fft,));
    for i in 0..spec.samples_per_fft {
        let x = ((i as f32) / (spec.samples_per_fft as f32) - 0.5) * 4.0;
        gaussian[i] = (-x * x).exp();
    }

    let mut prev_sq = Array::zeros((num_freqs,));
    for i in 0..wanted_channels {
        for j in 0..num_steps {
            let offset = j * spec.samples_per_step;

            let input_array = array.slice(s![offset..offset+spec.samples_per_fft, i]).to_owned();
            let input_array = input_array.clone() * gaussian.clone();
            fft.process(&mut input_array.to_vec(), &mut fft_output).expect("FFT failed for some reason");
            let output_view = ArrayView::from_shape((fft_output.len(),), &fft_output).expect("ArrayView failure");
            let output_view = output_view.slice(s![..num_freqs]);

            let output_cx = output_view.split_complex();
            let output_re = output_cx.re.into_owned();
            let output_im = output_cx.im.into_owned();
            let output_sq = output_re.clone() * output_re.clone() + output_im.clone() * output_im.clone();

            let values_g = output_sq.mapv(|e|(e.powf(0.25) * 32.) as u8);

            result.slice_mut(s![i, j, .., 0]).assign(&values_g);
            result.slice_mut(s![i, j, .., 1]).assign(&values_g);
            result.slice_mut(s![i, j, .., 2]).assign(&values_g);
            result.slice_mut(s![i, j, .., 3]).assign(&values_g);

            prev_sq.assign(&output_sq);
        }
    }
    Some(result.into_shape((wanted_channels * spec.height.unwrap() * spec.width * 4,)).expect("into_shape failure").to_vec())
}
