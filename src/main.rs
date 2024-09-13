use std::{
    fs, io::{prelude::*, BufReader}, net::{TcpListener, TcpStream}, time::SystemTime
};

use http_client::{ThreadPool, Request, App};

fn main() {
    let listener: TcpListener = TcpListener::bind("127.0.0.1:7878").unwrap();
    let pool = ThreadPool::new(4);

    let app = App::new();
    
    println!("{:?}", app.paths);
    
    for stream in listener.incoming() {
        let stream = stream.unwrap();
        let value = app.handle_request(stream);

        pool.execute(move || value);
    }
}