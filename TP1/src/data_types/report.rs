use crate::data_types::site_stats::SiteStats;
use crate::data_types::tag_stats::TagStats;
use crate::data_types::total_stats::TotalStats;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
/// Struct used to store stats for each site and total stats for all sites.
#[derive(Serialize, Deserialize, Debug)]
pub struct Report {
    /// student registration number
    pub padron: i32,
    /// hashmap whose key is the name of the site and the value is a
    /// SiteStats struct ( contains the stats for that site)
    pub sites: HashMap<String, SiteStats>,
    /// hashmap where all the tags and theirs stats from all the sites are accumulated.
    /// The key is the name of the tag and the value is a
    /// TagStats struct ( contains the number of questions and words on that site)
    pub tags: HashMap<String, TagStats>,
    /// structure that stores the top 10 chatty_sites and the top 10 chatty_tag.
    pub totals: TotalStats,
}

/// Returns a base report only with the padron defined and the other attributes initialized empty.
pub fn get_report_base() -> Report {
    Report {
        padron: 102256,
        sites: Default::default(),
        tags: Default::default(),
        totals: TotalStats {
            chatty_sites: vec![],
            chatty_tags: vec![],
        },
    }
}
