use leptos::prelude::ServerFnError;

fn contains_profanity(text: &str) -> bool {
    crate::profanity::contains_profanity(text)
}

pub(crate) fn validate_idea_title_and_content(
    title: &str,
    content: &str,
) -> Result<(), ServerFnError> {
    if title.trim().is_empty() {
        return Err(ServerFnError::new("Idea title cannot be empty"));
    }
    if title.len() > 100 {
        return Err(ServerFnError::new(
            "Idea title cannot exceed 100 characters",
        ));
    }
    if content.trim().is_empty() {
        return Err(ServerFnError::new("Idea description cannot be empty"));
    }
    if content.len() > 500 {
        return Err(ServerFnError::new(
            "Idea description cannot exceed 500 characters",
        ));
    }
    if contains_profanity(title) || contains_profanity(content) {
        return Err(ServerFnError::new(
            "Your submission contains inappropriate language. Please revise and try again.",
        ));
    }
    Ok(())
}

pub(crate) fn validate_idea_tags(tags: &str) -> Result<(), ServerFnError> {
    if tags.len() > 200 {
        return Err(ServerFnError::new("Tags cannot exceed 200 characters"));
    }
    Ok(())
}

pub(crate) fn validate_comment_content(content: &str) -> Result<(), ServerFnError> {
    if content.trim().is_empty() {
        return Err(ServerFnError::new("Comment cannot be empty"));
    }
    if content.len() > 500 {
        return Err(ServerFnError::new("Comment cannot exceed 500 characters"));
    }
    if contains_profanity(content) {
        return Err(ServerFnError::new(
            "Your comment contains inappropriate language. Please revise and try again.",
        ));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn err_message(err: ServerFnError) -> String {
        err.to_string()
    }

    #[test]
    fn validate_idea_title_and_content_rejects_empty_title() {
        let err = validate_idea_title_and_content("   ", "Valid content").unwrap_err();
        assert!(err_message(err).contains("Idea title cannot be empty"));
    }

    #[test]
    fn validate_idea_title_and_content_rejects_long_title() {
        let title = "a".repeat(101);
        let err = validate_idea_title_and_content(&title, "Valid content").unwrap_err();
        assert!(err_message(err).contains("Idea title cannot exceed 100 characters"));
    }

    #[test]
    fn validate_idea_title_and_content_rejects_empty_description() {
        let err = validate_idea_title_and_content("Valid title", "   ").unwrap_err();
        assert!(err_message(err).contains("Idea description cannot be empty"));
    }

    #[test]
    fn validate_idea_title_and_content_rejects_long_description() {
        let content = "a".repeat(501);
        let err = validate_idea_title_and_content("Valid title", &content).unwrap_err();
        assert!(err_message(err).contains("Idea description cannot exceed 500 characters"));
    }

    #[test]
    fn validate_idea_title_and_content_rejects_profanity() {
        let err = validate_idea_title_and_content("shit idea", "Valid content").unwrap_err();
        assert!(err_message(err).contains("contains inappropriate language"));
    }

    #[test]
    fn validate_idea_tags_enforces_limit() {
        assert!(validate_idea_tags(&"a".repeat(200)).is_ok());

        let err = validate_idea_tags(&"a".repeat(201)).unwrap_err();
        assert!(err_message(err).contains("Tags cannot exceed 200 characters"));
    }

    #[test]
    fn validate_comment_content_enforces_rules() {
        let empty_err = validate_comment_content("   ").unwrap_err();
        assert!(err_message(empty_err).contains("Comment cannot be empty"));

        let long_err = validate_comment_content(&"a".repeat(501)).unwrap_err();
        assert!(err_message(long_err).contains("Comment cannot exceed 500 characters"));

        let profanity_err = validate_comment_content("fuck").unwrap_err();
        assert!(err_message(profanity_err).contains("contains inappropriate language"));
    }
}
