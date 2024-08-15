mod logic;
use crate::logic::fork_join::launch_threads_to_process;
use logic::generate_final_report::generate_final_report;
use logic::pre_processing::{get_number_threads, get_vector_of_path_file};
mod data_types;

use crate::data_types::report::get_report_base;
use std::fs;

/// Receives a result with the vector of strings of the route (or a string with the
/// error message in case of error), and the number of threads, delegates
/// the processing of the files and the generation of the final report
fn process_final_all_sites(paths_files: Result<Vec<String>, String>, number_threads: &usize) {
    let mut report_final = get_report_base();
    match paths_files {
        Ok(vec_content) => {
            launch_threads_to_process(vec_content, &mut report_final.sites, number_threads);
        }
        Err(e) => {
            eprintln!("[ERROR]: {} ", e);
        }
    }
    generate_final_report(report_final);
}
/// It is responsible for opening the data directory and delegates the parsing logic to obtain the number of threads
/// and delegates processing for all files In case of error
/// prints a message with the error and terminates the program.
fn process_information() {
    let number_threads: usize = get_number_threads();
    let paths = fs::read_dir("./data");
    match paths {
        Ok(paths_content) => {
            let files_names = get_vector_of_path_file(paths_content);
            process_final_all_sites(files_names, &number_threads);
        }
        Err(e) => {
            eprintln!("[ERROR]: to open the directory: {}", e)
        }
    }
}
fn main() {
    process_information();
}
