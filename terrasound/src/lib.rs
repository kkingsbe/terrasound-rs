pub mod config;
mod audiosource;
mod audiobuffer;
mod metadata;
mod audiosocketserver;
mod localaudiosource;

use std::{sync::{Arc, Mutex}, thread};
use audiosocketserver::AudioSocketServer;
use audiosource::AudioSource;
use config::PREBUFFER_SIZE;
use localaudiosource::LocalAudioSource;
use rodio::{OutputStream, Sink, OutputStreamHandle};

pub trait TerrasoundSource {
    fn get_next(&mut self) -> Result<audiobuffer::AudioBuffer, audiosource::AudioSourceError>;
    fn play_next(&mut self);
}

#[derive(PartialEq, Clone)]
pub enum TerrasoundMode {
    PortInResonatorOut,
    ResonatorInSpeakerOut,
    PortInSpeakerOut
}

pub struct Terrasound {
    stream: OutputStream,
    stream_handle: OutputStreamHandle,
    audio_source: Option<Arc<Mutex<AudioSource>>>, //Must be specified if recieving audio over a port
    running: Arc<Mutex<bool>>,
    server: Option<AudioSocketServer>, //Used for recieving audio over a port
    local_source: Option<Arc<Mutex<LocalAudioSource>>>, //Used for recieving audio from a local source
    incoming_data: Option<Arc<Mutex<Vec<resonator::common::audiobuffer::AudioBuffer>>>>,
    outgoing_data: Option<Arc<Mutex<Vec<resonator::common::audiobuffer::AudioBuffer>>>>,
    mode: Arc<Mutex<TerrasoundMode>>
}

impl Terrasound {
    pub fn new_port_in_resonator_out(port: u16, outgoing_data: Arc<Mutex<Vec<resonator::common::audiobuffer::AudioBuffer>>>) -> Terrasound {
        Terrasound::new(port, TerrasoundMode::PortInResonatorOut, None, Some(outgoing_data))
    }

    pub fn new_port_in_speaker_out(port: u16) -> Terrasound {
        Terrasound::new(port, TerrasoundMode::PortInSpeakerOut, None, None)
    }

    pub fn new_resonator_in_speaker_out(incoming_data: Arc<Mutex<Vec<resonator::common::audiobuffer::AudioBuffer>>>) -> Terrasound {
        Terrasound::new(0, TerrasoundMode::ResonatorInSpeakerOut, Some(incoming_data), None)
    }

    /**
     * Initializes the Terrasound library on the specified port. If resonator_sink is provided, then audio will be written to that buffer instead of played
     */
    fn new(port: u16, mode: TerrasoundMode, incoming_data: Option<Arc<Mutex<Vec<resonator::common::audiobuffer::AudioBuffer>>>>, outgoing_data: Option<Arc<Mutex<Vec<resonator::common::audiobuffer::AudioBuffer>>>>) -> Terrasound {
        let (_stream, _stream_handle) = OutputStream::try_default().unwrap();

        if mode == TerrasoundMode::PortInResonatorOut && outgoing_data.is_none() {
            panic!("Must provide outgoing_data when using TerrasoundMode::PortInResonatorOut");
        }

        if mode == TerrasoundMode::ResonatorInSpeakerOut && incoming_data.is_none() {
            panic!("Must provide incoming_data when using TerrasoundMode::ResonatorInSpeakerOut");
        }

        if mode == TerrasoundMode::PortInSpeakerOut && (incoming_data.is_some() && outgoing_data.is_some()) {
            panic!("incoming_data and outgoing_data should not be provided when using TerrasoundMode::PortInSpeakerOut");
        }

        if mode == TerrasoundMode::PortInSpeakerOut {
            return Terrasound {
                stream: _stream,
                stream_handle: _stream_handle.clone(),
                audio_source: Some(Arc::new(Mutex::new(AudioSource { 
                    buffers: Vec::new(),
                    current_buffer_idx: 0,
                    sink: Sink::try_new(&_stream_handle.clone()).unwrap()
                }))),
                running: Arc::new(Mutex::new(true)),
                server: Some(AudioSocketServer::new(port)),
                local_source: None,
                incoming_data: None,
                outgoing_data,
                mode: Arc::new(Mutex::new(mode))
            }
        } else if mode == TerrasoundMode::ResonatorInSpeakerOut {
            return Terrasound {
                stream: _stream,
                stream_handle: _stream_handle.clone(),
                audio_source: None,
                running: Arc::new(Mutex::new(true)),
                server: None,
                local_source: Some(Arc::new(Mutex::new(LocalAudioSource::new(incoming_data.clone().unwrap(), Sink::try_new(&_stream_handle.clone()).unwrap())))),
                incoming_data,
                outgoing_data,
                mode: Arc::new(Mutex::new(mode))
            }
        } else if mode == TerrasoundMode::PortInResonatorOut {
            return Terrasound {
                stream: _stream,
                stream_handle: _stream_handle.clone(),
                audio_source: Some(Arc::new(Mutex::new(AudioSource { 
                    buffers: Vec::new(),
                    current_buffer_idx: 0,
                    sink: Sink::try_new(&_stream_handle.clone()).unwrap()
                }))),
                running: Arc::new(Mutex::new(true)),
                server: Some(AudioSocketServer::new(port)),
                local_source: None,
                incoming_data: None,
                outgoing_data,
                mode: Arc::new(Mutex::new(mode))
            }
        } else {
            panic!("Invalid mode specified");
        }
    }

