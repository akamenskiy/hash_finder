use std::sync::{Arc, Mutex, mpsc};
use std::thread;
use argh::FromArgs;
use sha256::digest;
use std::time::{Instant};

#[derive(FromArgs)]
/// get hash.
#[derive(Debug)]
struct Arguments {
    /// amount of zeroes
    #[argh(option, short = 'N')]
    n: usize,

    /// amount of hashes
    #[argh(option, short = 'F')]
    f: usize,
}

#[derive(Debug)]
struct HashPair(usize, String);

struct ThreadPool {
    workers: Vec<Worker>,
    sender: mpsc::Sender<Job>,
}


type Job = Box<dyn FnOnce() + Send + 'static>;

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

fn main() {
    let args: Arguments = argh::from_env();
    let start = Instant::now();
    run(args);
    let duration = start.elapsed();

    println!("Time elapsed in expensive_function() is: {:?}", duration);
}

fn run(args: Arguments) {
    let cores_amount = num_cpus::get();
    let pool = ThreadPool::new(cores_amount);
    let initial_state: Vec<HashPair> = Vec::new();
    let initial_sequence: (usize, usize) = (0, 0);
    let result = Arc::new(Mutex::new(initial_state));
    let numbers_found = Arc::new(Mutex::new(0));
    let current_number_sequence = Arc::new(Mutex::new(initial_sequence));
    while *numbers_found.lock().unwrap() < args.f {
        let numbers_found = Arc::clone(&numbers_found);
        let result = Arc::clone(&result);
        let current_number_sequence = Arc::clone(&current_number_sequence);
        let mut current_number_sequence = current_number_sequence.lock().unwrap();
        current_number_sequence.0 = current_number_sequence.1 + 1;
        current_number_sequence.1 = current_number_sequence.1 + 10000;
        let (start, end) = *current_number_sequence;
        pool.execute(move || {
            let mut current_state = result.lock().unwrap();
            for num in start..end {
                if *numbers_found.lock().unwrap() == args.f {
                    break;
                }
                let hash = digest(num.to_string());
                let is_hash_valid = hash.chars().rev().take(args.n).all(|char| char == '0');
                if is_hash_valid == true {
                    current_state.push(HashPair(num, hash));
                    let mut numbers_found = numbers_found.lock().unwrap();
                    *numbers_found += 1;
                }
            }
        });
    }
    let result = result.lock().unwrap();
    result.iter().for_each(|x| println!("{}: {}", x.0, x.1));
}

fn test_run(args: Arguments) {
    let initial_state: Vec<HashPair> = Vec::new();
    let mut result = initial_state;
    let mut numbers_found = 0;
    let mut current_number = 1;
    while numbers_found < args.f {
        let hash = digest(current_number.to_string());
        let is_hash_valid = hash.chars().rev().take(args.n).all(|char| char == '0');
        if is_hash_valid == true {
            result.push(HashPair(current_number, hash));
            numbers_found += 1;
        }
        current_number += 1;
    }
    result.iter().for_each(|x| println!("{}: {}", x.0, x.1));
}