use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;
use std::os::fd::FromRawFd;

fn get_file_descriptor(py: Python, file: PyObject) -> Result<i32, PyErr> {
    file.call_method0(py, pyo3::intern!(py, "fileno"))?.extract(py)
}

fn gxhash<T>(hasher: fn(&[u8], i64) -> T, bytes: &[u8], seed: i64) -> PyResult<T> {
    Ok(hasher(bytes, seed))
}

fn gxhash_nogil<T: Send>(py: Python, hasher: fn(&[u8], i64) -> T, bytes: &[u8], seed: i64) -> PyResult<T> {
    py.allow_threads(|| gxhash(hasher, bytes, seed))
}

fn gxhash_file<T>(hasher: fn(&[u8], i64) -> T, file_descriptor: i32, seed: i64) -> PyResult<T> {
    let file = unsafe { std::fs::File::from_raw_fd(libc::dup(file_descriptor)) };
    let mmap = unsafe { memmap2::Mmap::map(&file).unwrap() };
    drop(file);
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

    fn hash_nogil(&self, py: Python, bytes: &[u8]) -> PyResult<u32> {
        gxhash_nogil(py, self.hasher, bytes, self.seed)
    }

    fn hash_file(&self, py: Python, file: PyObject) -> PyResult<u32> {
        gxhash_file(self.hasher, get_file_descriptor(py, file)?, self.seed)
    }

    fn hash_file_async<'a>(&self, py: Python<'a>, file: PyObject) -> PyResult<Bound<'a, PyAny>> {
        let seed = self.seed;
        let hasher = self.hasher;
        let file_descriptor = get_file_descriptor(py, file)?;

        future_into_py(py, async move { gxhash_file(hasher, file_descriptor, seed) })
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

    fn hash_nogil(&self, py: Python, bytes: &[u8]) -> PyResult<u64> {
        gxhash_nogil(py, self.hasher, bytes, self.seed)
    }

    fn hash_file(&self, py: Python, file: PyObject) -> PyResult<u64> {
        gxhash_file(self.hasher, get_file_descriptor(py, file)?, self.seed)
    }

    fn hash_file_async<'a>(&self, py: Python<'a>, file: PyObject) -> PyResult<Bound<'a, PyAny>> {
        let seed = self.seed;
        let hasher = self.hasher;
        let file_descriptor = get_file_descriptor(py, file)?;

        future_into_py(py, async move { gxhash_file(hasher, file_descriptor, seed) })
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

    fn hash_nogil(&self, py: Python, bytes: &[u8]) -> PyResult<u128> {
        gxhash_nogil(py, self.hasher, bytes, self.seed)
    }

    fn hash_file(&self, py: Python, file: PyObject) -> PyResult<u128> {
        gxhash_file(self.hasher, get_file_descriptor(py, file)?, self.seed)
    }

    fn hash_file_async<'a>(&self, py: Python<'a>, file: PyObject) -> PyResult<Bound<'a, PyAny>> {
        let seed = self.seed;
        let hasher = self.hasher;
        let file_descriptor = get_file_descriptor(py, file)?;

        future_into_py(py, async move { gxhash_file(hasher, file_descriptor, seed) })
    }
}

#[pymodule(name = "gxhash")]
fn pygxhash(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<GxHash32>()?;
    m.add_class::<GxHash64>()?;
    m.add_class::<GxHash128>()?;
    Ok(())
}