    pub fn start(&mut self) {
        let audio_source = self.audio_source.clone();
        let server = self.server.clone();

        if self.mode.lock().unwrap().clone() == TerrasoundMode::PortInSpeakerOut || self.mode.lock().unwrap().clone() == TerrasoundMode::PortInResonatorOut {
            server.unwrap().begin(self.audio_source.clone().unwrap(), self.running.clone());
        }

        let running = self.running.clone();
        let resonator_outgoing_data = self.outgoing_data.clone();
        let resonator_incoming_data = self.incoming_data.clone();
        let mode = self.mode.clone();
        let local_source = self.local_source.clone();
        thread::spawn(move || {
            let mut num_buffers: usize = 0;
            let mut playing = false;
            let mode = mode.clone();
            let local_source = local_source.clone();

            while running.lock().unwrap().clone() == true {
                if mode.lock().unwrap().clone() == TerrasoundMode::PortInSpeakerOut {
                    let temp_audio_source_arc = audio_source.clone().unwrap();
                    let mut temp_audio_source = temp_audio_source_arc.lock().unwrap();
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

                if mode.lock().unwrap().clone() == TerrasoundMode::PortInResonatorOut {
                    let temp_audio_source_arc = audio_source.clone().unwrap();
                    let mut temp_audio_source = temp_audio_source_arc.lock().unwrap();
                    let new_num_buffers = temp_audio_source.buffers.len();
                    playing = true; //No prebuffer if outputting directly to bytestream. Dont want to have an instance where the audio is buffered multiple times
                    
                    if playing { //Obv not needed but left in for clarity
                        if temp_audio_source.current_buffer_idx >= new_num_buffers {
                            continue;
                        }
                        
                        //Audio will be output to provided vec of buffers
                        let mut sink = resonator_outgoing_data.as_ref().unwrap().lock().unwrap();
                        let next_buff_result = temp_audio_source.get_next();

                        if next_buff_result.is_ok() {
                            let next_buff = next_buff_result.unwrap();
                            sink.push(next_buff.to_resonator());
                        } else {
                            println!("Error occured while processing audio for resonator-rs");
                        }
                    }
                    num_buffers = new_num_buffers;
                }

                if mode.lock().unwrap().clone() == TerrasoundMode::ResonatorInSpeakerOut {
                    let temp_local_source_arc = local_source.clone().unwrap();
                    let mut temp_local_source = temp_local_source_arc.lock().unwrap();
                    let current_prebuffer_size = num_buffers - temp_local_source.current_buffer_idx;
                    let new_num_buffers = temp_local_source.num_buffers();
                    
                    //If the prebuffer is large enough, begin playback
                    if !playing && current_prebuffer_size >= PREBUFFER_SIZE {
                        playing = true;
                    }

                    if playing {
                        if new_num_buffers == 0 {
                            continue;
                        }
                        
                        println!("Terrasound: Playing next buffer");
                        //Audio will be played from speakers
                        temp_local_source.play_next();
                    }
                    num_buffers = new_num_buffers;
                }
            }
        });
    }
}

impl Drop for Terrasound {
    fn drop(&mut self) {
        *self.running.lock().unwrap() = false;
    }
}