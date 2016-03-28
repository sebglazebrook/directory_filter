use regex::Regex;
use directory_scanner::Directory;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use std::thread;
use crossbeam::sync::SegQueue;

pub fn find_matches(directory: &Arc<Mutex<Directory>>, regex: Regex) -> Vec<String> {
    execute(directory, regex)
}

//----------- private -------------//

fn execute(directory: &Arc<Mutex<Directory>>, regex: Regex) -> Vec<String> {
    let matches_queue = Arc::new(SegQueue::new());
    let current_concurrency = Arc::new(AtomicUsize::new(0));
    let concurrency_limit = Arc::new(AtomicUsize::new(4));
    fetch_matches(directory.clone(), regex.clone(), matches_queue.clone(), current_concurrency.clone(), concurrency_limit.clone());
    let mut merged_matches = vec![];
    let mut done = false;
    while !done {
        match matches_queue.try_pop() {
            Some(matches) => { merged_matches.extend(matches); }
            None => {
                if current_concurrency.load(Ordering::Relaxed) == 0 {
                    done = true
                }
            }
        }
    }
    merged_matches
}


fn fetch_matches(directory: Arc<Mutex<Directory>>, regex: Regex, matches_queue: Arc<SegQueue<Vec<String>>>, current_concurrency: Arc<AtomicUsize>, concurrency_limit: Arc<AtomicUsize>) {
    let locked_directory = directory.lock().unwrap();
    if is_match(&locked_directory.path, &regex) {
        matches_queue.push(locked_directory.contents());
    } else {
        for file in locked_directory.files.clone() {
            if is_match(&file.path(), &regex) {
                matches_queue.push(vec![file.as_string()]);
            }
        }
        for sub_directory in locked_directory.sub_directories.clone() {
            if max_concurrency_reached(current_concurrency.clone(), concurrency_limit.clone()) {
                fetch_matches(Arc::new(Mutex::new(sub_directory)), regex.clone(), matches_queue.clone(), current_concurrency.clone(), concurrency_limit.clone());
            } else {
                let local_current_concurrency = current_concurrency.clone();
                let local_concurrency_limit = concurrency_limit.clone();
                let local_regex = regex.clone();
                let local_matches_queue = matches_queue.clone();
                thread::spawn(move || {
                    local_current_concurrency.fetch_add(1, Ordering::SeqCst);
                    info!("Increased filter concurrency to {:?}", local_current_concurrency.load(Ordering::Relaxed));
                    fetch_matches(Arc::new(Mutex::new(sub_directory)), local_regex, local_matches_queue, local_current_concurrency.clone(), local_concurrency_limit);
                    local_current_concurrency.fetch_sub(1, Ordering::SeqCst);
                    info!("Decreased filter concurrency to {:?}", local_current_concurrency.load(Ordering::Relaxed));
                });
            }
        }
    }
}

fn is_match(path: &PathBuf, regex: &Regex) -> bool {
    regex.is_match(path.to_str().unwrap())
}

fn max_concurrency_reached(current_concurrency: Arc<AtomicUsize>, concurrency_limit: Arc<AtomicUsize>) -> bool {
    current_concurrency.load(Ordering::Relaxed) >= concurrency_limit.load(Ordering::Relaxed)
}
