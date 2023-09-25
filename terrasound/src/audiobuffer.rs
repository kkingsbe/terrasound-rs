use rodio::{
    source::{SamplesConverter, Source},
    Decoder, Sink,
};

use std::{fs::File, io::BufReader};

use crate::{config::AUDIO_BUFFER_SIZE, metadata::Metadata};

#[derive(Debug, Clone)]
pub struct AudioBuffer {
    pub samples: Vec<f32>,
    pub metadata: Metadata,
}

impl AudioBuffer {
    pub fn play(&self, sink: &Sink) {
        let sample_rate = self.metadata.sample_rate as u32;
        let buff = rodio::buffer::SamplesBuffer::new(1, sample_rate, self.samples.clone());
        sink.append(buff);
        //sink.sleep_until_end();
    }

    pub fn from_resonator(buff: resonator::common::audiobuffer::AudioBuffer) -> AudioBuffer {
        AudioBuffer {
            samples: buff.samples,
            metadata: Metadata {
                sample_rate: buff.metadata.sample_rate,
                buffer_size: buff.metadata.buffer_size,
            },
        }
    }

    pub fn to_resonator(&self) -> resonator::common::audiobuffer::AudioBuffer {
        resonator::common::audiobuffer::AudioBuffer {
            samples: self.samples.clone(),
            metadata: resonator::server::metadata::Metadata {
                sample_rate: self.metadata.sample_rate,
                buffer_size: self.metadata.buffer_size,
            },
        }
    }

    /// Create a vector of `AudioBuffer` from a given file. Each of the buffers is exactly
    /// `AUDIO_BUFFER_SIZE` in length.
    pub fn buffers_from_file(filename: &str) -> Vec<Self> {
        let converter: SamplesConverter<_, f32> =
            Decoder::new(BufReader::new(File::open(filename).unwrap()))
                .unwrap()
                .convert_samples();
        let channels = converter.channels();
        let sample_rate = converter.sample_rate() as i32;

        assert_eq!(channels, 1, "Only single channel audio is supported");

        let flattened_samples: Vec<_> = converter.collect();

        flattened_samples
            .chunks_exact(AUDIO_BUFFER_SIZE)
            .map(|chunk| Self {
                samples: chunk.iter().copied().collect(),
                metadata: Metadata {
                    sample_rate,
                    buffer_size: AUDIO_BUFFER_SIZE as i32,
                },
            })
            .collect()
    }
}
