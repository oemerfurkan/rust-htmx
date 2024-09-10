use std::{
    collections::HashMap,
    fs,
    sync::{mpsc, Arc, Mutex},
    thread,
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

        if let Ok(files) = fs::read_dir("./pages") {
            for file in files {
                if let Ok(entry) = file {
                    if let Some(file_name) = entry.file_name().to_str() {
                        if let Some(file_path) = entry.path().to_str() {
                            paths.insert(file_path.to_string()[1..].to_string(), file_name.to_string());
                        }
                    }
                }
            }
        }

        App { paths }
    }
}
