#![feature(const_trait_impl)]
#![feature(core_intrinsics)]
#![feature(const_default_impls)]
#![feature(const_mut_refs)]

mod autocomplete;
mod autocomplete_hashmap;
mod parsing;
mod suggestions;

pub use crate::{
	autocomplete::{AutoComplete, Movement},
	autocomplete_hashmap::compile_hashmap,
	parsing::{process_func_str, BackingFunction},
	suggestions::{
		generate_hint, get_last_term, split_function, split_function_chars, Hint, HINT_EMPTY,
		SUPPORTED_FUNCTIONS,
	},
};
