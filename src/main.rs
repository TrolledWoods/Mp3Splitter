use std::i16;
use std::path::Path;
use std::fs::File;

mod meta_data;
use meta_data::{ TrackData, parse_track_data };

mod decode;
use decode::decode_file;

const FADE_IN_TIME: usize = 5000;

pub struct SoundData {
    sample_rate: u32,
    samples: Vec<(i16, i16)>
}

fn export_album_to_waves(
    album_name: &str, 
    folder: &Path, 
    tracks: &[TrackData],
    album_data: &SoundData) 
         -> Result<(), String> {

    for track in tracks.iter() {
        let name = format!("{} - {}", track.order, track.name.as_str());
        let mut path = std::path::PathBuf::from(folder.join(name.as_str()));
        path.set_extension("wav");

        let spec = hound::WavSpec {
            channels: 2,
            sample_rate: album_data.sample_rate,
            bits_per_sample: 16,
            sample_format: hound::SampleFormat::Int,
        };

        let start = track.start * album_data.sample_rate as usize;
        let end = match track.end {
            Some(end) => end * album_data.sample_rate as usize,
            None => album_data.samples.len()
        };

        if start >= end + FADE_IN_TIME * 2 {
            return Err(format!("Track {} '{}' begins after it ends(or is too short so that fading in/out doesn't work properly), which is invalid", track.order, &track.name));
        }

        println!("Exporting track no. {} to {:?}", track.order, path);
        let mut writer = hound::WavWriter::create(path, spec).map_err(|e| format!("{}", e))?;

        // Fading in
        for (i, (left, right)) in album_data.samples[start..(start + FADE_IN_TIME)].iter().enumerate() {
            let percent = i as f32 / FADE_IN_TIME as f32; 
            writer.write_sample((*left as f32 * percent) as i16)
                    .map_err(|e| format!("Track {} had an error while writing a sample to the left channel, '{}'", track.order, e))?;
            writer.write_sample((*right as f32 * percent) as i16)
                    .map_err(|e| format!("Track {} had an error while writing a sample to the right channel, '{}'", track.order, e))?;
        }

        // Normal sampling
        for (left, right) in &album_data.samples[(start + FADE_IN_TIME)..(end - FADE_IN_TIME)] {
            writer.write_sample(*left)
                    .map_err(|e| format!("Track {} had an error while writing a sample to the left channel, '{}'", track.order, e))?;
            writer.write_sample(*right)
                    .map_err(|e| format!("Track {} had an error while writing a sample to the right channel, '{}'", track.order, e))?;
        }

        // Fading out
        for (i, (left, right)) in album_data.samples[(end - FADE_IN_TIME)..end].iter().enumerate() {
            let percent = ((FADE_IN_TIME - i) as f32) / FADE_IN_TIME as f32;
            let percent = percent * percent;
            writer.write_sample((*left as f32 * percent) as i16)
                    .map_err(|e| format!("Track {} had an error while writing a sample to the left channel, '{}'", track.order, e))?;
            writer.write_sample((*right as f32 * percent) as i16)
                    .map_err(|e| format!("Track {} had an error while writing a sample to the right channel, '{}'", track.order, e))?;
        }

        writer.finalize().map_err(|e| format!("Track {} had an error while finalizing; '{}'", track.order, e))?;
    }

    Ok(())
}

fn read_line() -> String {
    let mut buffer = String::new();
    std::io::stdin().read_line(&mut buffer).unwrap();
    String::from(buffer.trim())
}


fn main() {
    use std::env;
    let mut args: Vec<String> = env::args().collect();

    let album_path = if args.len() >= 2 {
        args[1].clone()
    }else{
        println!("Please enter the album sound files path");
        read_line()
    };

    println!("Reading sound file");
    let album_data = decode_file(album_path.as_str()).unwrap();

    println!("Please give the timestamp information. Type 'finished' in the last line");
    let mut timestamp_info = String::new();
    loop {
        let addition = read_line();
        if addition == "finished" {
            break;
        }
        timestamp_info.push_str(addition.as_str());
        timestamp_info.push_str("\n");
    }
    let tracks = parse_track_data(timestamp_info.as_str()).unwrap();

    println!("Please give the path to the album folder(the folder should exist, or else an error will occur)");
    let path = read_line();

    println!("Exporting album to wav file");
    export_album_to_waves(album_path.as_str(), &Path::new(path.as_str()), &tracks[..], &album_data).unwrap();
}
