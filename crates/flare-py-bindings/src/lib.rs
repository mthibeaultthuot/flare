use flare::Flare;
use flare_codegen_metal::compile as compile_metal;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

#[pyclass]
struct FlareCompiler {}

#[pymethods]
impl FlareCompiler {
    #[new]
    fn new() -> Self {
        Self {}
    }

    pub fn compile_to_metal(&self, source: &str) -> PyResult<String> {
        let program = Flare::compile_from_string(source)
            .map_err(|e| PyRuntimeError::new_err(format!("failed to parse kernel: {:?}", e)))?;
        let metal_code = compile_metal(&program)
            .map_err(|e| PyRuntimeError::new_err(format!("failed to generate Metal : {:?}", e)))?;
        Ok(metal_code)
    }
}

#[pymodule]
fn flare_py_bindings(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_class::<FlareCompiler>()?;
    Ok(())
}
