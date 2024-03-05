// Uncomment this block to pass the first stage
use std::{collections::HashMap, io::{Read, Write}, net::{TcpListener, TcpStream}, path::Path,};


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
    
    let req = match parse_request(&mut stream) {
        Ok(req) => req,
        Err(e) => {
            println!("{:?}",e);
            return;
        } 
    };

    let resp = match req.route() {
        ("GET","/") => HttpResponse {
            status_code: HttpResponseCode::Ok,
            ..HttpResponse::default()
        },
       ("GET", "/user-agent") => {
        if req.headers.contains_key("User-Agent") {
            let user_agent = req.headers.get("User-Agent").unwrap();
            HttpResponse {
                status_code: HttpResponseCode::Ok,
                content_type: String::from("text/plain"),
                content_length: user_agent.len() as i32,
                content: user_agent.to_string(),
            }
        }else{
            HttpResponse {
                status_code: HttpResponseCode::NotFound,
                ..HttpResponse::default()
            }
        }
        },
        ("GET",p) => {
            if p.starts_with("/echo/") {
                let echo = &p[6..];

                HttpResponse {
                    status_code: HttpResponseCode::Ok,
                    content: echo.to_string(),
                    content_type: String::from("text/plain"),
                    content_length: echo.len() as i32, 
                }
            }else if p.starts_with("/files/"){
                let args: Vec<String> = std::env::args().collect();
                    let dir = args[2].to_string();
                    let (_,filename) = p.split_once("/files/").unwrap();
                    let path = format!("{}/{}", dir, filename);
                    if Path::new(&path).exists() {
                        let mut file = std::fs::read_to_string(path).unwrap();
                        let content_length = file.len() as i32;
                        let content_type = String::from("application/octet-stream");
                        let content = std::mem::take(&mut file);
                        HttpResponse {
                            status_code: HttpResponseCode::Ok,
                            content_type,
                            content,
                            content_length,
                        }
                    }else {
                        HttpResponse {
                            status_code: HttpResponseCode::NotFound,
                            ..HttpResponse::default()
                        }
                    }
                }
            else{
                HttpResponse {
                    status_code: HttpResponseCode::NotFound,
                    ..HttpResponse::default()
                }
            }
        },
        ("POST",p) => if p.starts_with("/files/") {

            // getting path and filename
            let args: Vec<String> = std::env::args().collect();
            let dir = args[2].to_string();
            let (_,filename) = p.split_once("/files/").unwrap();
            let path = format!("{}/{}", dir, filename);

            // create file
            let mut file = std::fs::OpenOptions::new()
                        .write(true)
                        .truncate(true)
                        .create(true)
                        .open(path)
                        .unwrap();

            // write to file
            match req.body {
                Some(body) => {
                    file.write_all(body.as_slice()).unwrap();
                    HttpResponse {
                        status_code: HttpResponseCode::Created,
                        ..HttpResponse::default()
                    }
                }
                None => {
                    HttpResponse {
                        status_code: HttpResponseCode::BadRequest,
                        ..HttpResponse::default()
                    }
                }
            }
        }else{
            HttpResponse {
                status_code:HttpResponseCode::BadRequest,
                ..HttpResponse::default()
            }
        }
        _ => {
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
    NotFound,
    Created,
    BadRequest
}

impl HttpResponseCode {
    fn code(&self) -> i32 {
        match self {
            HttpResponseCode::Ok => 200,
            HttpResponseCode::NotFound => 404,
            HttpResponseCode::Created => 201,
            HttpResponseCode::BadRequest => 400
        }
    }

    fn text(&self) -> &str {
        match self {
            HttpResponseCode::Ok => "Ok",
            HttpResponseCode::NotFound => "Not Found",
            HttpResponseCode::Created => "Created",
            HttpResponseCode::BadRequest => "Bad Request"
        }
    }
}

#[derive(Debug)]
struct Request {
    method: String,
    path: String,
    headers: HashMap<String, String>,
    body: Option<Vec<u8>>
}

impl Request {
    fn route(&self) -> (&str, &str) {
        (&self.method, &self.path)
    }
}

#[derive(Debug)]
enum HttpParseError {
    HeadersTooBig,
    IoError(std::io::Error)
}

fn parse_request(stream: &mut TcpStream) -> Result<Request, HttpParseError> {
    let mut data = Vec::new();

    let mut buffer = [0; 2048];

    let (n, boundary) = 'outter: loop {
        match stream.read(&mut buffer) {
            Ok(n) => {
                let boundary = b"\r\n\r\n";
                let boundary_len = 4;
                for i in 0..=(n - boundary_len) {
                    if &buffer[i..i + boundary_len] == boundary {
                        break 'outter (n, i + boundary_len);
                    }
                }
                return Err(HttpParseError::HeadersTooBig);
            }
            Err(e) => {
                println!("{}", e);
                return Err(HttpParseError::IoError(e));
            }
        }
    };

    data.extend_from_slice(&buffer[0..boundary]);

    let req_data = &data;
    let stuff = std::str::from_utf8(&req_data).unwrap();
    let lines: Vec<&str> = stuff.split("\r\n").collect();
    let mut start_line = lines[0].split(" ");
    let method = start_line.next().unwrap().to_owned();
    let path = start_line.next().unwrap().to_owned();

    let mut headers = HashMap::<String, String>::new();
    for line in lines.iter().skip(1) {
        if *line == "" {
            break;
        }
        let mut split = line.split(":");
        let key = split.next().unwrap().trim();
        let value = split.next().unwrap().trim();
        headers.insert(key.to_string(), value.to_string());

    };
    let body: Option<Vec<u8>> = if headers.contains_key("Content-Length") {
        let raw = headers.get("Content-Length").unwrap();
        let mut remaining = usize::from_str_radix(raw, 10).unwrap();
        println!("Content-Length is: {remaining}");
        println!("{boundary} {n}");
        let mut body = Vec::with_capacity(remaining);
        // we've read some of the body already
        if n > boundary {
            let content = &buffer[boundary..(boundary + remaining)];
            body.extend_from_slice(content);
            remaining = match remaining.checked_sub(content.len()) {
                Some(val) => val,
                None => 0,
            };
        }
        loop {
            if remaining == 0 {
                break;
            }
            let mut buffer = [0; 2048];
            match stream.read(&mut buffer) {
                Ok(n) => {
                    body.extend_from_slice(&buffer[0..n]);
                    remaining = match remaining.checked_sub(n) {
                        Some(val) => val,
                        None => 0,
                    };
                }
                Err(e) => {
                    return Err(HttpParseError::IoError(e));
                }
            }
        }
        Some(body)
    } else {
        None
    };

    Ok(Request {
        method,
        path,
        headers,
        body,
    })
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