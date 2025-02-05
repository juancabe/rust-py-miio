use pyo3::prelude::*;
use pyo3::types::PyList;

mod constants;

pub fn get_device_types(python_source_path: &str) -> Result<Vec<String>, PyErr> {
    Python::with_gil(|py| {
        // (Optionally add "./src" to sys.path if not already there)

        let sys = py.import("sys")?;
        let binding = sys.getattr("path")?;
        let path = binding.downcast::<PyList>()?;
        path.insert(0, python_source_path)?;

        // Import the Python module
        let miio_module = PyModule::import(py, "miio_interface")?;

        // Retrieve the Python function 'get_device_types'
        let get_device_types = miio_module.getattr("get_device_types")?;
        // Call the function without arguments
        let device_types_py = get_device_types.call0()?;
        // Convert Python list to Rust Vec<String>
        let v: Vec<String> = device_types_py.extract()?;
        Ok(v)
    })
}

#[cfg(test)]
mod tests {
    use super::constants::PYTHON_SOURCE_PATH;
    use super::*;

    #[test]
    fn test_get_device_types_success() {
        assert!(!get_device_types(PYTHON_SOURCE_PATH).unwrap().is_empty())
    }

    #[test]
    fn test_get_device_types_cherry_picked() {
        let device_types = get_device_types(PYTHON_SOURCE_PATH).unwrap();
        assert!(device_types.contains(&String::from("Yeelight")));
        assert!(device_types.contains(&String::from("DummyWifiRepeater")));
        assert!(device_types.contains(&String::from("DummyWalkingpad")));
        assert!(device_types.contains(&String::from("FanMiot")));
        assert!(device_types.contains(&String::from("DummyAirQualityMonitorB1")));
    }
}
