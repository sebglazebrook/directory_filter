use regex::Regex;
use directory_scanner::{Directory, File};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use crossbeam::sync::SegQueue;
use scoped_threadpool::Pool;

pub fn find_matches(directory: &Arc<Mutex<Directory>>, regex: Regex) -> Vec<File> {
    execute(directory, regex)
}

pub fn find_file_matches(files: &Vec<File>, regex: Regex) -> Vec<File> {
    let file_matches_queue = Arc::new(SegQueue::new());
    let mut pool = Pool::new(8);
    pool.scoped(|scoped| {
        for file in files {
        let local_regex = regex.clone();
        let local_file_matches_queue = file_matches_queue.clone();
            scoped.execute(move || {
                if is_string_match(file.as_string(), &local_regex) {
                    local_file_matches_queue.push(file.clone());
                }
            });
        }
    });
    let mut file_merged_matches = vec![];
    let mut done = false;
    while !done {
        match file_matches_queue.try_pop() {
            Some(matches) => { file_merged_matches.push(matches); }
            None => { done = true }
        }
    }
    file_merged_matches
}

//----------- private -------------//

fn execute(directory: &Arc<Mutex<Directory>>, regex: Regex) -> Vec<File> {
    let file_matches_queue = Arc::new(SegQueue::new());
    let current_concurrency = Arc::new(AtomicUsize::new(0));
    let concurrency_limit = Arc::new(AtomicUsize::new(4));
    fetch_matches(directory.clone(), regex.clone(), file_matches_queue.clone(), current_concurrency.clone(), concurrency_limit.clone());
    let mut file_merged_matches = vec![];
    let mut done = false;
    while !done {
        match file_matches_queue.try_pop() {
            Some(matches) => { file_merged_matches.extend(matches); }
            None => {
                if current_concurrency.load(Ordering::Relaxed) == 0 {
                    done = true
                }
            }
        }
    }
    file_merged_matches
}


fn fetch_matches(directory: Arc<Mutex<Directory>>, regex: Regex, file_matches_queue: Arc<SegQueue<Vec<File>>>, current_concurrency: Arc<AtomicUsize>, concurrency_limit: Arc<AtomicUsize>) {
    let locked_directory = directory.lock().unwrap();
    if is_string_match(locked_directory.path.to_str().unwrap().to_string(), &regex) {
        file_matches_queue.push(locked_directory.files.clone());
    } else {
        for file in locked_directory.files.clone() {
            if is_string_match(file.as_string(), &regex) {
                file_matches_queue.push(vec![file.clone()]);
            }
        }
        for sub_directory in locked_directory.sub_directories.clone() {
            if max_concurrency_reached(current_concurrency.clone(), concurrency_limit.clone()) {
                fetch_matches(Arc::new(Mutex::new(sub_directory)), regex.clone(), file_matches_queue.clone(), current_concurrency.clone(), concurrency_limit.clone());
            } else {
                let local_current_concurrency = current_concurrency.clone();
                let local_concurrency_limit = concurrency_limit.clone();
                let local_regex = regex.clone();
                let local_file_matches_queue = file_matches_queue.clone();
                thread::spawn(move || {
                    local_current_concurrency.fetch_add(1, Ordering::SeqCst);
                    info!("Increased filter concurrency to {:?}", local_current_concurrency.load(Ordering::Relaxed));
                    fetch_matches(Arc::new(Mutex::new(sub_directory)), local_regex, local_file_matches_queue, local_current_concurrency.clone(), local_concurrency_limit);
                    local_current_concurrency.fetch_sub(1, Ordering::SeqCst);
                    info!("Decreased filter concurrency to {:?}", local_current_concurrency.load(Ordering::Relaxed));
                });
            }
        }
    }
}

fn is_string_match(path: String, regex: &Regex) -> bool {
    regex.is_match(&path)
}

fn max_concurrency_reached(current_concurrency: Arc<AtomicUsize>, concurrency_limit: Arc<AtomicUsize>) -> bool {
    current_concurrency.load(Ordering::Relaxed) >= concurrency_limit.load(Ordering::Relaxed)
}
