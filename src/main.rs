extern crate directory_filter;
extern crate directory_scanner;
extern crate crossbeam;

use std::io;
use directory_scanner::ScannerBuilder;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use directory_filter::ContinuousFilter;

fn main() {
    let(trans_new_directory_item, rec_new_directory_item) = channel();

    let mut scanner_builder = ScannerBuilder::new();
    scanner_builder = scanner_builder.start_from_path("./");
    scanner_builder = scanner_builder.max_threads(1);
    scanner_builder = scanner_builder.update_subscriber(trans_new_directory_item);
    let directory = scanner_builder.build().scan();

    let(trans_filter_change, rec_filter_change) = channel();
    let(trans_filter_match, rec_filter_match) = channel();

    let mut filter = ContinuousFilter::new(&directory, Arc::new(Mutex::new(rec_filter_change)), Arc::new(Mutex::new(rec_new_directory_item)), Arc::new(Mutex::new(trans_filter_match)));

    crossbeam::scope(|scope| {

        let finished_transmitter = filter.finished_transmitter.clone();
        scope.spawn(move || {
            filter.start();
        });

        let found = rec_filter_match.recv().unwrap();
        println!("Found {} files", found.matches.len());

        let mut done = false;
        let mut input = String::new();
        while !done {
            match io::stdin().read_line(&mut input) {
                Ok(_) => {
                    let last_line = input.lines().last().unwrap();
                    if input.lines().last().unwrap() == "exit" {
                        done = true;
                    } else {
                        trans_filter_change.send(last_line.to_string()).unwrap();
                        let found = rec_filter_match.recv().unwrap();
                        println!("Found: {:?}", found.matches);
                    }
                }
                Err(error) => { println!("error: {}", error); }
            }
        }
        println!("Finished");

        let _ = finished_transmitter.send(true);
        drop(trans_filter_change);
        drop(scanner_builder);
    });
}
