use crate::data_types::data_input_jsonl::DataInputJsonl;
use crate::data_types::site_stats::SiteStats;
use crate::data_types::tag_stats::TagStats;
use std::collections::HashMap;
use std::fs::File;
use std::io::{BufRead, BufReader};

/// Receives a line in string format from the .jsonl file and a reference
/// mutable SiteStats structure maps the string to a structure
/// DataInputJsonl and calculates the stats and accumulates them in the SiteStats struct.
/// In case of error, the problematic line is printed by stderr.
fn update_hashmaps_stats(line_response: String, site_stats: &mut SiteStats) {
    match serde_json::from_str::<DataInputJsonl>(&line_response) {
        Ok(jsonl_content) => {
            let mut number_of_words = 0;
            for text in jsonl_content.texts {
                number_of_words += text.split_whitespace().count() as u32;
            }
            site_stats.questions += 1;
            site_stats.words += number_of_words;
            for tag in jsonl_content.tags {
                let tag_obtained = site_stats.tags.entry(tag).or_insert(TagStats {
                    questions: 0,
                    words: 0,
                });
                tag_obtained.questions += 1;
                tag_obtained.words += number_of_words;
            }
        }
        Err(e) => {
            eprintln!(
                "[Error] : error trying to parse the {} to a DataInputJsonl {}",
                line_response, e
            );
        }
    }
}

/// Receives a File structure (by movement), reads each line of the file and delegates the
/// processing the stats of each jsonl line, accumulating the results
/// in the SiteStats structure, when all the lines are finished, the SiteStats is returned
fn get_hashmaps_stats(file_result: File) -> SiteStats {
    let mut site_stats = SiteStats {
        questions: 0,
        words: 0,
        tags: HashMap::new(),
        chatty_tags: vec![],
    };
    let reader = BufReader::new(file_result);
    for line in reader.lines() {
        match line {
            Ok(line_content) => {
                update_hashmaps_stats(line_content, &mut site_stats);
            }
            Err(e) => {
                eprintln!("[ERROR] in get_hashmaps_stats() the error is : {}", e)
            }
        }
    }
    site_stats.chatty_tags = get_top10_tags_from_hashmap(&site_stats.tags)
        .iter()
        .map(|(a, _)| a.clone())
        .collect();
    site_stats
}

/// Receives a hashmap by reference whose key is a string and the value is a TagStats structure
/// transforms it into a vector of tuples (taking the quotient between words/question (using TagStats))
/// returns vector ordering by quotient in descending order with only the first 10 elements.
pub fn get_top10_tags_from_hashmap(hashmap_tag: &HashMap<String, TagStats>) -> Vec<(String, f32)> {
    let mut top10_tag_from_a_vec: Vec<(String, f32)> = hashmap_tag
        .iter()
        .map(|(tag_name, tag_stats)| {
            (
                tag_name.clone(),
                tag_stats.words as f32 / tag_stats.questions as f32,
            )
        })
        .collect();
    sort_vec_and_set_top10(&mut top10_tag_from_a_vec);
    top10_tag_from_a_vec
}

/// Receives a mutable reference of a 2-element tuple vector,
/// the first element is a string and the second is a float:i32
/// sorts the vector by the second element (float) (in case of a tie,
/// it sorts by the string (name) in ascending order) in descending order and
/// truncates it with the first 10 elements (top 10).
pub fn sort_vec_and_set_top10(a_vec: &mut Vec<(String, f32)>) {
    a_vec.sort_by(|a, b| match b.1.partial_cmp(&a.1) {
        Some(a_ordering) => {
            if a_ordering == std::cmp::Ordering::Equal {
                match a.0.partial_cmp(&b.0) {
                    Some(string_cmp) => string_cmp,
                    None => std::cmp::Ordering::Equal,
                }
            } else {
                a_ordering
            }
        }
        None => std::cmp::Ordering::Equal,
    });
    a_vec.truncate(10);
}
/// Receives a file path by reference delegates file processing,
/// parses the route obtaining the name of the site finally
/// returns a tuple with the name of the site.jsonl and a siteStats structure
///, in case of error returns a string with the error message
fn get_file_stats(file_path: &String) -> Result<(String, SiteStats), String> {
    let file = File::open(file_path);
    match file {
        Ok(file_response) => {
            let file_stats = get_hashmaps_stats(file_response);
            let file_name_parsed: Vec<&str> = file_path.split('/').collect();
            match file_name_parsed.last() {
                Some(file_name_content) => Ok((file_name_content.to_string(), file_stats)),
                None => Err(format!(
                    "[ERROR]: With the parse of the file name: {} ",
                    file_path
                )),
            }
        }
        Err(e) => Err(format!("[ERROR]: the file {} can't open {}", file_path, e)),
    }
}

/// Receives a vector of the paths files (by movement) to be processed iterates each
/// path file and delegates its processing, then joins the results into a vector
/// of tuples ( (String, SiteStats) this tuple has the name of the site and
/// the SiteStats structure) and then returns it.
pub fn procces_a_vec_files_paths(vec_files_paths: Vec<String>) -> Vec<(String, SiteStats)> {
    let mut vec_file_stats: Vec<(String, SiteStats)> = vec![];
    for file_name in vec_files_paths.iter() {
        let file_stats = get_file_stats(file_name);
        match file_stats {
            Ok((site_name_content, site_stats_content)) => {
                vec_file_stats.push((site_name_content, site_stats_content));
            }
            Err(e) => {
                eprintln!("[ERROR] getting the file_stats : {}", e);
            }
        }
    }
    vec_file_stats
}

