use std::sync::Arc;
use std::sync::Mutex;
use std::thread;
use std::sync::mpsc;

trait FnBox {
    fn call_box(self: Box<Self>);
}

type Job = Box<dyn FnBox + Send + 'static>;

enum Message {
    NewJob(Job),
    Terminate,
}

pub struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Message>,
}

impl<F: FnOnce()> FnBox for F {
    fn call_box(self: Box<F>) {
        (*self)()
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
        let (sender, receiver) = mpsc::channel::<Message>();
        let mut workers = Vec::with_capacity(size);
        let receiver = Arc::new(Mutex::new(receiver));
        for _id in 0..size {
            workers.push(Worker::new(_id, Arc::clone(&receiver)));
        }

        ThreadPool { workers, sender, }
    }

    /// Execute a job associated with a thread.
    pub fn execute<F>(&self, f: F)
        where F: FnOnce() + Send + 'static
        {
            let job = Box::new(f);
            self.sender.send(Message::NewJob(job)).unwrap();
        }

    /// Signal shutdown and wait for the pool to exit.
    pub fn wait(&mut self) {
        //println!("Sending terminate message to all workers.");
        for _ in &mut self.workers {
            self.sender.send(Message::Terminate).unwrap();
        }

        //println!("Shutting all workers");
        for worker in &mut self.workers {
            //println!("Shutting down worker {}", worker._id);

            if let Some(thread) = worker.thread.take() {
                thread.join().unwrap();
            }
        }

        // Remove workers, required when explicitly called,
        // since destructor Done will be implicitly called.
        self.workers.clear();
    }
}

impl Drop for ThreadPool {
    /// Cleanly signal threads to stop and drop them from the pool.
    fn drop(&mut self) {
        self.wait();
    }
}

struct Worker {
    _id: usize,
    thread: Option<thread::JoinHandle<()>>,
}

impl Worker {
    fn new(_id: usize, receiver: Arc<Mutex<mpsc::Receiver<Message>>>) -> Worker {
        let thread = thread::spawn(move || {
            loop {
                let message = receiver
                    .lock().unwrap()
                    .recv().unwrap();
                match message {
                    Message::NewJob(job) => {
                        //println!("Worker {} got a job; executing.", _id);
                        job.call_box();
                    },
                    Message::Terminate => {
                        //println!("Worker {} was told to terminate.", _id);
                        break;
                    },
                }
            }
        });

        Worker { _id, thread: Some(thread), }
    }
}

