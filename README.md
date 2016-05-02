#Overview

This is a rust library that handles the filtering of results that are returned from another rust library [directory scanner](https://github.com/sebglazebrook/directory_scanner)

#Usage

See the tests for usage for the time being.

# TODO

- keep track of matches using the same directory structure not just Vec<File>
- a FilteredDirectory should be sorted alphabetically and by string length
- when finding a matching filter, updates should be made on each match, not at the end
