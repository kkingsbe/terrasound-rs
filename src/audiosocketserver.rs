use std::io::Read;
use std::sync::{Arc, Mutex};
use std::sync::mpsc::Sender;
use std::net::{TcpListener, TcpStream};
use std::thread;

use crate::audiobuffer::AudioBuffer;
use crate::audiosource::AudioSource;
use crate::metadata::Metadata;

const AUDIO_BUFFER_SIZE: usize = 1024;
const METADATA_SIZE: usize = 2 * std::mem::size_of::<i32>();
const BUFFER_SIZE: usize = (AUDIO_BUFFER_SIZE * std::mem::size_of::<f32>()) + METADATA_SIZE;

pub struct AudioSocketServer {
    pub port: u16,
}

impl AudioSocketServer {
    pub fn new(port: u16) -> AudioSocketServer {
        AudioSocketServer {
            port
        }
    }

    pub fn begin(&self, server_running: &mut bool, audio_source: Arc<Mutex<AudioSource>>) {
        *server_running = true;
        let listener: TcpListener = TcpListener::bind(format!("127.0.0.1:{}", self.port)).unwrap();
        let server_running_clone = server_running.clone();
    
        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(tcp_stream) => {
                        let server_running_clone = server_running_clone.clone(); // Clone for the closure
                        let audio_source = audio_source.clone();
                        thread::spawn(move || {
                            handle_client(tcp_stream, audio_source);
                        });
                    }
                    Err(e) => eprintln!("Error accepting connection: {}", e),
                }
            }
        });
    }
}

fn handle_client(mut tcp_stream: TcpStream, audio_source: Arc<Mutex<AudioSource>>) {
    let mut buffer = [0; BUFFER_SIZE];

    loop {
        let size = match tcp_stream.read(&mut buffer) {
            Ok(size) if size == 0 => {
                println!("Client closed connection");
                break;
            }
            Ok(size) => size,
            Err(_) => {
                println!("Client error");
                break;
            }
        };

        let bytes = &buffer[..];
        let metadata_bytes = &bytes[..METADATA_SIZE];
        let audio_bytes = &bytes[METADATA_SIZE..];

        let m_buff_size = i32::from_le_bytes(metadata_bytes[0..4].try_into().unwrap());
        let m_sample_rate = i32::from_le_bytes(metadata_bytes[4..8].try_into().unwrap());

        let samples: Vec<f32> = audio_bytes.chunks_exact(4).map(|chunk| {
            f32::from_le_bytes([chunk[0], chunk[1], chunk[2], chunk[3]])
        })
        .collect();

        let new_buff = AudioBuffer {
            samples,
            metadata: Metadata {
                sample_rate: m_sample_rate,
                buffer_size: m_buff_size,
            }
        };

        audio_source.lock().unwrap().add_buffer(new_buff);
        //audio_source.lock().unwrap().play_next();
    }
}