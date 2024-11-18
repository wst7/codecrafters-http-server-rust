use std::collections::HashMap;
use std::io::{BufRead, BufReader, Read, Write};
use std::net::TcpListener;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                handle_connection(&mut stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}
// GET /index.html HTTP/1.1\r\nHost: localhost:4221\r\nUser-Agent: curl/7.64.1\r\nAccept: */*\r\n\r\n

// Request line
// GET                          // HTTP method
// /index.html                  // Request target
// HTTP/1.1                     // HTTP version
// \r\n                         // CRLF that marks the end of the request line

// // Headers
// Host: localhost:4221\r\n     // Header that specifies the server's host and port
// User-Agent: curl/7.64.1\r\n  // Header that describes the client's user agent
// Accept: */*\r\n              // Header that specifies which media types the client can accept
// \r\n                         // CRLF that marks the end of the headers

// // Request body (empty)

fn handle_connection(stream: &mut std::net::TcpStream) {
    let mut buffer = [0; 1024];
    stream.read(&mut buffer).unwrap();
    let http_request = HttpRequest::new(&mut buffer);
    println!("http_request: {:?}", &http_request);
    match http_request.head.path.as_str() {
        "/" => {
            let response = "HTTP/1.1 200 OK\r\n\r\nHello, world";
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
                println!("content: {}", content);
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
