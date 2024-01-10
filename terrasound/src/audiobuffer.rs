use rodio::Sink;

use crate::metadata::Metadata;

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
                buffer_size: buff.metadata.buffer_size
            }
        }
    }

    pub fn to_resonator(&self) -> resonator::common::audiobuffer::AudioBuffer {
        resonator::common::audiobuffer::AudioBuffer {
            samples: self.samples.clone(),
            metadata: resonator::common::metadata::Metadata {
                sample_rate: self.metadata.sample_rate,
                buffer_size: self.metadata.buffer_size
            }
        }
    }
}