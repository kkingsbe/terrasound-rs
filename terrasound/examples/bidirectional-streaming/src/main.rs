use resonator::{
    client::resonatorclient::ResonatorClient,
    common::audiobuffer::AudioBuffer as ResonatorAudioBuffer, server::resonator::ResonatorServer,
};
use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc, Mutex,
    },
    thread,
    time::Duration,
};
use terrasound::{audiobuffer::AudioBuffer, Terrasound};

fn create_peer(
    port: u16,
    juce_audio: Arc<Mutex<Vec<ResonatorAudioBuffer>>>,
    resonator_audio: Arc<Mutex<Vec<ResonatorAudioBuffer>>>,
    stop_signal: Arc<AtomicBool>,
) {
    thread::spawn(move || {
        let mut terrasound_piro = Terrasound::new_port_in_resonator_out(port, juce_audio.clone());
        terrasound_piro.start();

        let mut terrasound_riso = Terrasound::new_resonator_in_speaker_out(resonator_audio.clone());
        terrasound_riso.start();

        // Wait for the network
        thread::sleep(Duration::from_secs(1));

        while !stop_signal.load(Ordering::SeqCst) {}
    });
}

fn main() {
    const P1_FILENAME: &str = "sound/256081__elettroedo__bass_phrase1.wav";
    const P2_FILENAME: &str = "sound/256999__paul-evans__short-bach-melody.wav";

    let mut d1 = AudioBuffer::buffers_from_file(P1_FILENAME);
    let mut d2 = AudioBuffer::buffers_from_file(P2_FILENAME);

    let buf1: Vec<_> = d1.drain(..).map(|b| b.to_resonator()).collect();
    let buf2: Vec<_> = d2.drain(..).map(|b| b.to_resonator()).collect();

    // Should have different ports for binding to and listening from
    let peer1_port = 6967;
    let peer2_port = 6968;

    let stop_signal = Arc::new(AtomicBool::new(false));

    // Inputs
    let juce_audio1: Arc<Mutex<Vec<ResonatorAudioBuffer>>> = Arc::new(Mutex::new(buf1));
    let juce_audio2: Arc<Mutex<Vec<ResonatorAudioBuffer>>> = Arc::new(Mutex::new(buf2));

    // Outputs
    let resonator_audio1: Arc<Mutex<Vec<ResonatorAudioBuffer>>> = Arc::new(Mutex::new(Vec::new()));
    let resonator_audio2: Arc<Mutex<Vec<ResonatorAudioBuffer>>> = Arc::new(Mutex::new(Vec::new()));

    // Create the Terrasound peers
    create_peer(
        peer1_port,
        juce_audio1.clone(),
        resonator_audio1.clone(),
        stop_signal.clone(),
    );
    create_peer(
        peer2_port,
        juce_audio2.clone(),
        resonator_audio2.clone(),
        stop_signal.clone(),
    );

    // Initialize Resonator server

    let send_port = 8080;
    let recv_port = 8081;

    let server = ResonatorServer::new(send_port, recv_port);
    server.begin();

    // Then initialize the clients

    let client1_id = 1;
    let client2_id = 2;

    let client1: ResonatorClient = ResonatorClient::new(
        send_port,
        recv_port,
        client1_id,
        client1_id,
        Some(juce_audio1.clone()),
        Some(resonator_audio2.clone()),
    );
    client1.begin();

    let client2: ResonatorClient = ResonatorClient::new(
        send_port,
        recv_port,
        client2_id,
        client2_id,
        Some(juce_audio2.clone()),
        Some(resonator_audio1.clone()),
    );
    client2.begin();

    // Wait for the audio to finish playing    
    thread::sleep(Duration::from_secs(6));
    stop_signal.store(true, Ordering::SeqCst);

    println!("Terrasound terminating\n\n");
    thread::sleep(Duration::from_secs(1));
}
