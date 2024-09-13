use std::{
    collections::HashMap,
    fs,
    io::{prelude::*, BufReader},
    net::TcpStream,
    sync::{mpsc, Arc, Mutex},
    thread,
    time::SystemTime,
};

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

struct Worker {
    id: usize,
    thread: thread::JoinHandle<()>,
}

impl Worker {
    fn new(id: usize, receiver: Arc<Mutex<mpsc::Receiver<Job>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let job = receiver.lock().unwrap().recv().unwrap();

            job();
        });

        Worker { id, thread }
    }
}

impl ThreadPool {
    /// Create a new ThreadPool.
    ///
    /// The size is the number of threads in the pool.
    ///
    /// # Panics
    ///
    /// The `new` function will panic if the size is zero.
    pub fn new(size: usize) -> ThreadPool {
        assert!(size > 0);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        let mut workers = Vec::with_capacity(size);

        for id in 0..size {
            workers.push(Worker::new(id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);

        self.sender.send(job).unwrap();
    }
}

#[derive(Debug)]
pub struct Request<'a> {
    pub method: &'a str,
    pub path: &'a str,
    pub http_version: &'a str,
}

impl<'a> Request<'a> {
    pub fn new(status_line: &'a str) -> Request {
        let v: Vec<&str> = status_line.rsplit(" ").collect();

        Request {
            method: v[2],
            path: v[1],
            http_version: v[0],
        }
    }
}

#[derive(Debug)]
pub struct App {
    pub paths: HashMap<String, String>,
}

impl App {
    pub fn new() -> App {
        let mut paths: HashMap<String, String> = HashMap::new();

        if let Ok(folders) = fs::read_dir(".") {
            folders
                .filter_map(|entry| entry.ok())
                .filter(|entry| {
                    let file_name = entry.file_name();
                    let file_name_str = file_name.to_string_lossy();

                    file_name_str != "src"
                        && file_name_str != "target"
                        && !file_name_str.contains(".")
                })
                .for_each(|folder| {
                    let folder_path = folder.path();

                    if let Ok(files) = fs::read_dir(&folder_path) {
                        files
                            .filter_map(|file_entry| file_entry.ok())
                            .for_each(|file_entry| {
                                if let Some(file_name) = file_entry.file_name().to_str() {
                                    let file_path = file_entry.path();
                                    if let Some(file_path_str) = file_path.to_str() {
                                        paths.insert(
                                            format!("/{}", file_name),
                                            file_path_str.to_string(),
                                        );
                                    }
                                }
                            });
                    }
                });
        }

        App { paths }
    }

    pub fn handle_request(&self, mut stream: TcpStream) {
        let start = SystemTime::now();

        let buf_reader = BufReader::new(&mut stream);

        let request_line = buf_reader.lines().next().unwrap().unwrap();

        let req = Request::new(&request_line);

        if let Some(filename) = self.paths.get(req.path) {
            let contents = fs::read_to_string(filename).unwrap();
            let length = contents.len();

            let response = format!("HTTP/1.1 200 OK\r\nContent-Length: {length}\r\n\r\n{contents}");
            stream.write_all(response.as_bytes()).unwrap();

            let end = SystemTime::now();

            let diff = end.duration_since(start).unwrap();
            println!(
                "{} {} {} - {}ms",
                req.method,
                req.path,
                req.http_version,
                diff.as_millis()
            );
        } else {
            let contents = fs::read_to_string("./pages/404.html").unwrap();
            let length = contents.len();

            let response = format!("HTTP/1.1 404 NOT FOUND\r\nContent-Length: {length}\r\n\r\n{contents}");
            stream.write_all(response.as_bytes()).unwrap();

            let end = SystemTime::now();

            let diff = end.duration_since(start).unwrap();
            println!(
                "{} {} {} - {}ms",
                req.method,
                req.path,
                req.http_version,
                diff.as_millis()
            );
        }
    }
}
