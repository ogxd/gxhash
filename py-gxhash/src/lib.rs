use pyo3::prelude::*;

#[pyfunction]
fn gxhash32(input_bytes: &[u8], seed: i64) -> PyResult<u32> {
    Ok(gxhash::gxhash32(input_bytes, seed))
}

#[pyfunction]
fn gxhash32_nogil(py: Python, input_bytes: &[u8], seed: i64) -> PyResult<u32> {
    py.allow_threads(|| Ok(gxhash::gxhash32(input_bytes, seed)))
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
fn gxhash128(input_bytes: &[u8], seed: i64) -> PyResult<u128> {
    Ok(gxhash::gxhash128(input_bytes, seed))
}

#[pyfunction]
fn gxhash128_nogil(py: Python, input_bytes: &[u8], seed: i64) -> PyResult<u128> {
    py.allow_threads(|| Ok(gxhash::gxhash128(input_bytes, seed)))
}

#[pymodule(name = "gxhash")]
fn pygxhash(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(gxhash32, m)?)?;
    m.add_function(wrap_pyfunction!(gxhash32_nogil, m)?)?;
    m.add_function(wrap_pyfunction!(gxhash64, m)?)?;
    m.add_function(wrap_pyfunction!(gxhash64_nogil, m)?)?;
    m.add_function(wrap_pyfunction!(gxhash128, m)?)?;
    m.add_function(wrap_pyfunction!(gxhash128_nogil, m)?)?;
    Ok(())
}
