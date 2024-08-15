use serde::{Deserialize, Serialize};

/// Struct used to store the number of words and questions associated with a tag.
#[derive(Debug, Serialize, Deserialize)]
pub struct TagStats {
    //// total number of questions for a tag
    pub questions: u32,
    //// total number of words for a tag
    pub words: u32,
}

/// The PartialEq trait is implemented to be able to more easily compare TagStats in the tests
impl PartialEq for TagStats {
    fn eq(&self, other: &Self) -> bool {
        if self.questions != other.questions || self.words != other.words {
            return false;
        }
        true
    }
}
