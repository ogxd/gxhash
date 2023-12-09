use ahash::AHashSet;
use criterion::{criterion_group, criterion_main, Criterion};
use fnv::FnvHashSet;
use gxhash::*;
use twox_hash::xxh3;
use std::hash::Hash;
use std::collections::HashSet;
use std::hash::{BuildHasherDefault, BuildHasher};

fn hashset_contains(c: &mut Criterion) {
    benchmark(c, "u32", 42u32);
    benchmark(c, "u64", 42u64);
    benchmark(c, "u128", 42u128);
    benchmark(c, "small string", "gxhash".to_owned());
    benchmark(c, "medium string","https://github.com/ogxd/gxhash".to_owned());
    benchmark(c, "large string","Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat. Duis aute irure dolor in reprehenderit in voluptate velit esse cillum dolore eu fugiat nulla pariatur. Excepteur sint occaecat cupidatat non proident, sunt in culpa qui officia deserunt mollit anim id est laborum.".to_owned());
    benchmark(c, "huge string", "Lorem ipsum dolor sit amet. Aut maxime voluptas ab quae explicabo et odio repellendus sed excepturi laboriosam? Ut molestiae obcaecati aut labore voluptates sed voluptatem voluptas non omnis harum et harum impedit ea eligendi autem id magni modi. Quo quam velit et error voluptas ut beatae repellendus et aspernatur incidunt hic veritatis accusamus sed autem modi cum error rerum. Sit perspiciatis consequuntur est perferendis veritatis et velit illum? At illo dolorum et voluptas nihil in voluptatum quas non quidem eveniet vel modi odit et sint nesciunt. Eos dicta consequuntur et sunt animi qui porro accusantium sed nisi voluptatum sed consectetur quibusdam ut ipsum mollitia. Et cupiditate iure aut omnis quia aut necessitatibus illum qui voluptas eius ut nihil laboriosam sit voluptatibus voluptas et galisum libero. Ut explicabo odit et adipisci accusantium ut officiis obcaecati. Eum pariatur sunt et autem neque ut eligendi autem. Qui voluptas Quis ut ratione officiis et placeat repudiandae sed tempora vitae At maxime quidem vel iure distinctio. Et doloremque esse ex eius voluptas id voluptatem recusandae qui illum quia ut consectetur quibusdam ea nisi accusamus!".to_owned());
}

fn benchmark<T>(c: &mut Criterion, name: &str, value: T)
    where T: Eq+PartialEq+Hash+Default
{
    let mut group = c.benchmark_group(format!("HashSet<{}>/{}", std::any::type_name::<T>(), name));

    let mut set = HashSet::<T>::new();
    group.bench_function("Default Hasher", |b| {
        iterate(b, &value, &mut set);
    });

    let mut set: HashSet::<T, GxBuildHasher> = GxHashSet::<T>::default();
    group.bench_function("GxHash", |b| {
        iterate(b, &value, &mut set);
    });

    let mut set = AHashSet::<T>::default();
    group.bench_function("AHash", |b| {
        iterate(b, &value, &mut set);
    });

    let mut set = HashSet::<T, BuildHasherDefault<xxh3::Hash64>>::default();
    group.bench_function("XxHash", |b| {
        iterate(b, &value, &mut set);
    });

    let mut set = FnvHashSet::<T>::default();
    group.bench_function("FNV-1a", |b| {
        iterate(b, &value, &mut set);
    });

    group.finish();
}

#[inline(never)]
fn iterate<T, B>(b: &mut criterion::Bencher<'_>, value: &T, set: &mut HashSet<T, B>)
    where B: BuildHasher, T: Eq+PartialEq+Hash+Default
{
    // If hashmap is empty, it may skip hashing the key and simply return false
    // So we add a single value to prevent this optimization
    set.insert(T::default());
    b.iter(|| {
        // We intentionally check on a value that is not present, otherwise there will be an
        // additional equality check perform, diluting the hashing time and biasing the benchmark.
        set.contains(criterion::black_box(value))
    });
}

criterion_group!(benches, hashset_contains);
criterion_main!(benches);