//! Interactive user interface components

pub mod prompts;

pub use prompts::{
    confirm_match, prompt_custom_search, prompt_manual_metadata, prompt_match_selection,
    UserChoice,
};
