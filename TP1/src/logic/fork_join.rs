use crate::data_types::site_stats::SiteStats;
use crate::logic::processing_file::procces_a_vec_files_paths;
use std::collections::HashMap;

/// Receives a reference number of threads and a reference to the size of the number of files,
/// if the number of threads is greater than the number of files the chunk size will be 1.
/// Otherwise it returns the quotient (applying rounding down)
/// between the number of files/number of threads.
pub fn get_chunk_size(number_threads: &usize, size_vec_all_sites_names: &usize) -> usize {
    let mut new_number_threads = *number_threads;
    if new_number_threads > *size_vec_all_sites_names {
        new_number_threads = *size_vec_all_sites_names;
    }
    (*size_vec_all_sites_names as f32 / new_number_threads as f32).floor() as usize
}

/// Receives the vector (by movement) with the file path of all the files, the hashmap
/// of the sites (a reference mutable ) and a number of threads (by reference).
/// (Basically performs the fork join) It divides the files paths vector into chunks,
/// and launches a thread for each chunk to process on that chunk (it processes
/// on the files paths vector associated with that chunk), then the joins are
/// performed to obtain the results obtained by each thread and join them
/// into the site hashmap passed as an argument.

pub fn launch_threads_to_process(
    vec_all_files_paths: Vec<String>,
    all_site_stats: &mut HashMap<String, SiteStats>,
    number_threads: &usize,
) {
    let mut join_handlers = vec![];
    let chunks_sites_name =
        vec_all_files_paths.chunks(get_chunk_size(number_threads, &vec_all_files_paths.len()));
    for a_chuck in chunks_sites_name {
        let cloned_vec = a_chuck.to_vec();
        join_handlers.push(std::thread::spawn(move || {
            procces_a_vec_files_paths(cloned_vec)
        }))
    }

    for handler in join_handlers {
        match handler.join() {
            Ok(join_content) => {
                for (site_name, site_stats) in join_content {
                    all_site_stats.insert(site_name, site_stats);
                }
            }
            Err(e) => {
                eprintln!("[ERROR] getting the file_stats : {:?}", e);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data_types::tag_stats::TagStats;
    fn get_mock_1_stats_site() -> SiteStats {
        let mut tags_mock: HashMap<String, TagStats> = HashMap::new();
        tags_mock.insert(
            "chiste".to_string(),
            TagStats {
                questions: 3,
                words: 26,
            },
        );
        tags_mock.insert(
            "random".to_string(),
            TagStats {
                questions: 1,
                words: 8,
            },
        );
        tags_mock.insert(
            "other3".to_string(),
            TagStats {
                questions: 2,
                words: 18,
            },
        );
        let stats_mock = SiteStats {
            questions: 3,
            words: 26,
            tags: tags_mock,
            chatty_tags: vec![
                "other3".to_string(),
                "chiste".to_string(),
                "random".to_string(),
            ],
        };
        stats_mock
    }
    fn get_mock_2_stats_site() -> SiteStats {
        let mut tags_mock2: HashMap<String, TagStats> = HashMap::new();
        tags_mock2.insert(
            "chiste".to_string(),
            TagStats {
                questions: 1,
                words: 7,
            },
        );
        tags_mock2.insert(
            "pokemon".to_string(),
            TagStats {
                questions: 2,
                words: 26,
            },
        );

        let stats_mock_site_2 = SiteStats {
            questions: 3,
            words: 33,
            tags: tags_mock2,
            chatty_tags: vec!["pokemon".to_string(), "chiste".to_string()],
        };
        stats_mock_site_2
    }
    #[test]
    fn test_1_threads_are_entered_to_process_and_there_88_files_the_size_of_each_chunk_should_be_88_applying_rounding_down_floor(
    ) {
        let number_threads = 1;
        let mut vec_all_files_name = vec![];
        for i in 0..88 {
            vec_all_files_name.push(format!("file_nro_{}.jsonl", i));
        }
        let size_chunk = get_chunk_size(&number_threads, &vec_all_files_name.len());
        assert_eq!(88, size_chunk);
    }
    #[test]
    fn test_3_threads_are_entered_to_process_and_there_88_files_the_size_of_each_chunk_should_be_29_applying_rounding_down_floor(
    ) {
        let number_threads = 3;
        let mut vec_all_files_name = vec![];
        for i in 0..88 {
            vec_all_files_name.push(format!("file_nro_{}.jsonl", i));
        }
        let size_chunk = get_chunk_size(&number_threads, &vec_all_files_name.len());
        assert_eq!(29, size_chunk);
    }

    #[test]
    fn test_5_threads_are_entered_to_process_and_there_88_files_the_size_of_each_chunk_should_be_17_applying_rounding_down_floor(
    ) {
        let number_threads = 5;
        let mut vec_all_files_name = vec![];
        for i in 0..88 {
            vec_all_files_name.push(format!("file_nro_{}.jsonl", i));
        }
        let size_chunk = get_chunk_size(&number_threads, &vec_all_files_name.len());
        assert_eq!(17, size_chunk);
    }
    #[test]
    fn test_100_threads_are_entered_to_process_and_there_88_files_the_size_of_each_chunk_should_be_1_applying_rounding_down_floor(
    ) {
        let number_threads = 100;
        let mut vec_all_files_name = vec![];
        for i in 0..88 {
            vec_all_files_name.push(format!("file_nro_{}.jsonl", i));
        }
        let size_chunk = get_chunk_size(&number_threads, &vec_all_files_name.len());
        assert_eq!(1, size_chunk);
    }

    #[test]
    fn test_given_a_total_vector_of_filesnames_the_number_of_words_questions_associated_with_each_site_and_tag_and_the_chatty_tag_must_be_obtained_correctly_using_2_threads(
    ) {
        let number_threads = 2;
        let mut hash_sities = HashMap::new();
        let mut hash_sities_mock = HashMap::new();
        let vec_all_files_name = vec![
            "data_mock/mock1.jsonl".to_string(),
            "data_mock/mock2.jsonl".to_string(),
        ];

        launch_threads_to_process(vec_all_files_name, &mut hash_sities, &number_threads);
        let stats_mock_site_1 = get_mock_1_stats_site();
        let stats_mock_site_2 = get_mock_2_stats_site();

        hash_sities_mock.insert("mock1.jsonl".to_string(), stats_mock_site_1);
        hash_sities_mock.insert("mock2.jsonl".to_string(), stats_mock_site_2);

        assert_eq!(hash_sities_mock.len(), hash_sities.len());
        for (site_name, site_stats) in &hash_sities {
            if hash_sities_mock.get(site_name) != Some(site_stats) {
                assert!(false, "[ERROR] the site stats are not equals")
            }
        }
    }
}
