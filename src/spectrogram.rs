use ndarray::{s, Array, ArrayView, ArrayViewMut};
use realfft::RealFftPlanner;
use std::cmp::Ordering;
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

    for i in 0..info.channels {
        let mut last_sq:Array<f32,_> = Array::zeros((num_freqs,));
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
            let output_sq = output_re.clone() * output_re.clone() + output_im.clone() * output_im.clone();

            let mut values_b = vec![0u8; num_freqs];
            for i in 0..num_freqs {
                let mut values = vec![0f32; 4];
                for j in 0..4 {
                    let k = (i * (j + 1)) / 4;
                    values[j] = output_sq[k];
                }
                values.sort_unstable_by(|a,b|if a<b {Ordering::Less} else {Ordering::Greater});
                values_b[i] = (values[1].powf(0.33) * 16.) as u8;
            }

            let values_r = output_sq.mapv(|e|(e.powf(0.33) * 16.) as u8);
            let values_g = values_r.clone();
            let values_b = Array::from_vec(values_b);

            result.slice_mut(s![i, j, .., 0]).assign(&values_r);
            result.slice_mut(s![i, j, .., 1]).assign(&values_g);
            result.slice_mut(s![i, j, .., 2]).assign(&values_b);
            result.slice_mut(s![i, j, .., 3]).fill(255);

            last_sq.assign(&output_sq);
        }
    }
    Some(result.into_shape((info.channels * spec.height.unwrap() * spec.width * 4,)).expect("into_shape failure").to_vec())
}
