use std::io::Read;
use std::os::unix::net::UnixListener;
use std::path::Path;

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
                    }
                    "stop" => {
                        println!("Stop command received");
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
