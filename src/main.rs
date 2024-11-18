use std::collections::HashMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::spawn;

fn main() {
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                spawn(|| handle_connection(stream));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let http_request = HttpRequest::new(&mut buffer);
    match http_request.head.path.as_str() {
        "/" => {
            let response = "HTTP/1.1 200 OK\r\n\r\n";
            stream.write_all(response.as_bytes()).unwrap();
        }
        "/user-agent" => {
            let user_agent = http_request.head.headers.get("User-Agent").unwrap();
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                user_agent.len(),
                user_agent
            );
            stream.write_all(response.as_bytes()).unwrap();
        }
        s => {
            if s.starts_with("/echo/") {
                let content = s.strip_prefix("/echo/").unwrap();
                let response = format!(
                    "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\n\r\n{}",
                    content.len(),
                    content
                );
                stream.write_all(response.as_bytes()).unwrap();
            } else {
                let response = "HTTP/1.1 404 Not Found\r\n\r\n";
                stream.write_all(response.as_bytes()).unwrap();
            }
        }
        _ => {
            let response = "HTTP/1.1 404 Not Found\r\n\r\n";
            stream.write_all(response.as_bytes()).unwrap();
        }
    }
}

#[derive(Debug)]
struct HttpRequest {
    head: Head,
}
#[derive(Debug)]
struct Head {
    method: String,
    path: String,
    headers: HashMap<String, String>,
}
impl HttpRequest {
    pub fn new(buffer: &mut [u8]) -> Self {
        HttpRequest {
            head: Head::new(buffer),
        }
    }
}
impl Head {
    pub fn new(buffer: &mut [u8]) -> Self {
        let request = String::from_utf8_lossy(&buffer[..]);
        let mut method = String::new();
        let mut path = String::new();
        let mut headers = HashMap::new();

        let mut is_header = true;
        let lines: Vec<_> = request.split("\r\n").collect();
        lines.iter().for_each(|&line| {
            if line.is_empty() {
                is_header = false;
                return;
            }
            if line.starts_with("GET") {
                let mut items = line.split(" ");
                method = items.next().unwrap().to_string();
                path = items.next().unwrap().to_string();
                return;
            }
            if is_header {
                let items: Vec<_> = line.split(": ").collect();
                headers.insert(items[0].to_string(), items[1].to_string());
            }
            //
        });

        Head {
            method,
            path,
            headers,
        }
    }
}
