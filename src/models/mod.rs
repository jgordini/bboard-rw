mod idea;
pub use idea::{Idea, IdeaForm};
mod vote;
pub use vote::{Vote, VoteForm};

// Old models disabled - tables don't exist in new schema
// mod user;
// pub use user::{User, UserPreview};
// mod pagination;
// pub use pagination::Pagination;
// mod article;
// pub use article::Article;
// mod comment;
// pub use comment::Comment;

#[cfg(feature = "ssr")]
pub const DATE_FORMAT: &str = "%d/%m/%Y %H:%M";
