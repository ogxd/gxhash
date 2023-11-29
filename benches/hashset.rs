use ahash::AHashSet;
use criterion::{criterion_group, criterion_main, Criterion};
use fnv::FnvHashSet;
use gxhash::*;
use std::collections::HashSet;
use std::hash::{BuildHasher, BuildHasherDefault};
use twox_hash::xxh3;

fn hashmap_insertion(c: &mut Criterion) {
    // Short keys
    benchmark_for_string(c, "gxhash");

    // Medium keys
    benchmark_for_string(c, "https://github.com/ogxd/gxhash");

    // Long keys
    benchmark_for_string(c, "Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.");

    // Very long keys
    benchmark_for_string(c, "Lorem ipsum dolor sit amet. Aut maxime voluptas ab quae explicabo et odio repellendus sed excepturi laboriosam? Ut molestiae obcaecati aut labore voluptates sed voluptatem voluptas non omnis harum et harum impedit ea eligendi autem id magni modi. Quo quam velit et error voluptas ut beatae repellendus et aspernatur incidunt hic veritatis accusamus sed autem modi cum error rerum. Sit perspiciatis consequuntur est perferendis veritatis et velit illum? At illo dolorum et voluptas nihil in voluptatum quas non quidem eveniet vel modi odit et sint nesciunt. Eos dicta consequuntur et sunt animi qui porro accusantium sed nisi voluptatum sed consectetur quibusdam ut ipsum mollitia. Et cupiditate iure aut omnis quia aut necessitatibus illum qui voluptas eius ut nihil laboriosam sit voluptatibus voluptas et galisum libero. Ut explicabo odit et adipisci accusantium ut officiis obcaecati. Eum pariatur sunt et autem neque ut eligendi autem. Qui voluptas Quis ut ratione officiis et placeat repudiandae sed tempora vitae At maxime quidem vel iure distinctio. Et doloremque esse ex eius voluptas id voluptatem recusandae qui illum quia ut consectetur quibusdam ea nisi accusamus!");
}

fn benchmark_for_string(c: &mut Criterion, string: &str) {
    let mut group = c.benchmark_group(format!("HashSet<&str[{}]>", string.len()));

    let mut set = HashSet::<String>::new();
    group.bench_function("Default Hasher", |b| {
        iterate(b, string, &mut set);
    });

    let mut set: HashSet<String, GxBuildHasher> = GxHashSet::<String>::default();
    group.bench_function("GxHash", |b| {
        iterate(b, string, &mut set);
    });

    let mut set = AHashSet::<String>::default();
    group.bench_function("AHash", |b| {
        iterate(b, string, &mut set);
    });

    let mut set = HashSet::<String, BuildHasherDefault<xxh3::Hash64>>::default();
    group.bench_function("XxHash", |b| {
        iterate(b, string, &mut set);
    });

    let mut set = FnvHashSet::<String>::default();
    group.bench_function("FNV-1a", |b| {
        iterate(b, string, &mut set);
    });

    group.finish();
}

#[inline(never)]
fn iterate<T>(b: &mut criterion::Bencher<'_>, string: &str, set: &mut HashSet<String, T>)
where
    T: BuildHasher,
{
    // If hashmap is empty, it may skip hashing the key and simply return false
    // So we add a single value to prevent this optimization
    set.insert("some text".to_string());
    b.iter(|| {
        // We intentionally check on a string that is not present, otherwise there will be an
        // additional equality check perform, diluting the hashing time and biasing the benchmark.
        set.contains(string)
    });
}

criterion_group!(benches, hashmap_insertion);
criterion_main!(benches);
