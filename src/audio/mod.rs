extern crate bus;
extern crate cpal;

mod track;

use self::bus::{BusReader};
use self::track::AudioTrack;
use self::cpal::{SampleFormat, StreamData, EventLoop, UnknownTypeOutputBuffer};

// initialize audio machinery
pub fn initialize_audio(midi_rx: BusReader<::midi::CommandMessage>) {

  // init our beautiful test audiotrack
  let mut audio_track = AudioTrack::new(midi_rx);
  audio_track.load_file("/Users/nunja/Documents/Audiolib/smplr/loop16.wav");

  // init audio with CPAL !
  // creates event loop
  let event_loop = EventLoop::new();

  // audio out device
  let device = cpal::default_output_device().expect("audio: no output device available");

  // supported formats is an iterator
  let mut supported_formats_range = device.supported_output_formats()
    .expect("audio: error while querying formats");
  
  let format = supported_formats_range.next()
    .expect("audio: No supported format.")
    .with_max_sample_rate();

  // display some info
  println!("audio: Default OUTPUT Samplerate: {}", format.sample_rate.0);
  match format.data_type {
    SampleFormat::U16 => println!("audio: Supported sample type is U16"),
    SampleFormat::I16 => println!("audio: Supported sample type is I16"),
    SampleFormat::F32 => println!("audio: Supported sample type is F32")
  }   

  // creates the stream
  let stream_id = event_loop.build_output_stream(&device, &format).unwrap();

  // add stream
  event_loop.play_stream(stream_id);

  // audio callback
  event_loop.run(move |_stream_id, stream_data| {
      match stream_data {
          StreamData::Output { buffer: UnknownTypeOutputBuffer::F32(mut buffer) } => {
              for elem in buffer.iter_mut() {
                  match audio_track.next() {
                    Some(sample) => {
                      // println!("sample: {}", sample);
                      *elem = sample * 0.5;
                    },
                    None => {
                      *elem = 0.0; // finish
                    }
                  }
              }
          },
          _ => (),
      }
  });
}