use crate::data_types::report::Report;
use crate::data_types::site_stats::SiteStats;
use crate::data_types::tag_stats::TagStats;
use crate::logic::processing_file::{get_top10_tags_from_hashmap, sort_vec_and_set_top10};
use std::collections::HashMap;

/// receives by reference a SiteStats structure and a mutable reference
/// of the accumulated tags hashmap It is responsible for accumulating the number
/// of questions and words that a tag has, taking into account all the sites.
fn accumulate_tag_stats(site_stats: &SiteStats, all_tags: &mut HashMap<String, TagStats>) {
    for (tag_name, stats) in &site_stats.tags {
        let value_tag = all_tags.entry(tag_name.clone()).or_insert(TagStats {
            questions: 0,
            words: 0,
        });
        value_tag.questions += stats.questions;
        value_tag.words += stats.words;
    }
}

/// receives the report structure (by movement) and is responsible for printing it through stdout.
fn show_final_report_in_json(report_final: Report) {
    match serde_json::to_string(&report_final) {
        Ok(json_content) => {
            println!("{}", json_content);
        }
        Err(e) => {
            eprintln!("[ERROR]: in function serder_json::to_string {} ", e);
        }
    }
}

/// receives the accumulated tag vector of all sites and the report structure, both by movement
/// delegates the processing of chatty sites and chatty tags
/// and with the result obtained, the struct report is finally completed
/// delegates the printing of the json to stdout.
fn process_the_latest_stats_and_show(
    mut chatty_sites_global_aux: Vec<(String, f32)>,
    mut report_final: Report,
) {
    sort_vec_and_set_top10(&mut chatty_sites_global_aux);
    report_final.totals.chatty_sites = chatty_sites_global_aux
        .iter()
        .map(|(a, _)| a.clone())
        .collect();
    report_final.totals.chatty_tags = get_top10_tags_from_hashmap(&report_final.tags)
        .iter()
        .map(|(a, _)| a.clone())
        .collect();
    show_final_report_in_json(report_final);
}

/// Receive a struct report (by movement) with the site hashmap already loaded
/// delegates the process of accumulating the tag stats,
/// the top 10 chatty_sites and chatty_tag among all sites, and
/// delegates the printing of the json to stdout.
pub fn generate_final_report(mut report_final: Report) {
    let mut chatty_sites_global_aux = vec![];
    for (site_name, stats_a_site) in &report_final.sites {
        chatty_sites_global_aux.push((
            site_name.clone(),
            stats_a_site.words as f32 / stats_a_site.questions as f32,
        ));
        accumulate_tag_stats(stats_a_site, &mut report_final.tags);
    }
    process_the_latest_stats_and_show(chatty_sites_global_aux, report_final);
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

    fn get_mock_all_tags_accumulate() -> HashMap<String, TagStats> {
        let mut mock_all_tags_accumulate: HashMap<String, TagStats> = HashMap::new();
        mock_all_tags_accumulate.insert(
            "chiste".to_string(),
            TagStats {
                questions: 4,
                words: 33,
            },
        );
        mock_all_tags_accumulate.insert(
            "pokemon".to_string(),
            TagStats {
                questions: 2,
                words: 26,
            },
        );
        mock_all_tags_accumulate.insert(
            "other3".to_string(),
            TagStats {
                questions: 2,
                words: 18,
            },
        );
        mock_all_tags_accumulate.insert(
            "random".to_string(),
            TagStats {
                questions: 1,
                words: 8,
            },
        );
        mock_all_tags_accumulate
    }
    #[test]
    fn test_give_two_site_stats_all_tags_from_both_site_stats_must_be_correctly_accumulated() {
        let mut all_tags_accumulate: HashMap<String, TagStats> = HashMap::new();
        let mock_all_tags_accumulate: HashMap<String, TagStats> = get_mock_all_tags_accumulate();
        accumulate_tag_stats(&get_mock_1_stats_site(), &mut all_tags_accumulate);
        accumulate_tag_stats(&get_mock_2_stats_site(), &mut all_tags_accumulate);
        for (tag_name, tag_stats) in all_tags_accumulate {
            if mock_all_tags_accumulate.get(&tag_name) != Some(&tag_stats) {
                assert!(false, "[ERROR] the tag_stats are not equal")
            }
        }
    }
}
