use std::sync::{Mutex, Arc};

use resonator::common::audiobuffer::AudioBuffer as ResonatorAudioBuffer;
use rodio::Sink;

use crate::{TerrasoundSource, audiobuffer::AudioBuffer, audiosource::AudioSourceError};

pub struct LocalAudioSource {
    buffers: Arc<Mutex<Vec<ResonatorAudioBuffer>>>,
    pub sink: Sink,
    pub current_buffer_idx: usize
}

impl LocalAudioSource {
    pub fn new(buffers: Arc<Mutex<Vec<ResonatorAudioBuffer>>>, sink: Sink) -> LocalAudioSource {
        LocalAudioSource {
            buffers,
            sink,
            current_buffer_idx: 0
        }
    }

    pub fn num_buffers(&self) -> usize {
        self.buffers.lock().unwrap().len()
    }
}

impl TerrasoundSource for LocalAudioSource {
    fn get_next(&mut self) -> Result<AudioBuffer, AudioSourceError> {
        let temp_buffers = self.buffers.lock().unwrap().clone();
        if temp_buffers.len() == 0 {
            return Err(AudioSourceError::NoMoreAudio);
        }

        let temp_buffer = self.buffers.lock().unwrap().remove(0);
        Ok(AudioBuffer::from_resonator(temp_buffer))
    }

    fn play_next(&mut self) {
        let next_buffer = self.get_next().unwrap().clone();
        let sink = &self.sink;
        next_buffer.play(sink);
    }
}