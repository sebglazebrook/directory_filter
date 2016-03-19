extern crate directory_filter;
extern crate directory_scanner;
extern crate time;
extern crate crossbeam;


use directory_scanner::ScannerBuilder;
use time::Tm;
use std::sync::mpsc::channel;
use std::sync::{Arc, Mutex};
use directory_filter::{SimpleFilter,ContinuousFilter};


#[test]
fn simple_filtering_example() {
    let mut scanner_builder = ScannerBuilder::new();
    scanner_builder = scanner_builder.start_from_path("tests/fixture_dir/");
    scanner_builder = scanner_builder.max_threads(1);
    let directory = scanner_builder.build().scan();

    let filter = SimpleFilter::new(&directory, "file-1");
    let filtered_directory = filter.execute();

    assert_eq!(filtered_directory.len(), 2);
}

#[test]
fn advanced_filtering_example() {

    let(trans_new_directory_item, rec_new_directory_item) = channel();

    let mut scanner_builder = ScannerBuilder::new();
    scanner_builder = scanner_builder.start_from_path("tests/fixture_dir/");
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

        trans_filter_change.send("file-1".to_string()).unwrap();

        let start_time = time::now();
        let duration = 5;
        let mut done = false;
        while !done {
            let found = rec_filter_match.recv().unwrap().len();
            if found == 2 {
                assert_eq!(found, 2);
                done = true;
            } else {
                if time_up(start_time, duration) {
                    assert_eq!(found, 2);
                    done = true;
                }
            }
        }
        let _ = finished_transmitter.send(true);
        drop(trans_filter_change);
        drop(scanner_builder);
    });
}

fn time_up(start_time: Tm, duration: i64) -> bool {
    let difference = time::now().to_timespec().sec - start_time.to_timespec().sec;
    difference > duration
}
