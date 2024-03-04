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
    filename: &str,
    id: i32,
    recv_id: i32,
    terrasound_port: u16,
    resonator_send_port: u16,
    resonator_recv_port: u16,
    stop_signal: Arc<AtomicBool>,
) {
    let mut audio_buf = AudioBuffer::buffers_from_file(filename);
    let buffers: Vec<_> = audio_buf.drain(..).map(|b| b.to_resonator()).collect();

    let juce_audio: Arc<Mutex<Vec<ResonatorAudioBuffer>>> = Arc::new(Mutex::new(buffers));
    let resonator_audio: Arc<Mutex<Vec<ResonatorAudioBuffer>>> = Arc::new(Mutex::new(Vec::new()));

    let sender: ResonatorClient = ResonatorClient::new(
        resonator_send_port,
        resonator_recv_port,
        id,
        recv_id,
        Some(juce_audio.clone()),
        Some(resonator_audio.clone()),
    );
    sender.begin();

    thread::spawn(move || {
        let mut terrasound_piro =
            Terrasound::new_port_in_resonator_out(terrasound_port, juce_audio.clone());
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
    const P3_FILENAME: &str = "sound/92002__jcveliz__violin-origional.wav";
    const P4_FILENAME: &str = "sound/557469__oleviolin__d_scale_violin.wav";

    let send_port = 8080;
    let recv_port = 8081;

    // Initialize Resonator server
    let server = ResonatorServer::new(send_port, recv_port);
    server.begin();


    let client1_id = 1;
    let client2_id = 2;
    let client3_id = 3;
    let client4_id = 4;

    // Should have different ports for binding to and listening from
    let peer1_port = 6967;
    let peer2_port = 6968;
    let peer3_port = 6969;
    let peer4_port = 6970;

    let stop_signal = Arc::new(AtomicBool::new(false));
    
    // Create the bidirectional stream on peer #1 and peer #2
    create_peer(
        P1_FILENAME,
        client1_id,
        client2_id,
        peer1_port,
        send_port,
        recv_port,
        stop_signal.clone(),
    );
    create_peer(
        P2_FILENAME,
        client2_id,
        client1_id,
        peer2_port,
        send_port,
        recv_port,
        stop_signal.clone(),
    );

    // Create the bidirectional stream on peer #3 and peer #4
    create_peer(
        P3_FILENAME,
        client3_id,
        client4_id,
        peer3_port,
        send_port,
        recv_port,
        stop_signal.clone(),
    );
    create_peer(
        P4_FILENAME,
        client4_id,
        client3_id,
        peer4_port,
        send_port,
        recv_port,
        stop_signal.clone(),
    );

    // Wait for the audio to finish playing
    thread::sleep(Duration::from_secs(11));
    stop_signal.store(true, Ordering::SeqCst);

    println!("Terrasound terminating\n\n");
    thread::sleep(Duration::from_secs(1));
}
