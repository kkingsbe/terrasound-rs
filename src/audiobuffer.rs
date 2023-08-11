use rodio::{Sink, Source};

use crate::metadata::Metadata;

#[derive(Debug, Clone)]
pub struct AudioBuffer {
    pub samples: Vec<f32>,
    pub metadata: Metadata,
}

impl AudioBuffer {
    pub fn play(&self, sink: &Sink) {
        let sample_rate = self.metadata.sample_rate as u32;
        let source = rodio::buffer::SamplesBuffer::new(1, sample_rate, self.samples.clone()).amplify(0.9);
        sink.append(source);
        sink.sleep_until_end();
    }
}