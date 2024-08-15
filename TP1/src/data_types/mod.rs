//! Module that contains all the data tips (structs) used in practical work.

//! - `data_input_jsonl`: Module where the DataInputJsonl struct is created to store the jsonl data.
//! - `report`: Module where the Report struct is created to generate the final report by stdout.
//! - `site_stats`: Module where the SiteStats struct is created to store statistics related to the site data.
//! - `tag_stats`: Module where the TagStats struct is created to store statistics related to tags.
//! - `total_stats`: Module where the TotalStats struct is created to store the top 10 among the sites
//! and the top 10 of the tags according to a quotient.
pub mod data_input_jsonl;
pub mod report;
pub mod site_stats;
pub mod tag_stats;
pub mod total_stats;
