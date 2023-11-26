use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::os::unix::net::UnixListener;
use std::os::unix::net::UnixStream;
use std::io::prelude::*;
use std::fs::remove_file;
use std::thread;
use std::env;
use std::time;
use bloomfilter::Bloom;
enum Commands{
    Ping,
    Time,
    Hset(String, String),
    Hget(String),
    Hdel(String),
    Hexists(String)
}

fn parse_command(request: &str)->Option<Commands>
{
    let req_vec:Vec<&str>  = request.split_whitespace().collect();
    let cmd = req_vec[0].to_lowercase();
    match cmd.as_str(){
        "ping" => {
            Some(Commands::Ping)
        }   
        "time" => {
            Some(Commands::Time)
        }
        "hget" => {
            if req_vec.len() >= 2{
                Some(Commands::Hget(req_vec[1..].join(" ")))
            }
            else{
                None
            }
        }
        "hdel" => {
            if req_vec.len() >= 2{
                Some(Commands::Hdel(req_vec[1..].join(" ")))
            }
            else{
                None
            }
        }
        "hexists" => {
            if req_vec.len() >= 2{
                Some(Commands::Hexists(req_vec[1..].join(" ")))
            }
            else{
                None
            }
        }
        "hset" => {
            if req_vec.len() >= 3{
                Some(Commands::Hset(req_vec[1].to_string(), req_vec[2..].join(" ")))
            }
            else{
                None
            }
        }
        _ => {
            None
        }
    }
}
fn ping()-> String
{
    "Pong bip7\n".to_string()
}
fn time()-> String
{
    let time = time::SystemTime::now().duration_since(time::UNIX_EPOCH).
    expect("Result Invalid").as_nanos().to_string() + "\n";
    time
}
fn handle_client(mut stream: UnixStream, 
    map:Arc<RwLock<HashMap<String, String>>>) -> std::io::Result<()>{
    
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
                    let cmd: Option<Commands> = parse_command(request.as_str());
                    match cmd
                    {
                        Some(Commands::Ping) => {
                            response.push_str(&ping())
                        }
                        Some(Commands::Time) => {
                            response.push_str(&time())
                        }
                        Some(Commands::Hset(key, value)) => {
                            let mut map = map.write().unwrap();
                            match map.insert(key, value){
                                None => response.push_str("1\n"),
                                Some(a) => {
                                    let res: String = "Overwrote value: ".to_owned() + &a + "\n";
                                    response.push_str(res.as_str())
                                }
                            }
                            
                        }
                        Some(Commands::Hget(key)) => {
                            let map = map.read().unwrap();
                            let res: String;
                            match map.get(&key){
                                Some(val)=> {
                                    res = val.to_string() + "\n";
                                }
                                None => {res = "Value not found. Please try another \n".to_string();}
                            }
                            response.push_str(&res)
                        }
                        Some(Commands::Hdel(key)) => {
                            let mut map = map.write().unwrap();
                            match map.remove(&key){
                                Some(_) => {response.push_str("1\n")}
                                None => {response.push_str("0\n")}
                            }
                            
                        }
                        Some(Commands::Hexists(key)) => {
                            let map = map.read().unwrap();
                            let ret = map.contains_key(&key).to_string() + "\n";
                            response.push_str(&ret)
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
    let map: Arc<RwLock<HashMap<String, String>>> = Arc::new(RwLock::new(HashMap::new()));
    // Remove the socket file if it already exists.
    let _ = remove_file(socket_path);

    // Create a Unix socket listener.
    let listener = UnixListener::bind(socket_path)?;

    // Accept incoming connections and handle them in a loop.
    for stream in listener.incoming() {
        let map_clone: Arc<RwLock<HashMap<String, String>>> = Arc::clone(&map);
        thread::spawn(move || 
            match stream {
                Ok(stream) => {
                    let res = handle_client(stream, map_clone);
                    let thread_id_str = format!("{:?}", thread::current().id());
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
