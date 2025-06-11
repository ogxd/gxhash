use std::path::PathBuf;

use pyo3::prelude::pyclass;
use pyo3::prelude::pymethods;
use pyo3::prelude::Bound;
use pyo3::prelude::Py;
use pyo3::prelude::PyAny;
use pyo3::prelude::PyResult;
use pyo3::prelude::Python;
use pyo3::types::PyBytes;
use pyo3::types::PyModuleMethods;
use pyo3_async_runtimes::tokio::future_into_py;

fn gxhash<T>(hasher: fn(&[u8], i64) -> T, bytes: &[u8], seed: i64) -> PyResult<T> {
    Ok(hasher(bytes, seed))
}

fn gxhash_file<T>(hasher: fn(&[u8], i64) -> T, file_path: PathBuf, seed: i64) -> PyResult<T> {
    let file = std::fs::File::open(file_path)?;
    let mmap = unsafe { memmap2::Mmap::map(&file)? };
    Ok(hasher(&mmap, seed))
}

#[pyclass]
struct GxHash32 {
    seed: i64,
    hasher: fn(&[u8], i64) -> u32,
}

#[pyclass]
struct GxHash64 {
    seed: i64,
    hasher: fn(&[u8], i64) -> u64,
}

#[pyclass]
struct GxHash128 {
    seed: i64,
    hasher: fn(&[u8], i64) -> u128,
}

#[pymethods]
impl GxHash32 {
    #[new]
    fn new(seed: i64) -> Self {
        GxHash32 {
            seed,
            hasher: gxhash::gxhash32,
        }
    }

    fn hash(&self, bytes: &[u8]) -> PyResult<u32> {
        gxhash(self.hasher, bytes, self.seed)
    }

    fn hash_async<'a>(&self, py: Python<'a>, bytes: Py<PyBytes>) -> PyResult<Bound<'a, PyAny>> {
        let seed = self.seed;
        let hasher = self.hasher;

        future_into_py(py, async move { gxhash(hasher, Python::with_gil(|py| bytes.as_bytes(py)), seed) })
    }

    fn hash_file(&self, file_path: PathBuf) -> PyResult<u32> {
        gxhash_file(self.hasher, file_path, self.seed)
    }

    fn hash_file_async<'a>(&self, py: Python<'a>, file_path: PathBuf) -> PyResult<Bound<'a, PyAny>> {
        let seed = self.seed;
        let hasher = self.hasher;

        future_into_py(py, async move { gxhash_file(hasher, file_path, seed) })
    }
}

#[pymethods]
impl GxHash64 {
    #[new]
    fn new(seed: i64) -> Self {
        GxHash64 {
            seed,
            hasher: gxhash::gxhash64,
        }
    }

    fn hash(&self, bytes: &[u8]) -> PyResult<u64> {
        gxhash(self.hasher, bytes, self.seed)
    }

    fn hash_async<'a>(&self, py: Python<'a>, bytes: Py<PyBytes>) -> PyResult<Bound<'a, PyAny>> {
        let seed = self.seed;
        let hasher = self.hasher;

        future_into_py(py, async move { gxhash(hasher, Python::with_gil(|py| bytes.as_bytes(py)), seed) })
    }

    fn hash_file(&self, file_path: PathBuf) -> PyResult<u64> {
        gxhash_file(self.hasher, file_path, self.seed)
    }

    fn hash_file_async<'a>(&self, py: Python<'a>, file_path: PathBuf) -> PyResult<Bound<'a, PyAny>> {
        let seed = self.seed;
        let hasher = self.hasher;

        future_into_py(py, async move { gxhash_file(hasher, file_path, seed) })
    }
}

#[pymethods]
impl GxHash128 {
    #[new]
    fn new(seed: i64) -> Self {
        GxHash128 {
            seed,
            hasher: gxhash::gxhash128,
        }
    }

    fn hash(&self, bytes: &[u8]) -> PyResult<u128> {
        gxhash(self.hasher, bytes, self.seed)
    }

    fn hash_async<'a>(&self, py: Python<'a>, bytes: Py<PyBytes>) -> PyResult<Bound<'a, PyAny>> {
        let seed = self.seed;
        let hasher = self.hasher;

        future_into_py(py, async move { gxhash(hasher, Python::with_gil(|py| bytes.as_bytes(py)), seed) })
    }

    fn hash_file(&self, file_path: PathBuf) -> PyResult<u128> {
        gxhash_file(self.hasher, file_path, self.seed)
    }

    fn hash_file_async<'a>(&self, py: Python<'a>, file_path: PathBuf) -> PyResult<Bound<'a, PyAny>> {
        let seed = self.seed;
        let hasher = self.hasher;

        future_into_py(py, async move { gxhash_file(hasher, file_path, seed) })
    }
}

#[pyo3::prelude::pymodule(name = "gxhash")]
fn pygxhash(m: &Bound<'_, pyo3::prelude::PyModule>) -> PyResult<()> {
    m.add_class::<GxHash32>()?;
    m.add_class::<GxHash64>()?;
    m.add_class::<GxHash128>()?;
    Ok(())
}
