use directory_scanner::Directory;
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct FilteredDirectory {
    pub directory: Arc<Mutex<Directory>>,
    pub matches: Vec<String>, // TODO this should be a collection of references/pointers to paths in the directory
    // TODO gonna need to store the filterstring/regex here
}

impl FilteredDirectory {

    pub fn len(&self) -> usize {
        self.matches.len()
    }

}
