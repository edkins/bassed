use ndarray::{s, Array, ArrayView, ArrayViewMut};
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
    let mut fft_input = fft.make_input_vec();
    let mut fft_output = fft.make_output_vec();
    assert_eq!(fft_input.len(), spec.samples_per_fft);
    assert!(fft_output.len() >= num_freqs);

    let mut result:Array<u8,_> = Array::zeros((info.channels, spec.height.unwrap(), spec.width, 4));

    let bands = (spec.samples_per_step as f32) / (spec.samples_per_fft as f32) * 6.283185307179586;
    let cos = Array::from_iter((0..num_freqs).map(|t|(t as f32 * bands).cos()));
    let sin = Array::from_iter((0..num_freqs).map(|t|(t as f32 * bands).sin()));

    for i in 0..info.channels {
        let mut prev_out_re:Array<f32,_> = Array::zeros((num_freqs,));
        let mut prev_out_im:Array<f32,_> = Array::zeros((num_freqs,));
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
            let values_r = (output_re.clone() * output_re.clone() + output_im.clone() * output_im.clone()).mapv(|e|(e.powf(0.33) * 16.) as u8);
            let values_g = values_r.clone();

            let diff_re = prev_out_re.clone() * cos.clone() - prev_out_im.clone() * sin.clone() - output_re.clone();
            let diff_im = prev_out_re.clone() * sin.clone() + prev_out_im.clone() * cos.clone() - output_im.clone();

            let values_b = (diff_re.clone() * diff_re.clone() + diff_im.clone() * diff_im.clone()).mapv(|e|(e.powf(0.33) * 32.) as u8);

            result.slice_mut(s![i, j, .., 0]).assign(&values_r);
            result.slice_mut(s![i, j, .., 1]).assign(&values_g);
            result.slice_mut(s![i, j, .., 2]).assign(&values_b);
            result.slice_mut(s![i, j, .., 3]).fill(255);

            prev_out_re.assign(&output_re);
            prev_out_im.assign(&output_im);
        }
    }
    Some(result.into_shape((info.channels * spec.height.unwrap() * spec.width * 4,)).expect("into_shape failure").to_vec())
}
