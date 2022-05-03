#![feature(custom_test_frameworks)]
#![test_runner(criterion::runner)]

#[allow(unused_imports)]
use parsing::suggestions::split_function_chars;

use std::{fs::File, os::raw::c_int, path::Path};

use criterion::profiler::Profiler;
use criterion::Criterion;
use criterion_macro::criterion;
use pprof::ProfilerGuard;

pub struct FlamegraphProfiler<'a> {
	frequency: c_int,
	active_profiler: Option<ProfilerGuard<'a>>,
}

impl<'a> FlamegraphProfiler<'a> {
	#[allow(dead_code)]
	pub fn new(frequency: c_int) -> Self {
		FlamegraphProfiler {
			frequency,
			active_profiler: None,
		}
	}
}

impl<'a> Profiler for FlamegraphProfiler<'a> {
	fn start_profiling(&mut self, _benchmark_id: &str, _benchmark_dir: &Path) {
		self.active_profiler = Some(ProfilerGuard::new(self.frequency).unwrap());
	}

	fn stop_profiling(&mut self, _benchmark_id: &str, benchmark_dir: &Path) {
		std::fs::create_dir_all(benchmark_dir).unwrap();
		let flamegraph_path = benchmark_dir.join("flamegraph.svg");
		let flamegraph_file = File::create(&flamegraph_path)
			.expect("File system error while creating flamegraph.svg");
		if let Some(profiler) = self.active_profiler.take() {
			profiler
				.report()
				.build()
				.unwrap()
				.flamegraph(flamegraph_file)
				.expect("Error writing flamegraph");
		}
	}
}

fn custom_criterion() -> Criterion {
	Criterion::default().with_profiler(FlamegraphProfiler::new(100))
}

#[criterion(custom_criterion())]
fn split_function_bench(c: &mut Criterion) {
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

	c.bench_function("split_function", |b| {
		b.iter(|| {
			data_chars.iter().for_each(|a| {
				split_function_chars(a);
			})
		})
	});
}
