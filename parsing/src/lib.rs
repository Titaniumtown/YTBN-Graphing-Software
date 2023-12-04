#![feature(const_trait_impl)]
#![feature(core_intrinsics)]
#![feature(const_mut_refs)]
#![feature(const_for)]
mod autocomplete;
mod autocomplete_hashmap;
mod parsing;
mod splitting;
mod suggestions;

pub use crate::{
	autocomplete::{AutoComplete, Movement},
	autocomplete_hashmap::compile_hashmap,
	parsing::{process_func_str, BackingFunction, FlatExWrapper},
	splitting::{split_function, split_function_chars, SplitType},
	suggestions::{generate_hint, get_last_term, Hint, HINT_EMPTY, SUPPORTED_FUNCTIONS},
};
