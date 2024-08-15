use serde::{Deserialize, Serialize};

/// Struct used to store the total stats obtained using all sites
#[derive(Serialize, Deserialize, Debug)]
pub struct TotalStats {
    /// vector with the name of the 10 sites with the highest ratio:
    /// words/question among all sites, ordered in descending order.
    pub chatty_sites: Vec<String>,
    /// vector with the name of the 10 tags with the highest ratio:
    /// words/questions, of the accumulated tags of all the sites,
    /// ordered in descending order.
    pub chatty_tags: Vec<String>,
}
