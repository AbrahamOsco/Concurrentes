use serde::{Deserialize, Serialize};

/// Struct used to save the data of each line of a jsonl.
#[derive(Serialize, Deserialize, Debug)]
pub struct DataInputJsonl {
    /// A vector of strings to store all the entire value of the texts field of a line of jsonl.
    pub texts: Vec<String>,
    /// A vector of strings to store all the entire value of the tags field of a line of jsonl.
    pub tags: Vec<String>,
}
