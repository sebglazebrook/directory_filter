use regex::Regex;
use directory_scanner::Directory;
use std::path::PathBuf;

pub fn find_matches(directory: &Directory, regex: Regex) -> Vec<String> {
    let mut matches = vec![];
    if is_match(&directory.path, &regex) {
        matches.extend(directory.contents());
    } else {
        for file in directory.files.clone() {
            if is_match(&file.path(), &regex) {
                matches.push(file.as_string());
            }
        }
        for sub_directory in directory.sub_directories.clone() {
            // TODO use a thread pool or something
            // for directories to make things quicker
            matches.extend(find_matches(&sub_directory, regex.clone()))
        }
    }
    matches
}

fn is_match(path: &PathBuf, regex: &Regex) -> bool {
    regex.is_match(path.to_str().unwrap())
}
