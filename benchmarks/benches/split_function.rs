use criterion::{criterion_group, criterion_main, Criterion};
use parsing::split_function_chars;
use std::time::Duration;

fn custom_criterion() -> Criterion {
	Criterion::default()
		.warm_up_time(Duration::from_millis(250))
		.sample_size(1000)
}

fn mutli_split_function(c: &mut Criterion) {
	let data_chars = vec![
		"sin(x)cos(x)",
		"x^2",
		"2x",
		"log10(x)",
		"E^x",
		"xxxxx",
		"xsin(x)",
		"(2x+1)(3x+1)",
		"x**2",
		"pipipipipipix",
		"pi(2x+1)",
		"(2x+1)pi",
	]
	.iter()
	.map(|a| a.chars().collect::<Vec<char>>())
	.collect::<Vec<Vec<char>>>();

	let mut group = c.benchmark_group("split_function");
	for entry in data_chars {
		group.bench_function(entry.iter().collect::<String>(), |b| {
			b.iter(|| {
				split_function_chars(&entry, parsing::SplitType::Multiplication);
			})
		});
	}
	group.finish();
}

criterion_group! {
	name = benches;
	config = custom_criterion();
	targets = mutli_split_function
}
criterion_main!(benches);
