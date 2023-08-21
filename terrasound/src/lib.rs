pub mod config;
mod audiosource;
mod audiobuffer;
mod metadata;
mod audiosocketserver;

use std::{sync::{Arc, Mutex}, thread};
use audiosocketserver::AudioSocketServer;
use audiosource::AudioSource;
use config::PREBUFFER_SIZE;
use rodio::{OutputStream, Sink, OutputStreamHandle};

pub struct Terrasound {
    stream: OutputStream,
    stream_handle: OutputStreamHandle,
    audio_source: Arc<Mutex<AudioSource>>,
    running: Arc<Mutex<bool>>,
    server: AudioSocketServer,
}

impl Terrasound {
    pub fn new(port: u16) -> Terrasound {
        let (_stream, _stream_handle) = OutputStream::try_default().unwrap();

        Terrasound {
            stream: _stream,
            stream_handle: _stream_handle.clone(),
            audio_source: Arc::new(Mutex::new(AudioSource { 
                buffers: Vec::new(),
                current_buffer_idx: 0,
                sink: Sink::try_new(&_stream_handle.clone()).unwrap()
            })),
            running: Arc::new(Mutex::new(true)),
            server: AudioSocketServer::new(port),
        }
    }

    pub fn start(&mut self) {
        let audio_source = self.audio_source.clone();
        self.server.begin(self.audio_source.clone(), self.running.clone());

        let running = self.running.clone();
        thread::spawn(move || {
            let mut num_buffers: usize = 0;
            let mut playing = false;

            while running.lock().unwrap().clone() == true {
                let mut temp_audio_source = audio_source.lock().unwrap();
                let current_prebuffer_size = num_buffers - temp_audio_source.current_buffer_idx;
                let new_num_buffers = temp_audio_source.buffers.len();

                //If the prebuffer is large enough, begin playback
                if !playing && current_prebuffer_size >= PREBUFFER_SIZE {
                    playing = true;
                }
                
                if playing {
                    if temp_audio_source.current_buffer_idx >= new_num_buffers {
                        continue;
                    }

                    temp_audio_source.play_next();
                }
                num_buffers = new_num_buffers;
            }
        });
    }
}

impl Drop for Terrasound {
    fn drop(&mut self) {
        *self.running.lock().unwrap() = false;
    }
}