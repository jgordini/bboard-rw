#![cfg_attr(not(feature = "ssr"), allow(dead_code))]

use leptos::prelude::ServerFnError;

#[cfg(feature = "ssr")]
fn contains_profanity(text: &str) -> bool {
    crate::profanity::contains_profanity(text)
}

#[cfg(not(feature = "ssr"))]
fn contains_profanity(_text: &str) -> bool {
    false
}

pub(crate) fn validate_idea_title_and_content(
    title: &str,
    content: &str,
) -> Result<(), ServerFnError> {
    if title.trim().is_empty() {
        return Err(ServerFnError::new("Idea title cannot be empty"));
    }
    if title.len() > 100 {
        return Err(ServerFnError::new("Idea title cannot exceed 100 characters"));
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
