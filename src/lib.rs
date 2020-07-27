use std::thread;
use uuid::Uuid;
use std::sync::{mpsc, Arc, Mutex};

pub struct ThreadPool {
    thread_workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

struct Worker {
    id: Uuid,
    thread: Option<thread::JoinHandle<()>>,
}

enum Message {
    NewJob(Job),
    Terminate,
}

type Job = Box<dyn FnOnce() + Send + 'static>;

impl Worker {
    fn new(guid: Uuid, receiver: Arc::<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || loop {
            let message = receiver.lock().unwrap().recv().unwrap();
            
            match message {
                Message::NewJob(job) => {
                    println!("Worker {} got a job; executing.", guid);
                    job();
                }
                Message::Terminate => {
                    println!("Worker {} instructed to terminate.", guid);
                    break;
                }
            }

        });
        Worker{ id: guid, thread: Some(thread) }
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
        assert!(size > 0, "Invalid size of ThreadPool provided to **ThreadPool()** line 10 in main.rs.");
        
        let mut threadpool = Vec::with_capacity(size);

        let (sender, receiver) = mpsc::channel();

        let receiver = Arc::new(Mutex::new(receiver));

        for _ in 0..size {
            let guid = Uuid::new_v4();
            threadpool.push(Worker::new(guid, Arc::clone(&receiver)));
        }

        ThreadPool { thread_workers: threadpool, sender }
    }

    pub fn execute<F>(&self, f: F)
    where
        F: FnOnce() + Send + 'static,
    {
        let job = Box::new(f);
        self.sender.send(Message::NewJob(job)).unwrap();
    }    
}

impl Drop for ThreadPool {
    fn drop(&mut self) {
        println!{"Sending terminate message to all workers."}

        for _ in &mut self.thread_workers {
            &mut self.sender.send(Message::Terminate).unwrap();
        }
        println!("Shutting down all workers.");

        for worker in &mut self.thread_workers {
            println!("Shutting down worker: {}", worker.id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }
    }
}
