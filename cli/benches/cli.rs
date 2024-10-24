use iai_callgrind::{binary_benchmark, binary_benchmark_group, main, BinaryBenchmarkConfig};

#[binary_benchmark]
#[bench::short(None, Some(5))]
#[bench::long(None, Some(15))]
#[bench::full(None, None)]
fn bench_cli_command(min: Option<usize>, max: Option<usize>) -> iai_callgrind::Command {
    let mut cmd = iai_callgrind::Command::new(env!("CARGO_BIN_EXE_bacrama-ranking-cli"));
    cmd.arg(env!("CARGO_MANIFEST_DIR").to_string() + "/../simulations");

    if let Some(min) = min {
        cmd.arg(format!("-m={:?}", min));
    }

    if let Some(max) = max {
        cmd.arg(format!("-M={:?}", max));
    }

    cmd.build()
}

binary_benchmark_group!(
    name = cli_command;
    benchmarks = bench_cli_command
);

main!(
    config = BinaryBenchmarkConfig::default().callgrind_args(["--branch-sim=yes"]);
    binary_benchmark_groups = cli_command
);
