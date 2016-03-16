use directory_scanner::Directory;

pub struct FilteredDirectory<'b> {
    pub directory: &'b Directory,
    pub matches: Vec<String>, // TODO this should be a collection of references/pointers to paths in the directory
}

impl<'b> FilteredDirectory<'b> {

    pub fn len(&self) -> usize {
        self.matches.len()
    }

}
