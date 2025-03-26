use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;
use std::os::fd::FromRawFd;

fn get_file_descriptor(py: Python, file: PyObject) -> Result<i32, PyErr> {
    file.call_method0(py, pyo3::intern!(py, "fileno"))?.extract(py)
}

fn gxhash<T>(hasher: fn(&[u8], i64) -> T, file_descriptor: i32, seed: i64) -> PyResult<T> {
    let file = unsafe { std::fs::File::from_raw_fd(libc::dup(file_descriptor)) };
    let mmap = unsafe { memmap2::Mmap::map(&file).unwrap() };
    drop(file);
    Ok(hasher(&mmap, seed))
}

#[pyfunction]
fn gxhash32(py: Python, file: PyObject, seed: i64) -> PyResult<u32> {
    let file_descriptor = get_file_descriptor(py, file)?;
    gxhash(gxhash::gxhash32, file_descriptor, seed)
}

#[pyfunction]
fn gxhash32_async(py: Python, file: PyObject, seed: i64) -> PyResult<Bound<PyAny>> {
    let file_descriptor = get_file_descriptor(py, file)?;
    future_into_py(py, async move { gxhash(gxhash::gxhash32, file_descriptor, seed) })
}

#[pyfunction]
fn gxhash64(py: Python, file: PyObject, seed: i64) -> PyResult<u64> {
    let file_descriptor = get_file_descriptor(py, file)?;
    gxhash(gxhash::gxhash64, file_descriptor, seed)
}

#[pyfunction]
fn gxhash64_async(py: Python, file: PyObject, seed: i64) -> PyResult<Bound<PyAny>> {
    let file_descriptor = get_file_descriptor(py, file)?;
    future_into_py(py, async move { gxhash(gxhash::gxhash64, file_descriptor, seed) })
}

#[pyfunction]
fn gxhash128(py: Python, file: PyObject, seed: i64) -> PyResult<u128> {
    let file_descriptor = get_file_descriptor(py, file)?;
    gxhash(gxhash::gxhash128, file_descriptor, seed)
}

#[pyfunction]
fn gxhash128_async(py: Python, file: PyObject, seed: i64) -> PyResult<Bound<PyAny>> {
    let file_descriptor = get_file_descriptor(py, file)?;
    future_into_py(py, async move { gxhash(gxhash::gxhash128, file_descriptor, seed) })
}

#[pymodule(name = "gxhash")]
fn pygxhash(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(gxhash32, m)?)?;
    m.add_function(wrap_pyfunction!(gxhash32_async, m)?)?;
    m.add_function(wrap_pyfunction!(gxhash64, m)?)?;
    m.add_function(wrap_pyfunction!(gxhash64_async, m)?)?;
    m.add_function(wrap_pyfunction!(gxhash128, m)?)?;
    m.add_function(wrap_pyfunction!(gxhash128_async, m)?)?;
    Ok(())
}
