mod idea;
pub use idea::{Idea, IdeaForm};
mod vote;
pub use vote::{Vote, VoteForm};
mod comment;
pub use comment::Comment;

#[cfg(feature = "ssr")]
pub const DATE_FORMAT: &str = "%d/%m/%Y %H:%M";
