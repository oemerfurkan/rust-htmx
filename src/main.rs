use std::{
    fs, io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}, thread, time::{Duration, SystemTime}
};

use http_client::{ThreadPool, Request};

fn main() {
    let listener: TcpListener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    for stream in listener.incoming() {
        let stream = stream.unwrap();

        pool.execute(|| handle_connection(stream));
    }
}

fn handle_connection(mut stream: TcpStream) {
    let start = SystemTime::now();
    let buf_reader = BufReader::new(&mut stream);

    let request_line = buf_reader.lines().next().unwrap().unwrap();

    let req = Request::new(&request_line);

    let (status_line, filname) = match req.method {
        "GET" => match req.path {
            "/" => ("HTTP/1.1 200 OK", "pages/index.html"),
            _ => ("HTTP/1.1 404 NOT FOUND", "pages/404.html")
        },
        "POST" => match req.path {
            "/deneme" => ("HTTP/1.1 200 OK", "components/button.html"),
            _ => ("HTTP/1.1 404 NOT FOUND", "pages/404.html")
        },
        _ => ("HTTP/1.1 200 OK", "pages/404.html")
    };

    
    let contents = fs::read_to_string(filname).unwrap();
    let length = contents.len();
    
    let response = format!("{status_line}\r\nContent-Length: {length}\r\n\r\n{contents}");
    stream.write_all(response.as_bytes()).unwrap();

    let end = SystemTime::now();

    let diff = end.duration_since(start).unwrap();
    println!("{} {} {} - {}ms", req.method, req.path, req.http_version, diff.as_millis());
}
