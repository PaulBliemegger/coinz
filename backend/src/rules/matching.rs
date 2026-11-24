use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Operator {
    Contains,
    ContainsWord,
    Equals,
}
impl Operator {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Contains => "contains",
            Self::ContainsWord => "contains_word",
            Self::Equals => "equals",
        }
    }
    pub fn matches(self, pattern: &str, description: &str) -> bool {
        if pattern.is_empty() {
            return false;
        }
        let pattern = pattern.to_lowercase();
        let description = description.to_lowercase();
        match self {
            Self::Contains => description.contains(&pattern),
            Self::Equals => description == pattern,
            Self::ContainsWord => description
                .split(|c: char| !c.is_alphanumeric())
                .any(|word| word == pattern),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::Operator::*;
    #[test]
    fn literal_matching_handles_case_punctuation_and_word_boundaries() {
        assert!(ContainsWord.matches("coffee", "COFFEE-shop"));
        assert!(!ContainsWord.matches("coffee", "coffeeshop"));
        assert!(ContainsWord.matches("café", "CAFÉ, Vienna"));
        assert!(Contains.matches("%_", "literal %_ purchase"));
        assert!(!Contains.matches("%_", "ordinary purchase"));
        assert!(Equals.matches("Corner coffee", "CORNER COFFEE"));
        assert!(!Equals.matches("coffee", "Corner coffee"));
    }
}
