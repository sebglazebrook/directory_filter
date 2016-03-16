#[allow(dead_code)] // TODO remove after testing
extern crate directory_scanner;
extern crate regex;

use directory_scanner::ScannerBuilder;
use std::sync::mpsc::{Sender, Receiver};

mod directory_filter;
use directory_filter::{SimpleFilter, ContinuousFilter};

// used in the tests
use std::sync::mpsc::channel;


#[test]
fn simple_filtering_example() {
    let mut scanner_builder = ScannerBuilder::new();
    scanner_builder = scanner_builder.start_from_path("test/fixture_dir/");
    scanner_builder = scanner_builder.max_threads(1);
    let directory = scanner_builder.build().scan();

    let filter = SimpleFilter::new(&directory, "fixture_dir");
    let filtered_directory = filter.execute();

    assert_eq!(filtered_directory.len(), 10);
}

#[test]
fn advanced_filtering_example() {
    let(trans_new_directory_item, rec_new_directory_item) = channel();

    let mut scanner_builder = ScannerBuilder::new();
    scanner_builder = scanner_builder.start_from_path("test/fixture_dir/");
    scanner_builder = scanner_builder.max_threads(1);
    scanner_builder = scanner_builder.update_subscriber(trans_new_directory_item);
    let directory = scanner_builder.build().scan();

    let(trans_filter_change, rec_filter_change) = channel();
    let(trans_filter_match, rec_filter_match) = channel();

    let mut filter = ContinuousFilter::new(&directory, rec_filter_change, rec_new_directory_item, trans_filter_match);

    filter.start();

    trans_filter_change.send("fixture_dir".to_string()).unwrap();

    // listening for rescans and initiate rescan
    // need to listen for both new filter strings and directory changes
    // how will I stop the rescans?

    assert_eq!(rec_filter_match.recv().unwrap().len(), 10);

    filter.stop();
}
