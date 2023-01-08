use ndarray::{s, Array, ArrayView, Dim};
use realfft::RealFftPlanner;
use std::fs::File;
use byteorder::{LittleEndian, ReadBytesExt};
use crate::project::{ProjectAudio, ProjectSpectrogram, ProjectImage};
use image::io::Reader;
use image::imageops::FilterType;

pub fn get_spectrogram_with_images(info: &ProjectAudio, spec: &ProjectSpectrogram, images: &[ProjectImage]) -> Option<Vec<u8>> {
    let filename = format!("projects/{}", info.file);
    let num_steps = spec.width.unwrap();
    let num_freqs = spec.height;
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
    let mut result:Array<u8,_> = Array::zeros((wanted_channels, spec.full_height, spec.width.unwrap(), 4));

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

            let mut values = output_sq.mapv(|e|(e.powf(0.25) * 32.) as u8).to_vec();
            values.reverse();
            let values = Array::from_vec(values);

            result.slice_mut(s![i, 0..num_freqs, j, 0]).assign(&values);
            result.slice_mut(s![i, 0..num_freqs, j, 1]).assign(&values);
            result.slice_mut(s![i, 0..num_freqs, j, 2]).assign(&values);
            result.slice_mut(s![i, 0..num_freqs, j, 3]).assign(&values);

            prev_sq.assign(&output_sq);
        }
    }

    for image in images {
        let x0 = (((image.start - 1.0) * info.bar_length + info.bar_offset) * (info.rate as f32) / (spec.samples_per_step as f32)) as usize;
        let x1 = ((image.end * info.bar_length + info.bar_offset) * (info.rate as f32) / (spec.samples_per_step as f32)) as usize;
        let data = get_image(&image.file, x1 - x0)?;
        let height = data.shape()[0];
        let width = data.shape()[1];
        result.slice_mut(s![0, spec.height..spec.height + height, x0..x0 + width, ..]).assign(&data);
    }

    Some(result.into_shape((wanted_channels * spec.full_height * spec.width.unwrap() * 4,)).expect("into_shape failure").to_vec())
}

fn get_image(name: &str, desired_width: usize) -> Option<Array<u8,Dim<[usize;3]>>> {
    let image = Reader::open(format!("projects/{name}")).ok()?.decode().ok()?;
    let width = image.width();
    let height = image.height();
    let image = image.resize(desired_width as u32, ((desired_width as u64) * (height as u64) / (width as u64)) as u32, FilterType::Triangle);
    let width = image.width();
    let height = image.height();
    let data:Vec<u8> = image.into_rgba8().into_vec();
    Array::from_vec(data).into_shape((height as usize, width as usize, 4usize)).ok()
}
