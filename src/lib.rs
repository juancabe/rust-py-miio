//! This module provides an interface to interact with Miio devices via Python.
//!
//! It offers functions to retrieve available device types, create devices, and call device methods.
//! Devices are represented by the Device struct which supports serialization and deserialization.

use std::collections::HashMap;
use std::ffi::CString;

use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyDict, PyModule};

use serde::{Deserialize, Serialize};
use serde_json;

mod constants;

const MIIO_INTERFACE_CODE: &str = include_str!("../python-src/miio_interface.py");

/// Retrieves a list of available device types from the Python interface.
///
/// # Returns
///
/// * `Ok(Vec<String>)` - A vector of device type names if successful.
/// * `Err(PyErr)` - An error if the Python call fails.

pub fn get_device_types() -> Result<Vec<String>, PyErr> {
    Python::with_gil(|py| {
        // Import the Python module
        let miio_module = PyModule::from_code(
            py,
            CString::new(MIIO_INTERFACE_CODE)?.as_c_str(),
            &CString::new("miio_interface.py")?,
            &CString::new("miio_interface")?,
        )?;

        // Retrieve the Python function 'get_device_types'
        let get_device_types = miio_module.getattr("get_device_types")?;
        // Call the function without arguments
        let device_types_py = get_device_types.call0()?;
        // Convert Python list to Rust Vec<String>
        let v: Vec<String> = device_types_py.extract()?;
        Ok(v)
    })
}

/// Represents a Miio device with its associated properties and Python object.
///
/// The Device struct includes data necessary for device communication and method invocation,
/// along with functionalities to serialize/deserialize the device configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Device {
    /// The type of the device.
    device_type: String,
    /// The IP address of the device.
    ip: String,
    /// The token used for device authentication.
    token: String,
    /// A serialized representation of the underlying Python object as bytes.
    serialized_py_object: Vec<u8>,
    /// A map of callable method names to their corresponding Python signatures.
    callable_methods: HashMap<String, String>,
}

