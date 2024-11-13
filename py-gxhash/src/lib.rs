use pyo3::prelude::*;

#[pyfunction]
fn gxhash32(input: &[u8], seed: i64) -> PyResult<u32> {
    Ok(gxhash::gxhash32(input, seed))
}

#[pyfunction]
fn gxhash64(input: &[u8], seed: i64) -> PyResult<u64> {
    Ok(gxhash::gxhash64(input, seed))
}

#[pyfunction]
fn gxhash128(input: &[u8], seed: i64) -> PyResult<u128> {
    Ok(gxhash::gxhash128(input, seed))
}

#[pymodule(name = "gxhash")]
fn pygxhash(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(gxhash32, m)?)?;
    m.add_function(wrap_pyfunction!(gxhash64, m)?)?;
    m.add_function(wrap_pyfunction!(gxhash128, m)?)?;
    Ok(())
}
