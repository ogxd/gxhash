use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;
use pyo3_async_runtimes::tokio::future_into_py;
use tokio::task::spawn_blocking;

#[pyfunction]
fn gxhash32(input_bytes: &[u8], seed: i64) -> PyResult<u32> {
    Ok(gxhash::gxhash32(input_bytes, seed))
}

#[pyfunction]
fn gxhash32_nogil(py: Python, input_bytes: &[u8], seed: i64) -> PyResult<u32> {
    py.allow_threads(|| Ok(gxhash::gxhash32(input_bytes, seed)))
}

#[pyfunction]
fn gxhash32_async<'p>(py: Python<'p>, input_bytes: &'p [u8], seed: i64) -> PyResult<Bound<'p, PyAny>> {
    let input_bytes_clone = input_bytes.to_vec();

    future_into_py(py, async move {
        let result = Python::with_gil(|py| py.allow_threads(|| spawn_blocking(move || gxhash::gxhash32(&input_bytes_clone, seed)))).await;

        match result {
            Ok(result) => Ok(result),
            Err(e) => Err(PyRuntimeError::new_err(format!("Task failed: {:?}", e))),
        }
    })
}

#[pyfunction]
fn gxhash64(input_bytes: &[u8], seed: i64) -> PyResult<u64> {
    Ok(gxhash::gxhash64(input_bytes, seed))
}

#[pyfunction]
fn gxhash64_nogil(py: Python, input_bytes: &[u8], seed: i64) -> PyResult<u64> {
    py.allow_threads(|| Ok(gxhash::gxhash64(input_bytes, seed)))
}

#[pyfunction]
fn gxhash64_async<'p>(py: Python<'p>, input_bytes: &'p [u8], seed: i64) -> PyResult<Bound<'p, PyAny>> {
    let input_bytes_clone = input_bytes.to_vec();

    future_into_py(py, async move {
        let result = Python::with_gil(|py| py.allow_threads(|| spawn_blocking(move || gxhash::gxhash64(&input_bytes_clone, seed)))).await;

        match result {
            Ok(result) => Ok(result),
            Err(e) => Err(PyRuntimeError::new_err(format!("Task failed: {:?}", e))),
        }
    })
}

#[pyfunction]
fn gxhash128(input_bytes: &[u8], seed: i64) -> PyResult<u128> {
    Ok(gxhash::gxhash128(input_bytes, seed))
}

#[pyfunction]
fn gxhash128_nogil(py: Python, input_bytes: &[u8], seed: i64) -> PyResult<u128> {
    py.allow_threads(|| Ok(gxhash::gxhash128(input_bytes, seed)))
}

#[pyfunction]
fn gxhash128_async<'p>(py: Python<'p>, input_bytes: &'p [u8], seed: i64) -> PyResult<Bound<'p, PyAny>> {
    let input_bytes_clone = input_bytes.to_vec();

    future_into_py(py, async move {
        let result = Python::with_gil(|py| py.allow_threads(|| spawn_blocking(move || gxhash::gxhash128(&input_bytes_clone, seed)))).await;

        match result {
            Ok(result) => Ok(result),
            Err(e) => Err(PyRuntimeError::new_err(format!("Task failed: {:?}", e))),
        }
    })
}

#[pymodule(name = "gxhash")]
fn pygxhash(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(gxhash32, m)?)?;
    m.add_function(wrap_pyfunction!(gxhash32_nogil, m)?)?;
    m.add_function(wrap_pyfunction!(gxhash32_async, m)?)?;
    m.add_function(wrap_pyfunction!(gxhash64, m)?)?;
    m.add_function(wrap_pyfunction!(gxhash64_nogil, m)?)?;
    m.add_function(wrap_pyfunction!(gxhash64_async, m)?)?;
    m.add_function(wrap_pyfunction!(gxhash128, m)?)?;
    m.add_function(wrap_pyfunction!(gxhash128_nogil, m)?)?;
    m.add_function(wrap_pyfunction!(gxhash128_async, m)?)?;
    Ok(())
}
