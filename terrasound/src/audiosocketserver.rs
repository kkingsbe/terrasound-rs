use std::io::Read;
use std::sync::{Arc, Mutex};
use std::net::{TcpListener, TcpStream};
use std::thread;

use crate::audiobuffer::AudioBuffer;
use crate::audiosource::AudioSource;
use crate::config::{BUFFER_SIZE, METADATA_SIZE};
use crate::metadata::Metadata;

pub struct AudioSocketServer {
    pub port: u16,
}

impl AudioSocketServer {
    pub fn new(port: u16) -> AudioSocketServer {
        AudioSocketServer {
            port
        }
    }

    pub fn begin(&self, client_connected: Arc<Mutex<bool>>, audio_source: Arc<Mutex<AudioSource>>) {
        println!("Beginning socket server on port {}", self.port);
        let listener: TcpListener = TcpListener::bind(format!("127.0.0.1:{}", self.port)).unwrap();
    
        thread::spawn(move || {
            for stream in listener.incoming() {
                match stream {
                    Ok(tcp_stream) => {
                        let audio_source = audio_source.clone();
                        let client_connected = client_connected.clone();
                        thread::spawn(move || {
                            handle_client(tcp_stream, audio_source, client_connected);
                        });
                    }
                    Err(e) => eprintln!("Error accepting connection: {}", e),
                }
            }
        });
    }
}

fn handle_client(mut tcp_stream: TcpStream, audio_source: Arc<Mutex<AudioSource>>, client_connected: Arc<Mutex<bool>>) {
    let mut buffer = [0; BUFFER_SIZE];
    *client_connected.lock().unwrap() = true;

    loop {
        match tcp_stream.read(&mut buffer) {
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
    }

    *client_connected.lock().unwrap() = false;
}