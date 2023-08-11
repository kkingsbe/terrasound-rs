mod audiosource;
mod audiobuffer;
mod metadata;
mod audiosocketserver;

use std::sync::{mpsc::{channel, Sender, Receiver}, Arc, Mutex};
use audiobuffer::AudioBuffer;
use audiosocketserver::AudioSocketServer;
use audiosource::AudioSource;
use rodio::{OutputStream, Sink};

const PREBUFFER_SIZE: usize = 20;

fn main() {
    //Create MPSC chanel
    let (tx, rx): (Sender<AudioBuffer>, Receiver<AudioBuffer>) = channel();
    
    //Create the audio stream & sink
    let (_stream, stream_handle) = OutputStream::try_default().unwrap();
    let sink = Sink::try_new(&stream_handle).unwrap();

    //Create the audio source which stores the buffers
    let audio_source: Arc<Mutex<AudioSource>> = Arc::new(Mutex::new(AudioSource { 
        buffers: Vec::new(),
        current_buffer_idx: 0,
        sink,
    }));
    
    let mut server_running: Box<bool> = Box::new(false);

    //Start the TCP socket server
    let server: AudioSocketServer = AudioSocketServer::new(6968);
    server.begin(&mut *server_running, audio_source.clone());

    let mut num_buffers: usize = 0;
    let mut playing = false;

    while *server_running {
        //Recieve a new buffer if able
        //let new_buff_res = rx.recv();
        //println!("Empty: {}", audio_source.sink.empty());
        println!("Len: {}", audio_source.lock().unwrap().sink.len());
        
        let mut temp_audio_source = audio_source.lock().unwrap();
        let current_prebuffer_size = num_buffers - temp_audio_source.current_buffer_idx;
        let new_num_buffers = temp_audio_source.buffers.len();

        
        if current_prebuffer_size < PREBUFFER_SIZE {
            //println!("Prebuffering... {}/{}", current_prebuffer_size, PREBUFFER_SIZE);
        }

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
}
