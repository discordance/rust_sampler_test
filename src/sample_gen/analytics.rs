extern crate num;
extern crate sample;
extern crate time_calc;
extern crate aubio;

use std::path::Path;

use self::aubio::onset::Onset;
use self::aubio::tempo::Tempo;
use self::num::ToPrimitive;
use self::time_calc::{Samples};


// consts
const HOP_SIZE: usize = 512;
const WIND_SIZE: usize = 2048;
const SR: usize = 44_100;

// Parse the original tempo based on the beat value written in the filename
fn parse_filepath_beats(path: &str) -> Result<u64, &str> {
  // compute path
  let path_obj = Path::new(path);
  let file_stem = match path_obj.file_stem() {
    Some(fstem) => fstem,
    None => return Err("NoFileName"),
  };
  let file_stem = match file_stem.to_str() {
    Some(s) => s,
    None => return Err("NoFileName"),
  };
  let split: Vec<&str> = file_stem.split("_").collect();
  match split.last() {
    Some(last) => {
      match last.parse::<u64>() {
        Ok(b) => return Ok(b),
        Err(err) => return Err("ParseIntError"),
      };
    }
    None => return Err("NoBeatNum"),
  }
}

/// Get the original tempo based on the beat value written in the filename, or analized with Aubio if not present.
/// Returns original tempo as computed from file name and the number of beats
/// @TODO the aubio part
pub fn get_original_tempo(path: &str, num_samples: usize) -> (f64, u64) {
  // compute number of beats
  let num_beats = match parse_filepath_beats(path) {
    Ok(n) => n,
    Err(err) => {
      println!("Can't parse beats on the filename {}", err);
      // default to 4 (one bar)
      4
    }
  };
  let ms = Samples((num_samples as i64) / 2).to_ms(44_100.0);

  let secs = ms.to_f64().unwrap() / 1000.0;
  return (60.0 / (secs / num_beats as f64), num_beats);
}

/// Onset detector via Aubio.
pub fn detect_onsets(samples: &[f32]) -> Vec<u64> {

  let len = samples.len() / 2;
  let mono: Vec<f32> = samples.iter().step_by(2).map(|x| *x).collect();
  let mut chunk_iter = mono.chunks(HOP_SIZE);

  // onset
  let mut onset = Onset::new(WIND_SIZE, HOP_SIZE, SR).expect("Onset::new");

  // params
  onset.set_threshold(0.9);
  onset.set_silence(-40.0);
  onset.set_minioi(0.005);

  // save position in seconds (we can get that in samples later)
  let mut positions: Vec<u64> = Vec::new();
  // zero by default
  positions.push(0);
  // track
  let mut latest_detection = 0;

  loop {
    let next = chunk_iter.next();
    match next {
      Some(chunk) => {
        // break the fft
        if chunk.len() != HOP_SIZE {
          break;
        }
        onset.execute(&chunk);
        let mut detected = onset.last_onset();
        if latest_detection < detected {
          positions.push(detected as u64);
          latest_detection = detected;
        }
      }
      None => break,
    }
  }
  // push the len as last position
  positions.push(len as u64);
  // return
  positions
}

/// bpm detector via aubio.
pub fn detect_bpm(samples: &[f32]) -> f64 {
  // mono version
  let mono: Vec<f32> = samples.iter().step_by(2).map(|x| *x).collect();
  let mut chunk_iter = mono.chunks(HOP_SIZE); // by chunk
  let mut tempo = Tempo::new(WIND_SIZE, HOP_SIZE, SR).expect("Tempo::new");
  let mut detected_tempo = 120.0;

  loop {
    let next = chunk_iter.next();
    match next {
      Some(chunk) => {
        // break the fft
        if chunk.len() != HOP_SIZE {
          break;
        }
        tempo.execute(&chunk);
        match tempo.bpm() {
          Some(tempo) => detected_tempo = tempo,
          None => (),
        }
      }
      None => break,
    }
  }

  println!("analysis: detected tempo: {}", detected_tempo);

  // return
  detected_tempo as f64
}

/// Basic division onsets position.
pub fn slice_onsets(len: usize, divisor: usize) -> Vec<u64> {
  let step = len/divisor;
  let mut positions = Vec::new();
  for x in 0..divisor {
    positions.push((x*step) as u64);
  }
  positions.push(len as u64);
  return positions
}

/// Quantize a position vector to factor `multiple`
pub fn quantize_pos(d: &Vec<u64>, multiple: u64) -> Vec<u64> {
  let mut new_pos = Vec::new();
  for pos in d.iter() {
    let q = (*pos as f32 / multiple as f32).round() * multiple as f32;
    new_pos.push(q as u64);
  }
  new_pos
}