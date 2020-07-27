use std::net::{TcpListener, TcpStream};
use std::io::prelude::*;
use std::fs;
use std::thread;
use std::time::Duration;
use multithread_server::ThreadPool;

fn main() {
    let tcp_listener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let thread_pool = ThreadPool::new(16);

    println!("Thead pool initialized.");

    for tcp_stream in tcp_listener.incoming() {
        let tcp_stream = tcp_stream.unwrap();

        println!("Connection established!");
        
        thread_pool.execute(|| {
             handle_connection(tcp_stream); 
        });
    }
    println!("Shutting down server.")
}

fn handle_connection(mut stream: TcpStream) {
    let mut buffer = [0; 1024];

    stream.read(&mut buffer).unwrap();

    let buffer_stringified = String::from_utf8_lossy(&buffer);

    let (status_line, filename) = if buffer_stringified.contains("GET") {
        if buffer_stringified.contains("/sleep") {
            println!("Waiting 5 seconds.");
            thread::sleep(Duration::from_secs(5));
            println!("Request response served.");
            ("HTTP/1.1 200 OK", "hello.html")
        } else {
            ("HTTP/1.1 200 OK", "hello.html")
        }
    } else {
        ("HTTP/1.1 404 NOT FOUND", "404.html")
    };

    let clrf = "\r\n";
    let html_content = fs::read_to_string(filename).unwrap();
    let parsed_content = format!(
        "{}{}Content-Length: {}{}{}{}", status_line, clrf, html_content.len(), clrf, clrf, html_content
    );

    stream.write(parsed_content.as_bytes()).unwrap();
    stream.flush().unwrap();
}