impl Device {
    /// Serializes the Device instance to a JSON file.
    ///
    /// # Arguments
    ///
    /// * `folder` - The directory where the file will be saved.
    /// * `file_name` - The name of the file to create.
    ///
    /// # Returns
    ///
    /// * `Ok(())` on success, or an std::io::Error on failure.
    pub fn serialize_to_file(&self, folder: &str, file_name: &str) -> std::io::Result<()> {
        let path = format!("{}/{}", folder, file_name);
        let json_str = serde_json::to_string_pretty(self)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))?;
        std::fs::write(path, json_str)
    }

    /// Deserializes a Device instance from a JSON file.
    ///
    /// # Arguments
    ///
    /// * `folder` - The directory containing the file.
    /// * `file_name` - The name of the file to read.
    ///
    /// # Returns
    ///
    /// * `Ok(Device)` if deserialization is successful.
    /// * `Err(std::io::Error)` if an error occurs during file read or parsing.
    pub fn deserialize_from_file(folder: &str, file_name: &str) -> std::io::Result<Device> {
        let path = format!("{}/{}", folder, file_name);
        let json_str = std::fs::read_to_string(path)?;
        serde_json::from_str(&json_str)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
    }

    /// Creates a new Device instance by invoking the Python function.
    ///
    /// This function calls the Python module to create a device and retrieve its properties,
    /// including serialized state and callable methods.
    ///
    /// # Arguments
    ///
    /// * `ip` - The IP address of the device.
    /// * `token` - The token used for authentication.
    /// * `device_type` - The type of the device.
    ///
    /// # Returns
    ///
    /// * `Ok(Device)` on success.
    /// * `Err(PyErr)` if any Python call fails.
    pub fn create_device(ip: &str, token: &str, device_type: &str) -> Result<Device, PyErr> {
        Python::with_gil(|py| {
            // Import the Python module
            let miio_module = PyModule::from_code(
                py,
                CString::new(MIIO_INTERFACE_CODE)?.as_c_str(),
                &CString::new("miio_interface.py")?,
                &CString::new("miio_interface")?,
            )?;

            // Retrieve the Python function 'create_device'
            let create_device = miio_module.getattr("get_device")?;
            // Call the function with arguments
            let device: Bound<'_, PyBytes> = create_device
                .call1((ip, token, device_type))?
                .downcast::<PyBytes>()?
                .clone();

            // Retrieve the Python function 'get_device_methods'
            let get_device_methods = miio_module.getattr("get_device_methods")?;
            // Call the function with arguments
            let methods = get_device_methods.call1((device.clone(),))?; // Dict returned
            let methods = methods.downcast::<PyDict>()?;
            let mut callable_methods = HashMap::new();
            for (key, value) in methods.iter() {
                let key = key.extract::<String>()?;
                let value = value.extract::<String>()?;
                callable_methods.insert(key, value);
            }

            let device_bytes = device.as_bytes().to_vec();
            Ok(Device {
                device_type: device_type.to_string(),
                ip: ip.to_string(),
                token: token.to_string(),
                serialized_py_object: device_bytes,
                callable_methods,
            })
        })
    }

    /// Calls a method on the device by invoking the corresponding Python function.
    ///
    /// This function sends a command to the device through Python and returns the result.
    ///
    /// # Arguments
    ///
    /// * `method_name` - The name of the method to be called.
    /// * `args` - A vector of string arguments for the method.
    ///
    /// # Returns
    ///
    /// * `Ok(String)` containing the result if successful.
    /// * `Err(PyErr)` if the Python call fails.
    pub fn call_method(&self, method_name: &str, args: Vec<&str>) -> Result<String, PyErr> {
        Python::with_gil(|py| {
            // Import the Python module
            let miio_module = PyModule::from_code(
                py,
                CString::new(MIIO_INTERFACE_CODE)?.as_c_str(),
                &CString::new("miio_interface.py")?,
                &CString::new("miio_interface")?,
            )?;

            // Retrieve the Python function 'call_method'
            let call_method = miio_module.getattr("call_method")?;
            // Call the function with arguments
            let result: String = call_method
                .call1((self.serialized_py_object.clone(), method_name, args))?
                .extract()?;
            Ok(result)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::constants::*;
    use super::*;

    #[test]
    fn test_python_path() {
        // eprintln!("Python interpreter path loading...");
        let res: Result<(), PyErr> = Python::with_gil(|py| {
            let sys = PyModule::import(py, "sys")?;
            let path = sys.getattr("path")?;
            let path: Vec<String> = path.extract()?;
            if path.is_empty() {
                return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                    "Python path is empty",
                ));
            }
            Ok(())
        });
        assert!(res.is_ok());
    }

    #[test]
    fn test_get_device_types_success() {
        assert!(!get_device_types().unwrap().is_empty())
    }

    #[test]
    fn test_get_device_types_cherry_picked() {
        let device_types = get_device_types().unwrap();
        assert!(device_types.contains(&String::from("Yeelight")));
        assert!(device_types.contains(&String::from("FanMiot")));
        assert!(device_types.contains(&String::from("AirHumidifierMiot")));
    }

    #[test]
    fn test_get_device_types_cherry_picked_ne() {
        let device_types = get_device_types().unwrap();
        assert!(!device_types.contains(&String::from("Yeeli")));
        assert!(!device_types.contains(&String::from("DummyStupidWifiRepeater")));
        assert!(!device_types.contains(&String::from("DummySleepingpad")));
        assert!(!device_types.contains(&String::from("Fanatico")));
        assert!(!device_types.contains(&String::from("Yourmama")));
    }

    #[test]
    fn test_create_device_success() {
        let device = Device::create_device(IP, TOKEN, DEVICE_TYPE).unwrap();
        assert_eq!(device.device_type, DEVICE_TYPE);
        assert_eq!(device.ip, IP);
        assert_eq!(device.token, TOKEN);
        assert!(!device.serialized_py_object.is_empty());
    }

    #[test]
    fn test_get_device_types_error() {
        assert!(!Device::create_device("0.0.0.0.0.0.0.0.0", TOKEN, DEVICE_TYPE,).is_ok());
        assert!(!Device::create_device(IP, "tokennnnnnnnnnnnnn", DEVICE_TYPE,).is_ok());
        assert!(!Device::create_device(IP, TOKEN, "NotADeviceYkkkkkkkkkkkkk",).is_ok());
    }

    #[test]
    fn test_get_device_methods() {
        let device = Device::create_device(IP, TOKEN, DEVICE_TYPE).unwrap();
        assert!(!device.callable_methods.is_empty());
        assert!(device.callable_methods.contains_key("toggle"));
    }

    #[test]
    fn test_call_method() {
        let device = Device::create_device(IP, TOKEN, DEVICE_TYPE).unwrap();
        let result = device.call_method(METHOD_NAME, vec![]).unwrap();
        assert_eq!(result, "['ok']");
    }

    #[test]
    fn test_serialize_to_file() {
        let device = Device::create_device(IP, TOKEN, DEVICE_TYPE).unwrap();
        let folder = std::env::temp_dir();
        let folder = folder.to_str().unwrap();
        let file_name = "device.json";
        device.serialize_to_file(folder, file_name).unwrap();
        let path = format!("{}/{}", folder, file_name);
        assert!(std::fs::metadata(path).is_ok());
    }

    #[test]
    fn test_serialize_deserialize_to_file() {
        let device = Device::create_device(IP, TOKEN, DEVICE_TYPE).unwrap();
        let folder = std::env::temp_dir();
        let folder = folder.to_str().unwrap();
        let file_name = "device.json";
        device.serialize_to_file(folder, file_name).unwrap();
        let deserialized_device = Device::deserialize_from_file(folder, file_name).unwrap();
        assert_eq!(device.device_type, deserialized_device.device_type);
        assert_eq!(device.ip, deserialized_device.ip);
        assert_eq!(device.token, deserialized_device.token);
        assert_eq!(
            device.serialized_py_object,
            deserialized_device.serialized_py_object
        );
        assert_eq!(
            device.callable_methods,
            deserialized_device.callable_methods
        );
    }
}
