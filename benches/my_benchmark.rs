use criterion::{black_box, criterion_group, criterion_main, Criterion};
use e_core_detection::{get_pe_partition_async, get_pe_partition_sync};
use tokio::runtime::Runtime;

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("taskset-async", |b| {
        b.to_async(Runtime::new().unwrap())
            .iter(|| get_pe_partition_async())
    });
    c.bench_function("taskset-sync", |b| b.iter(|| get_pe_partition_sync()));
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
