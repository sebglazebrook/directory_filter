use regex::Regex;
use directory_scanner::Directory;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

pub fn find_matches(directory: &Arc<Mutex<Directory>>, regex: Regex) -> Vec<String> {
    //println!("Scanning a dir {:?}", directory);
    let mut matches = vec![];
    let locked_directory = directory.lock().unwrap();
    if is_match(&locked_directory.path, &regex) {
        matches.extend(locked_directory.contents());
    } else {
        for file in locked_directory.files.clone() {
            if is_match(&file.path(), &regex) {
                matches.push(file.as_string());
            }
        }
        for sub_directory in locked_directory.sub_directories.clone() {
            // TODO use a thread pool or something
            // for directories to make things quicker
            matches.extend(find_matches(&Arc::new(Mutex::new(sub_directory)), regex.clone()))
        }
    }
    matches
}

fn is_match(path: &PathBuf, regex: &Regex) -> bool {
    regex.is_match(path.to_str().unwrap())
}
