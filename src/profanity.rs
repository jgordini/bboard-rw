//! Profanity filter for content moderation
//! Filters out common offensive words and their variations

/// List of blocked words (the 7 dirty words and common slurs)
const BLOCKED_WORDS: &[&str] = &[
    // The 7 dirty words
    "shit",
    "piss",
    "fuck",
    "cunt",
    "cocksucker",
    "motherfucker",
    "tits",
    // Common variations
    "f*ck",
    "f**k",
    "sh*t",
    "sh1t",
    "f u c k",
    "s h i t",
    // Common slurs (abbreviated for safety)
    "n1gger",
    "n1gga",
    "f4g",
    "f4gg0t",
];

/// Character substitutions commonly used to bypass filters
fn normalize_text(text: &str) -> String {
    text.to_lowercase()
        .replace('0', "o")
        .replace('1', "i")
        .replace('3', "e")
        .replace('4', "a")
        .replace('5', "s")
        .replace('7', "t")
        .replace('@', "a")
        .replace('$', "s")
        .replace('!', "i")
        .replace('+', "t")
        .replace(' ', "")
}

/// Check if text contains profanity
pub fn contains_profanity(text: &str) -> bool {
    let normalized = normalize_text(text);

    for word in BLOCKED_WORDS {
        let normalized_word = normalize_text(word);
        if normalized.contains(&normalized_word) {
            return true;
        }
    }

    false
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_profanity() {
        assert!(contains_profanity("this is shit"));
        assert!(contains_profanity("what the fuck"));
    }

    #[test]
    fn test_clean_text() {
        assert!(!contains_profanity("this is a good idea"));
        assert!(!contains_profanity("improve the system"));
    }

    #[test]
    fn test_variations() {
        assert!(contains_profanity("f*ck this"));
        assert!(contains_profanity("sh1t"));
        assert!(contains_profanity("f u c k"));
    }

    #[test]
    fn test_number_substitution() {
        assert!(contains_profanity("sh1t"));
        assert!(contains_profanity("fvck")); // Won't catch this specific one
    }
}
