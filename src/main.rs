// Uncomment this block to pass the first stage
use std::{io::{Read, Write}, net::{TcpListener, TcpStream}, path::Path, str::from_utf8};


fn main() {
    // You can use print statements as follows for debugging, they'll be visible when running tests.
    println!("Logs from your program will appear here!");

    // Uncomment this block to pass the first stage
    //
    let listener = TcpListener::bind("127.0.0.1:4221").unwrap();
    
    for stream in listener.incoming() {
        match stream {
            Ok(_stream) => {
                std::thread::spawn(move || {
                    handle_client(_stream)
                });
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
        "/" => HttpResponse {
            status_code: HttpResponseCode::Ok,
            ..HttpResponse::default()
        },
        "/user-agent" => {
            let user_agent = lines[2].split(" ").collect::<Vec<&str>>()[1];
            HttpResponse {
                status_code: HttpResponseCode::Ok,
                content_type: String::from("text/plain"),
                content_length: user_agent.len() as i32,
                content: user_agent.to_string(),
            }
        },
        _ => if path.starts_with("/echo/") {
            let (_,echo) = path.split_once("/echo/").unwrap();
            HttpResponse {
                status_code: HttpResponseCode::Ok,
                content_type: String::from("text/plain"),
                content_length: echo.len() as i32,
                content: echo.to_string(),
            }
        }else if path.starts_with("/files/"){
            let args: Vec<String> = std::env::args().collect();
                let dir = args[2].to_string();
                let filename = lines[2].split(" ").collect::<Vec<&str>>()[1];
                let path = format!("{}/{}", dir, filename);
                if Path::new(&path).exists() {
                    HttpResponse {
                        status_code: HttpResponseCode::Ok,
                        content_type: String::from("application/octet-stream"),
                        content: std::fs::read_to_string(path).unwrap(),
                        ..HttpResponse::default()
                    }
                } else {
                    HttpResponse {
                        status_code: HttpResponseCode::NotFound,
                        ..HttpResponse::default()
                    }
                }
        } 
        else {
            HttpResponse{
                status_code: HttpResponseCode::NotFound,
                ..HttpResponse::default()
            }
        },
    };
    stream.write_all(resp.to_string().as_bytes()).unwrap()

}

enum HttpResponseCode {
    Ok,
    NotFound
}

impl HttpResponseCode {
    fn code(&self) -> i32 {
        match self {
            HttpResponseCode::Ok => 200,
            HttpResponseCode::NotFound => 404,
        }
    }

    fn text(&self) -> &str {
        match self {
            HttpResponseCode::Ok => "",
            HttpResponseCode::NotFound => "",
        }
    }
}

struct HttpResponse {
    status_code: HttpResponseCode,
    content_type: String,
    content_length: i32,
    content: String
}

impl Default for HttpResponse {
    fn default() -> Self {
        Self {
            status_code: HttpResponseCode::Ok,
            content_type: String::new(),
            content_length: 0,
            content: String::new()
        }
    }
}

impl HttpResponse {
    fn to_string(&self) -> String {
        let mut resp: String = String::from("HTTP/1.1 ");
        resp.push_str(self.status_code.code().to_string().as_str());
        resp.push_str(" ");
        resp.push_str(self.status_code.text());
        resp.push_str("\r\n");
        if !self.content.is_empty() {
            resp.push_str("Content-Type: ");
            resp.push_str(self.content_type.as_str());
            resp.push_str("\r\n");
            resp.push_str("Content-Length: ");
            resp.push_str(self.content_length.to_string().as_str());
            resp.push_str("\r\n\r\n");
            resp.push_str(self.content.as_str());
            resp.push_str("\r\n");
        }
        resp.push_str("\r\n");
        resp
    }
}