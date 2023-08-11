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
}