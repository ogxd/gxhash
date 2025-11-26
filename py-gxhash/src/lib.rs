use pyo3::prelude::Bound;
use pyo3::prelude::PyResult;
use pyo3::prelude::Python;
use pyo3::prelude::pyclass;
use pyo3::prelude::pymethods;
use pyo3::types::PyModuleMethods;

#[cfg_attr(not(any(Py_3_8, Py_3_9)), pyclass(frozen, immutable_type))]
#[cfg_attr(any(Py_3_8, Py_3_9), pyclass(frozen))]
struct GxHash32 {
    seed: i64,
}

#[cfg_attr(not(any(Py_3_8, Py_3_9)), pyclass(frozen, immutable_type))]
#[cfg_attr(any(Py_3_8, Py_3_9), pyclass(frozen))]
struct GxHash64 {
    seed: i64,
}

#[cfg_attr(not(any(Py_3_8, Py_3_9)), pyclass(frozen, immutable_type))]
#[cfg_attr(any(Py_3_8, Py_3_9), pyclass(frozen))]
struct GxHash128 {
    seed: i64,
}

macro_rules! impl_gxhash_methods {
    ($Self:ident, $return_type:ty, $hasher:path) => {
        #[pymethods]
        impl $Self {
            #[new]
            fn new(seed: i64) -> Self {
                $Self { seed }
            }

            fn hash(&self, bytes: &[u8]) -> $return_type {
                $hasher(bytes, self.seed)
            }

            fn hash_async<'a>(&self, py: Python<'a>, bytes: pyo3::prelude::Py<pyo3::types::PyBytes>) -> PyResult<Bound<'a, pyo3::prelude::PyAny>> {
                let seed = self.seed;

                pyo3_async_runtimes::tokio::future_into_py(py, async move {
                    tokio::task::spawn_blocking(move || $hasher(Python::attach(|py| bytes.as_bytes(py)), seed))
                        .await
                        .map_err(|e| pyo3::exceptions::PyRuntimeError::new_err(format!("Task Join Error: {}", e)))
                })
            }
        }
    };
}

impl_gxhash_methods!(GxHash32, u32, gxhash::gxhash32);
impl_gxhash_methods!(GxHash64, u64, gxhash::gxhash64);
impl_gxhash_methods!(GxHash128, u128, gxhash::gxhash128);

#[pyo3::prelude::pymodule(name = "gxhash")]
fn pygxhash(m: &Bound<'_, pyo3::prelude::PyModule>) -> PyResult<()> {
    m.add_class::<GxHash32>()?;
    m.add_class::<GxHash64>()?;
    m.add_class::<GxHash128>()?;
    Ok(())
}