#[cfg(test)]
mod tests {
    use super::*;
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
    fn test_given_a_vector_of_tuples_with_2_string_elements_and_floats_it_should_be_sorted_by_the_floats_descendingly_and_in_case_of_a_tie_it_should_be_sorted_by_the_string_ascendingly(
    ) {
        let mut vec_mock = vec![
            ("Sosa".to_string(), 30.0),
            ("Juan".to_string(), 30.0),
            ("Jose".to_string(), 100.0),
            ("Maria".to_string(), 80.0),
        ];
        let ordered_vec = vec![
            ("Jose".to_string(), 100.0),
            ("Maria".to_string(), 80.0),
            ("Juan".to_string(), 30.0),
            ("Sosa".to_string(), 30.0),
        ];
        sort_vec_and_set_top10(&mut vec_mock);
        assert_eq!(ordered_vec, vec_mock);
    }
    #[test]
    fn test_given_two_mock_lines_the_number_of_total_words_and_the_total_number_associated_with_each_tag_should_be_correctly_obtained(
    ) {
        let mut site_stats = SiteStats {
            questions: 0,
            words: 0,
            tags: HashMap::new(),
            chatty_tags: vec![],
        };
        let line_mock_1 =
            r#"{"texts": ["¿Cuanto es 2+2 pepe?", "La respuesta es 4"], "tags": ["chiste", "random", "other3"]}"#
                .to_string();
        let line_mock_2 =
            r#"{"texts": ["¿Donde vives?", "La respuesta es en mi casa en flores"], "tags": ["chiste"]}"#
                .to_string();
        update_hashmaps_stats(line_mock_1, &mut site_stats);
        update_hashmaps_stats(line_mock_2, &mut site_stats);
        let tag_stats_chiste = site_stats.tags.get("chiste");
        let tag_stats_random = site_stats.tags.get("random");
        let tag_stats_other3 = site_stats.tags.get("other3");

        assert_eq!(2, site_stats.questions);
        assert_eq!(18, site_stats.words);
        assert_eq!(3, site_stats.tags.len());
        match tag_stats_chiste {
            Some(tag_content) => {
                assert_eq!(2, tag_content.questions);
                assert_eq!(18, tag_content.words);
            }
            None => {
                assert!(
                    false,
                    "[ERROR] Trying to obtain a content of stats from a linea of jsonl"
                )
            }
        }
        match tag_stats_random {
            Some(tag_content) => {
                assert_eq!(1, tag_content.questions);
                assert_eq!(8, tag_content.words);
            }
            None => {
                assert!(
                    false,
                    "[ERROR] Trying to obtain a content of stats stats from a linea of jsonl"
                )
            }
        }
        match tag_stats_other3 {
            Some(tag_content) => {
                assert_eq!(1, tag_content.questions);
                assert_eq!(8, tag_content.words);
            }
            None => {
                assert!(
                    false,
                    "[ERROR] Trying to obtain a content of stats stats from a linea of jsonl"
                )
            }
        }
    }

    #[test]
    fn test_given_a_jsonl_the_number_of_words_questions_tags_with_number_questions_words_associated_with_each_one_should_be_obtained_and_chatty_tag(
    ) {
        match File::open("data_mock/mock1.jsonl") {
            Ok(file_content) => {
                let a_stats = get_hashmaps_stats(file_content);
                let stats_mock = get_mock_1_stats_site();
                assert_eq!(stats_mock, a_stats);
            }
            Err(_) => {
                assert!(
                    false,
                    "[ERROR] Trying to obtain a content of stats stats from a linea of jsonl"
                )
            }
        }
    }

    #[test]
    fn test_given_the_path_of_a_json_the_name_of_the_site_and_the_total_and_correct_stats_of_the_site_must_be_obtained(
    ) {
        match get_file_stats(&"data_mock/mock1.jsonl".to_string()) {
            Ok((name_content, stats_content)) => {
                let name_file_expected = "mock1.jsonl".to_string();
                let stats_mock = get_mock_1_stats_site();
                assert_eq!(stats_mock, stats_content);
                assert_eq!(name_file_expected, name_content);
            }
            Err(_) => {
                assert!(
                    false,
                    "[ERROR] Trying to obtain a content of stats stats from a linea of jsonl"
                )
            }
        }
    }

    #[test]
    fn test_given_a_vector_of_file_names_should_obtain_the_site_name_and_its_stats_correctly() {
        let files_names = vec![
            "data_mock/mock1.jsonl".to_string(),
            "data_mock/mock2.jsonl".to_string(),
        ];
        let vec_site_stats: Vec<(String, SiteStats)> = procces_a_vec_files_paths(files_names);
        let stats_mock_site_1 = get_mock_1_stats_site();
        let stats_mock_site_2 = get_mock_2_stats_site();

        let (file_name1, site_stats1) = &vec_site_stats[0];
        let (file_name2, site_stats2) = &vec_site_stats[1];
        assert_eq!(2, vec_site_stats.len());
        assert_eq!("mock1.jsonl".to_string(), file_name1.to_string());
        assert_eq!("mock2.jsonl".to_string(), file_name2.to_string());
        assert_eq!(stats_mock_site_1, *site_stats1);
        assert_eq!(stats_mock_site_2, *site_stats2);
    }
}
