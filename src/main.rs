// Uncomment this block to pass the first stage
use std::{io::{Read, Write}, net::{TcpListener, TcpStream}, str::from_utf8};

use nom::AsBytes;

fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                handle_client(_stream);
            }
            Err(e) => {
                println!("error: {}", e);
            }
        }
    }
}

fn handle_client(mut stream: TcpStream){
    let mut buff: [u8; 512] = [0; 512]; 

    stream.read(&mut buff).unwrap();

    let headers = from_utf8(&buff).unwrap();
    let lines: Vec<&str> = headers.split("\r\n").collect();

    let path = lines[0].split(" ").collect::<Vec<&str>>()[1];

    let resp = match path {
        "/" => "HTTP/1.1 200 OK\r\n\r\n",
        _ =>"HTTP/1.1 400 Not Found\r\n\r\n",
    };
    stream.write_all(resp.as_bytes()).unwrap()

}