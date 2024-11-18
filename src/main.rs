use std::io::{BufRead, BufReader, Write};
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
    let buf_reader = BufReader::new(&mut *stream);

    let http_request: Vec<_> = buf_reader
    .lines()
    .map(|line| line.unwrap())
    .take_while(|x| !x.is_empty())
    .collect();

    let request_line = http_request[0].split(" ").collect::<Vec<&str>>();

    match request_line[1] {
        "/" => {
            let response = "HTTP/1.1 200 OK\r\n\r\nHello, world";
            stream.write_all(response.as_bytes()).unwrap();
        }
        _ => {
            let response = "HTTP/1.1 404 Not Found\r\n\r\n";
            stream.write_all(response.as_bytes()).unwrap();
        }
    }

}