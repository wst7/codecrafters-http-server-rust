use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread::spawn;

fn main() {
    let args = env::args().collect::<Vec<String>>();
    let mut dir = None;
    if args.len() == 3 && args[1] == "--directory" {
        dir = Some(args[2].clone());
    }
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                let dir = dir.clone();
                spawn(|| handle_connection(stream, dir));
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_connection(mut stream: TcpStream, dir: Option<String>) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let http_request = HttpRequest::new(&mut buffer);
    match http_request.method.as_str() {
        "GET" => handle_get_request(http_request, stream, dir),
        "POST" => handle_post_request(http_request, stream, dir),
        _ => {
            let response = "HTTP/1.1 405 Method Not Allowed\r\n\r\n";
            stream.write_all(response.as_bytes()).unwrap();
        }
    }
}

fn handle_get_request(http_request: HttpRequest, mut stream: TcpStream, dir: Option<String>) {
    match http_request.path.as_str() {
        "/" => {
            let response = "HTTP/1.1 200 OK\r\n\r\n";
            stream.write_all(response.as_bytes()).unwrap();
        }
        "/user-agent" => {
            let user_agent = http_request.headers.get("User-Agent").unwrap();
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
            } else if s.starts_with("/files/") {
                let filename = s.strip_prefix("/files").unwrap();
                let filepath = match dir {
                    Some(dir) => format!("{dir}/{filename}"),
                    None => format!("/{filename}"),
                };
                let response = match read_file(filepath) {
                    Ok(content) => format!(
                        "HTTP/1.1 200 OK\r\nContent-Type: application/octet-stream\r\nContent-Length: {}\r\n\r\n{}",
                        content.len(),
                        content
                    ),
                    Err(_) => "HTTP/1.1 404 Not Found\r\n\r\n".to_string(),
                };
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
fn handle_post_request(http_request: HttpRequest, mut stream: TcpStream, dir: Option<String>) {
    match http_request.path.as_str() {
        s => {
            if s.starts_with("/files/") {
                let filename = s.strip_prefix("/files").unwrap();
                let filepath = match dir {
                    Some(dir) => format!("{dir}/{filename}"),
                    None => format!("/{filename}"),
                };
                write_file(filepath, http_request.body).unwrap();
                let response = format!("HTTP/1.1 201 Created\r\n\r\n");
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

fn read_file(filepath: String) -> std::io::Result<String> {
    let mut file = File::open(filepath)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    Ok(content)
}
fn write_file(filepath: String, content: Vec<u8>) -> std::io::Result<()> {
    let mut file = File::create(filepath)?;
    file.write_all(&content)?;
    Ok(())
}

#[derive(Debug)]
struct HttpRequest {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: Vec<u8>,
}

impl HttpRequest {
    pub fn new(buffer: &mut [u8]) -> Self {
        let request = String::from_utf8_lossy(&buffer[..]);
        let mut method = String::new();
        let mut path = String::new();
        let mut headers = HashMap::new();
        let mut body = Vec::new();
        let mut is_header = true;
        let mut is_first_line = true;
        let lines: Vec<_> = request.split("\r\n").collect();
        lines.iter().for_each(|&line| {
            if line.is_empty() {
                is_header = false;
                return;
            }
            if is_first_line {
                let mut items = line.split(" ");
                method = items.next().unwrap().to_string();
                path = items.next().unwrap().to_string();
                is_first_line = false;
                return;
            }
            if is_header {
                let items: Vec<_> = line.split(": ").collect();
                headers.insert(items[0].to_string(), items[1].to_string());
                return;
            }
            // body
            body.extend_from_slice(line.trim_matches('\0').as_bytes());
        });
        HttpRequest {
            method,
            path,
            headers,
            body,
        }
    }
}
