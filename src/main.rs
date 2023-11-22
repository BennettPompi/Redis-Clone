//use std::collections::btree_map::Values;
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::io::prelude::*;
use std::fs::remove_file;
use std::thread;
use std::env;
use std::time;
enum Commands{
    Ping,
    Time
}
fn parse_command(request: &String)->Option<Commands>
{
    if request.split_whitespace().next()?.to_lowercase().eq("ping")
    {
        return Some(Commands::Ping)
    }
    else if request.split_whitespace().next()?.to_lowercase().eq("time")
    {
        return Some(Commands::Time)
    }
    else {
        None
    }
}
fn ping()-> String
{
    return "Pong bip7\n".to_string()
}
fn time()-> String
{
    let time = time::SystemTime::now().duration_since(time::UNIX_EPOCH).
    expect("Result Invalid").as_nanos().to_string() + "\n";
    return time;
}
fn handle_client(mut stream: UnixStream) -> std::io::Result<()>{
    let mut request = String::new();
    let mut response = String::new();
    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(bytes_read) if bytes_read == 0 => {
                // Connection closed by the client.
                println!("Connection closed by client.");
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
                    let cmd: Option<Commands> = parse_command(&request);
                    match cmd
                    {
                        Some(Commands::Ping) => {
                            response.push_str(&ping());
                        }
                        Some(Commands::Time) => {
                            response.push_str(&time());
                        }
                        None => {
                            response.push_str("Unknown Command\n")
                        }
                    }

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
    Ok(())
}
fn main() -> std::io::Result<()> {
    // Replace "/home/clay/rust-test" with the path to your Unix socket file.
    let args: Vec<String> = env::args().collect();
    let socket_path = &args[1];

    // Remove the socket file if it already exists.
    let _ = remove_file(socket_path);

    // Create a Unix socket listener.
    let listener = UnixListener::bind(socket_path)?;

    // Accept incoming connections and handle them in a loop.
    for stream in listener.incoming() {
        thread::spawn(|| 
            match stream {
                Ok(stream) => {
                    let res = handle_client(stream);
                    let thread_id = thread::current().id();
                    let thread_id_str = format!("{:?}", thread_id);
                    match res {
                        Ok(_) => println!("{} ok", thread_id_str),
                        Err(err) => println!("err: {:?}", err),
                    }
                }
                Err(err) => {
                    eprintln!("Error accepting connection: {:?}", err);
                }
            }
        );
    }
        
    Ok(())
}
