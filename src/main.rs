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
    Hexists(String), 
    BFReserve(usize, f64),
    BFAdd(String),
    BFExists(String)
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
        "bfreserve" => {
            if req_vec.len() == 3{
                let items_parse = req_vec[1].parse::<usize>();
                match items_parse{
                    Ok(items) =>{
                        let fp_parse = req_vec[2].parse::<f64>();
                        match fp_parse{
                            Ok(fp) =>{
                                Some(Commands::BFReserve(items, fp))
                            }
                            Err(_) =>{
                                //println!("Something went wrong. Check parameters.");
                                return None;
                            }
                        }
                    }
                    Err(_) =>{
                        //println!("Something went wrong. Check parameters.");
                        return None;
                    }
                }
            }
            else{
                None
            }
        }
        "bfexists" => {
            if req_vec.len() >= 2{
                Some(Commands::BFExists(req_vec[1..].join(" ")))
            }
            else{
                None
            }
        }
        "bfadd" => {
            if req_vec.len() >= 2{
                Some(Commands::BFAdd(req_vec[1..].join(" ")))
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
    map:Arc<RwLock<HashMap<String, String>>>, bf_arc:Arc<RwLock<Bloom<String>>>) -> std::io::Result<()>{
    let mut request = String::new();
    let mut response = String::new();
    let mut buffer = [0; 1024];
    loop {
        match stream.read(&mut buffer) {
            Ok(bytes_read) if bytes_read == 0 => {
                // Connection closed by the client.
                //println!("Connection closed by client.");
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
                    //println!("in received with request: {}", request);
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
                            let map_res = map.write();
                            match map_res{
                                Ok(mut map) =>{
                                    match map.insert(key, value){
                                        None => response.push_str("1\n"),
                                        Some(a) => {
                                            let res: String = "Overwrote value: ".to_owned() + &a + "\n";
                                            response.push_str(res.as_str())
                                        }
                                    }
                                }
                                Err(_) => {
                                    response.push_str("0")
                                }
                            }
                            
                        }
                        Some(Commands::Hget(key)) => {
                            let map_res = map.read();
                            let res; 
                            match map_res{
                                Ok(map) => {
                                    match map.get(&key){
                                        Some(val)=> {
                                            res = val.to_string() + "\n";
                                        }
                                        None => {res = "Value not found. Please try another \n".to_string();}
                                    }
                                }
                                Err(_) => {
                                    res = "Something went wrong. \n".to_string();
                                }
                            }
                            response.push_str(&res)
                        }
                        Some(Commands::Hdel(key)) => {
                            let map_res = map.write();
                            match map_res{
                                Ok(mut map) =>{
                                    match map.remove(&key){
                                        Some(_) => {response.push_str("1\n")}
                                        None => {response.push_str("0\n")}
                                    }
                                }
                                Err(_) => {response.push_str("Something went wrong. \n")}
                            }
                            
                        }
                        Some(Commands::Hexists(key)) => {
                            let map_res = map.read();
                            let ret: String;
                            match map_res{
                                Ok(map) => {ret = map.contains_key(&key).to_string() + "\n";}
                                Err(_) =>{ret = "Something went wrong. \n".to_string()}
                            }
                            response.push_str(&ret)
                        }
                        Some(Commands::BFReserve(items, fp)) => {
                            let bf_res = bf_arc.write();
                            match bf_res{
                                Ok(mut bf)=>{
                                    *bf = bloomfilter::Bloom::new_for_fp_rate(items, fp);
                                    //println!("{:?}\n", *bf);
                                    response.push_str("1\n")
                                }
                                Err(_)=>{response.push_str("Something went wrong\n")}
                            }
                            
                        }
                        Some(Commands::BFAdd(item)) => {
                            let bf_res = bf_arc.write();
                            match bf_res{
                                Ok(mut bf) =>{
                                    bf.set(&item);
                                    response.push_str("1\n")
                                }
                                Err(_) =>{response.push_str("Something went wrong\n")}
                            }
                            
                        }
                        Some(Commands::BFExists(item)) => {
                            let bf_res = bf_arc.read();
                            let ret: String; 
                            match bf_res{
                                Ok(bf)=>{ret = bf.check(&item).to_string() + "\n";}
                                Err(_)=>{ret = "Something went wrong\n".to_string()}
                            }                           
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
                println!("Error reading from connection: {:?}", err);
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
    
    let shared_bf : Arc<RwLock<bloomfilter::Bloom<String>>> = 
        Arc::new(RwLock::new(bloomfilter::Bloom::new_for_fp_rate(1000, 0.01)));

    // Remove the socket file if it already exists.
    let _ = remove_file(socket_path);

    // Create a Unix socket listener.
    let listener = UnixListener::bind(socket_path)?;

    // Accept incoming connections and handle them in a loop.
    for stream in listener.incoming() {
        let map_clone: Arc<RwLock<HashMap<String, String>>> = Arc::clone(&map);
        let bf_clone: Arc<RwLock<Bloom<String>>> = Arc::clone(&shared_bf);

        thread::spawn(move || 
            match stream {
                Ok(stream) => {
                    let res = handle_client(stream, map_clone, bf_clone);
                    let thread_id_str = format!("{:?}", thread::current().id());
                    match res {
                        Ok(_) => println!("{} ok", thread_id_str),
                        Err(err) => println!("err: {:?}", err),
                    }
                }
                Err(err) => {
                    println!("Error accepting connection: {:?}", err);
                }
            }
        );
    }
        
    Ok(())
}
