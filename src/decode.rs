use crate::SoundData;
use std::path::Path;
use std::fs::File;

pub fn decode_file(path: impl AsRef<Path>) -> Result<SoundData, String> {
    let path = path.as_ref();
    match path.extension().map(|v| v.to_str()).flatten() {
        Some("mp3") => decode_mp3_file(path),
        None => Err(format!("The sound file must have an extension")),
        _ => Err(format!("Currently the extension '{}' is not supported", path.extension().unwrap().to_str().unwrap()))
    }
}

pub fn decode_mp3_file(path: impl AsRef<Path>) -> Result<SoundData, String> {
    use simplemad::{ Decoder, Frame };
    let file = File::open(path).map_err(|e| format!("{:?}", e))?;
    let decoder = Decoder::decode(file).map_err(|e| format!("{:?}", e))?;

    println!("There is no need to panic if you see some errors, it is perfectly normal to see a few");

    let mut sample_rate = None;
    let mut samples = Vec::new();

    const SECONDS_PER_NOTICE: u32 = 60;
    let mut samples_left = 0;
    let mut n_seconds = 0;
    for decoding_result in decoder {
        match decoding_result {
            Err(e) => println!("DECODING ERROR: {:?}", e),
            Ok(frame) => {
                match sample_rate {
                    None => {
                        sample_rate = Some(frame.sample_rate);
                        samples_left = frame.sample_rate * SECONDS_PER_NOTICE;
                    },
                    Some(current_sample_rate) => if frame.sample_rate != current_sample_rate {
                        return Err(format!("Currently mp3:s with changing sample rates are not supported"));
                    }
                }

                if frame.samples.len() == 1 {
                    for sample in frame.samples[0].iter() {
                        samples.push((sample.to_i16(), sample.to_i16()));
                        samples_left -= 1;
                    }
                }else if frame.samples.len() == 2 {
                    for (left, right) in frame.samples[0].iter().zip(frame.samples[1].iter()) {
                        samples.push((left.to_i16(), right.to_i16()));
                        samples_left -= 1;
                    }
                }else{
                    return Err(format!("Unsupported number of samples; {}", frame.samples.len()));
                }

                while samples_left <= 0 {
                    samples_left += SECONDS_PER_NOTICE * sample_rate.unwrap();
                    n_seconds += SECONDS_PER_NOTICE;
                    println!("{} seconds processed", n_seconds);
                }
            }
        }
    }

    if let Some(sample_rate) = sample_rate {
        Ok(SoundData {
            sample_rate: sample_rate,
            samples: samples
        })
    }else{
        Err(format!("Couldn't get a single frame of information, 
                    something is wrong with your mp3, or it's not supported"))
    }
}
