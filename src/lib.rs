use argh::FromArgs;
use sha256::digest;
use std::sync::mpsc::Sender;
use std::sync::{mpsc, Arc, Mutex};
use std::thread;

const CHUNK_SIZE: usize = 10;

#[derive(FromArgs)]
/// get hash.
#[derive(Debug)]
pub struct Arguments {
    /// amount of zeroes
    #[argh(option, short = 'N')]
    pub n: usize,

    /// amount of hashes
    #[argh(option, short = 'F')]
    pub f: usize,
}

#[derive(Debug, PartialEq)]
pub struct HashPair(usize, String);

pub fn find_hashes(n: usize, f: usize) -> Vec<HashPair> {
    let chunk_idx = Arc::new(Mutex::new(0));
    let cores_amount = num_cpus::get();
    let mut results = vec![];

    let (tx, rx) = mpsc::channel();

    for _ in 0..cores_amount {
        let chunk_idx = chunk_idx.clone();
        let tx = tx.clone();
        thread::spawn(move || worker(chunk_idx, n, tx));
    }

    while let Ok(result) = rx.recv() {
        results.push(result);
        if results.len() == f {
            break;
        }
    }
    return results;
}

fn worker(chunk_idx: Arc<Mutex<usize>>, n: usize, tx: Sender<HashPair>) {
    loop {
        let mut guard = chunk_idx.lock().unwrap();
        let chunk_idx = *guard;
        *guard += 1;
        drop(guard);
        let range = (chunk_idx * CHUNK_SIZE)..((chunk_idx + 1) * CHUNK_SIZE);
        for x in range {
            let hash = digest(&x.to_string());

            let is_hash_valid = hash.chars().rev().take(n).all(|char| char == '0');
            if is_hash_valid == true {
                tx.send(HashPair(x, hash)).unwrap();
            }
        }
    }
}

pub fn run(args: Arguments) {
    for HashPair(number, hash) in find_hashes(args.n, args.f) {
        println!("{number}: {hash}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn n_3_f_6() {
        let test_vec = vec![
            HashPair(
                4163,
                String::from("95d4362bd3cd4315d0bbe38dfa5d7fb8f0aed5f1a31d98d510907279194e3000"),
            ),
            HashPair(
                11848,
                String::from("cb58074fd7620cd0ff471922fd9df8812f29f302904b15e389fc14570a66f000"),
            ),
            HashPair(
                12843,
                String::from("bb90ff93a3ee9e93c123ebfcd2ca1894e8994fef147ad81f7989eccf83f64000"),
            ),
            HashPair(
                13467,
                String::from("42254207576dd1cfb7d0e4ceb1afded40b5a46c501e738159d8ac10b36039000"),
            ),
            HashPair(
                20215,
                String::from("1f463eb31d6fa7f3a7b37a80f9808814fc05bf10f01a3f653bf369d7603c8000"),
            ),
            HashPair(
                28892,
                String::from("dab12874ecae90c0f05d7d87ed09921b051a586c7321850f6bb5e110bc6e2000"),
            ),
        ];
        let mut result = find_hashes(3, 6);
        result.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(test_vec, result);
    }

    #[test]
    fn n_5_f_3() {
        let test_vec = vec![
            HashPair(
                828028,
                String::from("d95f19b5269418c0d4479fa61b8e7696aa8df197082b431a65ff37595c100000"),
            ),
            HashPair(
                2513638,
                String::from("862d4525b0b60779d257be2b3920b90e3dbcd60825b86cfc6cffa49a63c00000"),
            ),
            HashPair(
                3063274,
                String::from("277430daee71c67b356dbb81bb0a39b6d53efd19d14177a173f2e05358a00000"),
            ),
        ];
        let mut result = find_hashes(5, 3);
        result.sort_by(|a, b| a.0.cmp(&b.0));
        assert_eq!(test_vec, result);
    }
}
