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

    fn assert_error_contains(result: Result<(), ServerFnError>, expected: &str) {
        let err = result.expect_err("expected validation to fail");
        let error_message = err.to_string();
        assert!(
            error_message.contains(expected),
            "expected error containing '{expected}', got '{error_message}'",
        );
    }

    #[test]
    fn idea_title_and_content_accept_valid_boundaries() {
        let title = "a".repeat(100);
        let content = "b".repeat(500);
        assert!(validate_idea_title_and_content(&title, &content).is_ok());
    }

    #[test]
    fn idea_title_and_content_reject_empty_fields() {
        assert_error_contains(
            validate_idea_title_and_content("   ", "content"),
            "Idea title cannot be empty",
        );
        assert_error_contains(
            validate_idea_title_and_content("title", "   "),
            "Idea description cannot be empty",
        );
    }

    #[test]
    fn idea_title_and_content_reject_over_length() {
        let long_title = "a".repeat(101);
        let long_content = "b".repeat(501);
        assert_error_contains(
            validate_idea_title_and_content(&long_title, "content"),
            "Idea title cannot exceed 100 characters",
        );
        assert_error_contains(
            validate_idea_title_and_content("title", &long_content),
            "Idea description cannot exceed 500 characters",
        );
    }

    #[test]
    fn idea_title_and_content_reject_profanity() {
        assert_error_contains(
            validate_idea_title_and_content("clean", "this is shit"),
            "contains inappropriate language",
        );
    }

    #[test]
    fn idea_tags_reject_over_length() {
        let long_tags = "t".repeat(201);
        assert_error_contains(
            validate_idea_tags(&long_tags),
            "Tags cannot exceed 200 characters",
        );
    }

    #[test]
    fn comment_content_accepts_valid_boundary() {
        let content = "c".repeat(500);
        assert!(validate_comment_content(&content).is_ok());
    }

    #[test]
    fn comment_content_rejects_empty_overlength_and_profanity() {
        assert_error_contains(validate_comment_content("   "), "Comment cannot be empty");
        assert_error_contains(
            validate_comment_content(&"c".repeat(501)),
            "Comment cannot exceed 500 characters",
        );
        assert_error_contains(
            validate_comment_content("what the fuck"),
            "contains inappropriate language",
        );
    }

    /// Mirrors the validation order in `update_idea_content_mod`: tags are
    /// validated before title/content so a tag error is surfaced first when
    /// both inputs are invalid.
    fn validate_idea_update_order(
        title: &str,
        content: &str,
        tags: &str,
    ) -> Result<(), ServerFnError> {
        validate_idea_tags(tags)?;
        validate_idea_title_and_content(title, content)?;
        Ok(())
    }

    #[test]
    fn idea_update_returns_tag_error_before_title_content_error() {
        let long_tags = "t".repeat(201);
        // Both tags and title are invalid -- tag error should win.
        assert_error_contains(
            validate_idea_update_order("", "", &long_tags),
            "Tags cannot exceed 200 characters",
        );
    }
}
