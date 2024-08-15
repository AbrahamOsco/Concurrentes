//! Module that contains all the logic to carry out the practical work

//! - `fork_join`: Module that contains the logic to perform the fork join
//!      and the calculation of chunks to use.
//! - `generate_final_report`: Module that contains the logic to accumulate the statistics
//!      totals (using all SiteStats structures to get: tags, chatty_site top10 and chatty_tag)
//!      and print the final report in json format.
//! - `pre_processing`: Module that contains the logic to obtain the number of threads and the vector
//!      of strings with the paths of all the files.
//! - `processing_file`: Module that contains the logic to read/process each line of a file
//!     .jsonl and get the vector of tuples (name_site, SiteStats). which contains all the statistics
//!     individual for each file.
pub mod fork_join;
pub mod generate_final_report;
pub mod pre_processing;
pub mod processing_file;
