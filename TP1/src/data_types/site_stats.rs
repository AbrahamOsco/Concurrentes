use crate::data_types::tag_stats::TagStats;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// struct to store stats for a site.
#[derive(Debug, Serialize, Deserialize)]
pub struct SiteStats {
    /// total number of questions for this site
    pub questions: u32,
    /// total number of words for this site
    pub words: u32,
    /// hashmap whose key is the name of the tag and the value is a
    /// TagStats struct ( contains the number of questions and words on that site)
    pub tags: HashMap<String, TagStats>,
    /// vector with the names of the 10 tags with the highest word/question ratio for this site
    pub chatty_tags: Vec<String>,
}

/// The PartialEq trait is implemented to be able to compare SiteStats in tests.
impl PartialEq for SiteStats {
    fn eq(&self, other: &Self) -> bool {
        if self.questions != other.questions
            || self.words != other.words
            || self.tags.len() != other.tags.len()
            || self.chatty_tags.len() != other.chatty_tags.len()
        {
            return false;
        }

        for (key, value) in &self.tags {
            if other.tags.get(key) != Some(value) {
                return false;
            }
        }
        for (idx, tag) in self.chatty_tags.iter().enumerate() {
            if other.chatty_tags.get(idx) != Some(tag) {
                return false;
            }
        }
        true
    }
}
