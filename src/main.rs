use std::os::unix::net::{UnixListener};
use std::io::prelude::*;
use std::fs::remove_file;


fn main() -> std::io::Result<()> {
    // Replace "/home/clay/rust-test" with the path to your Unix socket file.
    let socket_path = "/home/bip7/bip7-redis-server/socket";

    // Remove the socket file if it already exists.
    let _ = remove_file(socket_path);

    // Create a Unix socket listener.
    let listener = UnixListener::bind(socket_path)?;

    // Accept incoming connections and handle them in a loop.
    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut request = String::new();
                let mut response = String::new();
                let mut buffer = [0; 1024];

                loop {
                    match stream.read(&mut buffer) {
                        Ok(bytes_read) if bytes_read == 0 => {
                            // Connection closed by the client.
                            break;
                        }
                        Ok(bytes_read) => {
                            let received_data = &buffer[0..bytes_read];
                            request.push_str(&String::from_utf8_lossy(received_data));

                            // Check for a message boundary (e.g., newline) to determine
                            // when the message is complete.
                            if received_data.contains(&b'\n') {
                                // Process the complete message.
                                // You can add logic to handle different types of requests.
                                println!("in received with request: {}", request);
                                // Prepare a response.
                                response.push_str("Received: ");
                                response.push_str(&request);

                                // Send the response back to the client.
                                stream.write_all(response.as_bytes()).unwrap();

                                // Clear the request and response buffers for the next message.
                                request.clear();
                                response.clear();
                            }
                        }
                        Err(err) => {
                            eprintln!("Error reading from connection: {:?}", err);
                            break;
                        }
                    }
                }
            }
            Err(err) => {
                eprintln!("Error accepting connection: {:?}", err);
            }
        }
    }

    Ok(())
}
