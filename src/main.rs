use std::fs::{create_dir_all, File};
use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;

use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use dirs::home_dir;
use hound::{WavSpec, WavWriter};
use tokio::runtime::Runtime;

fn main() {
    let rt = Runtime::new().unwrap();
    rt.block_on(async {
        start_service().await;
    });
}

async fn start_service() {
    let socket_path = "/tmp/hanasu_socket";
    if Path::new(socket_path).exists() {
        std::fs::remove_file(socket_path).unwrap();
    }

    let listener = UnixListener::bind(socket_path).expect("Failed to bind Unix socket");
    println!("Listening on {}", socket_path);

    let recording = Arc::new(Mutex::new(false));
    let audio_data = Arc::new(Mutex::new(Vec::new()));
    let sample_rate = Arc::new(Mutex::new(0u32));

    let rec_state = Arc::clone(&recording);
    let audio_buffer = Arc::clone(&audio_data);
    let sample_rate_clone = Arc::clone(&sample_rate);

    let _audio_thread = thread::spawn(move || {
        let host = cpal::default_host();
        let device = host
            .default_input_device()
            .expect("Failed to get default input device");
        let config = device
            .default_input_config()
            .expect("Failed to get default input config");

        let err_fn = |err| eprintln!("An error occurred on the input stream: {}", err);

        *sample_rate_clone.lock().unwrap() = config.sample_rate().0;

        let stream = device
            .build_input_stream(
                &config.into(),
                move |data: &[i16], _| {
                    let mut recording = rec_state.lock().unwrap();
                    if *recording {
                        let mut buffer = audio_buffer.lock().unwrap();
                        buffer.extend_from_slice(data);
                    }
                },
                err_fn,
                None,
            )
            .expect("Failed to build input stream");

        stream.play().expect("Failed to start audio stream");

        loop {
            thread::sleep(Duration::from_millis(100));
        }
    });

    for stream in listener.incoming() {
        match stream {
            Ok(mut socket) => {
                let mut buffer = [0; 128];
                let bytes_read = socket
                    .read(&mut buffer)
                    .expect("Failed to read from socket");
                let msg = String::from_utf8_lossy(&buffer[..bytes_read])
                    .trim()
                    .to_string();

                match msg.as_str() {
                    "wake" => {
                        println!("Wake command received");
                        let mut rec_state = recording.lock().unwrap();
                        *rec_state = true;
                    }
                    "stop" => {
                        println!("Stop command received");
                        let mut rec_state = recording.lock().unwrap();
                        *rec_state = false;

                        save_audio(&audio_data.lock().unwrap(), &sample_rate);
                        audio_data.lock().unwrap().clear();
                    }
                    _ => {
                        println!("Unknown command. Possible commands are: wake|stop");
                    }
                }
            }
            Err(err) => {
                eprintln!("Failed to accept socket connection: {}", err);
            }
        }
    }
}

fn save_audio(data: &[i16], sample_rate: &Arc<Mutex<u32>>) {
    let home = home_dir().expect("Could not determine home directory");
    let logs_dir = home.join("hanasu-logs");
    create_dir_all(&logs_dir).expect("Failed to create hanasu-logs directory");

    let timestamp = chrono::Local::now().format("%Y-%m-%d_%H-%M-%S").to_string();
    let file_path = logs_dir.join(format!("recording_{}.wav", timestamp));

    let sample_rate_value = *sample_rate.lock().expect("Failed to lock sample rate");

    let spec = WavSpec {
        channels: 1,
        sample_rate: sample_rate_value,
        bits_per_sample: 16,
        sample_format: hound::SampleFormat::Int,
    };

    let mut writer = WavWriter::create(&file_path, spec).expect("Failed to create WAV file");

    for &sample in data {
        writer.write_sample(sample).expect("Failed to write sample");
    }

    writer.finalize().expect("Failed to finalize WAV file");
    println!("Audio saved to {:?}", file_path);
}
