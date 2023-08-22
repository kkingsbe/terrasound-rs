use std::sync::{Mutex, Arc};
use resonator::{common::audiobuffer::AudioBuffer, client::resonatorclient::ResonatorClient};
use terrasound::Terrasound;

fn main() {
    let juce_audio: Arc<Mutex<Vec<AudioBuffer>>> = Arc::new(Mutex::new(Vec::new()));
    let mut terrasound_piro = Terrasound::new_port_in_resonator_out(6967, juce_audio.clone());
    terrasound_piro.start();

    let resonator_audio: Arc<Mutex<Vec<AudioBuffer>>> = Arc::new(Mutex::new(Vec::new()));
    let mut terrasound_riso = Terrasound::new_resonator_in_speaker_out(resonator_audio.clone());
    terrasound_riso.start();

    let resonator_client: ResonatorClient = ResonatorClient::new(8080, 8081, 1, 1, Some(juce_audio.clone()), Some(resonator_audio.clone()));
    resonator_client.begin();

    loop {
        std::thread::sleep(std::time::Duration::from_millis(1000));
        println!("Received audio length: {}", resonator_audio.lock().unwrap().len());
    }
}