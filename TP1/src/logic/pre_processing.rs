use std::env;
use std::fs::ReadDir;

/// Parse when executing the application ("cargo run -- a) to obtain the amount of threads,
/// returns the number of threads, if not passed the number of threads, returns 1
pub fn get_number_threads() -> usize {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        eprintln!(
            "[ERROR] you need input the number of threads, the execution continues with 1 thread"
        );
        return 1;
    }
    match args[1].parse() {
        Ok(padron_content) => padron_content,
        Err(e) => {
            eprintln!("[ERROR]: In the parse from the number of threads: {}", e);
            1
        }
    }
}

/// Receives a ReadDir struct by movement, returns in a vector of strings the paths
/// of the files that contain a ".jsonl", if there was an error it returns an error message.
pub fn get_vector_of_path_file(paths_results: ReadDir) -> Result<Vec<String>, String> {
    let mut paths_files = Vec::new();
    for a_path in paths_results {
        match a_path {
            Ok(path_content) => {
                let file_name = path_content.path().display().to_string();
                if file_name.contains(".jsonl") {
                    paths_files.push(file_name);
                }
            }
            Err(e) => {
                eprintln!(" [ERROR]: to read a name of a file: {} ", e)
            }
        }
    }
    if paths_files.is_empty() {
        Err("[ERROR]: There's no files in directory".to_string())
    } else {
        Ok(paths_files)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    #[test]
    fn test_read_the_data_mock_should_obtain_a_vector_of_size_2_for_two_files_that_exists() {
        match fs::read_dir("./data_mock") {
            Ok(dir_content) => match get_vector_of_path_file(dir_content) {
                Ok(vec_files_content) => {
                    assert_eq!(2, vec_files_content.len());
                }
                Err(_) => {}
            },
            Err(_) => {}
        }
    }
}
