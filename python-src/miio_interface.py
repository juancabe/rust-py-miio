import pkgutil
import importlib
import inspect
from typing import List, Dict, Set, Type
from miio.device import Device
import pickle

# Define type aliases for clarity.
DeviceTypeName = str
DeviceTypeClass = Type[Device]
CallableMethodSignature = str

def _load_integration_modules() -> None:
    """
    Dynamically loads all integration modules within the 'miio.integrations' package.

    This function iterates over all modules contained in the 'miio.integrations' package
    and imports them. Any exceptions during import are silently ignored.

    Returns:
        None
    """
    package = importlib.import_module("miio.integrations")
    for module_info in pkgutil.walk_packages(package.__path__, package.__name__ + "."):
        try:
            importlib.import_module(module_info.name)
        except Exception:
            continue

def get_device_types() -> List[DeviceTypeName]:
    """
    Retrieves a list of available device type names.

    This function loads all integration modules and recursively collects subclasses of the Device class.
    The names of these device types are collected and returned as a list.

    Returns:
        List[DeviceTypeName]: A list of strings representing the available device type names.
    """
    _load_integration_modules()
    device_classes: Set[DeviceTypeClass] = set()

    def recurse_subclasses(cls: Type[Device]) -> None:
        """
        Recursively add all subclasses of the given Device class.

        Args:
            cls (Type[Device]): The base Device class or its subclass to inspect.
        """
        for subclass in cls.__subclasses__():
            device_classes.add(subclass)
            recurse_subclasses(subclass)

    recurse_subclasses(Device)
    return [cls.__name__ for cls in device_classes]

def _get_device_object(ip: str, token: str, device_type: DeviceTypeName) -> Device:
    """
    Constructs a device instance of the specified type.

    This function loads integration modules and searches for a Device subclass
    that matches the provided device type name. When a match is found, it instantiates
    the device object using the provided IP and token.

    Args:
        ip (str): The IP address of the device.
        token (str): The authentication token of the device.
        device_type (DeviceTypeName): The name of the device type to instantiate.

    Returns:
        Device: An instance of the device corresponding to device_type.

    Raises:
        ValueError: If no matching device type is found.
    """
    _load_integration_modules()
    device_classes: Set[DeviceTypeClass] = set()

    def recurse_subclasses(cls: Type[Device]) -> None:
        """
        Recursively add all subclasses of the given Device class.

        Args:
            cls (Type[Device]): The base Device class or its subclass to inspect.
        """
        for subclass in cls.__subclasses__():
            device_classes.add(subclass)
            recurse_subclasses(subclass)

    recurse_subclasses(Device)
    for cls in device_classes:
        if cls.__name__ == device_type:
            return cls(ip, token)
    raise ValueError(f"Device type '{device_type}' not found")

def _get_device_bytes(device: Device) -> bytes:
    return pickle.dumps(device)

def get_device(ip: str, token: str, device_type: DeviceTypeName) -> bytes:
    """
    Constructs a device object of the specified type and serializes it to bytes.

    This function creates a device instance of the specified type using the provided IP
    address and token. The resulting device object is then serialized to bytes using the
    pickle module.

    Args:
        ip (str): The IP address of the device.
        token (str): The authentication token of the device.
        device_type (DeviceTypeName): The name of the device type to instantiate.

    Returns:
        bytes: A serialized representation of the device object.

    Raises:
        ValueError: If no matching device type is found.
    """
    device = _get_device_object(ip, token, device_type)
    return _get_device_bytes(device)

def _get_device_methods(device: Device) -> Dict[str, CallableMethodSignature]:
    """
    Retrieves information about all callable methods of the provided device object.

    This function uses Python's inspect module to extract the method signatures for all
    callable members of the device that do not start with an underscore. If the signature
    cannot be determined, a default message is stored.

    Args:
        device (Device): The device instance from which to retrieve method information.

    Returns:
        Dict[str, CallableMethodSignature]: A dictionary where each key is a callable method name
        and the value is a string representation of its parameter signature.
    """
    methods_info: Dict[str, CallableMethodSignature] = {}
    for name, member in inspect.getmembers(device, predicate=callable):
        if not name.startswith("_"):
            try:
                sig = inspect.signature(member)
                methods_info[name] = str(sig)
            except Exception:
                methods_info[name] = "No signature available"
    return methods_info

def get_device_methods(device: bytes) -> Dict[str, CallableMethodSignature]:
    """
    Deserializes a device object from bytes and retrieves information about its callable methods.

    This function deserializes the device object from bytes using the pickle module and then
    calls _get_device_methods to retrieve information about its callable methods.

    Args:
        device (bytes): A serialized representation of the device object.

    Returns:
        Dict[str, CallableMethodSignature]: A dictionary where each key is a callable method name
        and the value is a string representation of its parameter signature.
    """
    device = pickle.loads(device)
    return _get_device_methods(device)

def _call_method(device: Device, method_name: str, args: List[str]) -> str:
    try:
        method = getattr(device, method_name, None)
        if not method or not callable(method):
            raise ValueError(f"Method '{method_name}' not found on device {type(device).__name__}")
        result = method(*args)
        return str(result)
    except Exception as e:
        return f"Error calling method '{method_name}': {e}"
    
def call_method(device: bytes, method_name: str, args: List[str]) -> str:
    device = pickle.loads(device)
    return _call_method(device, method_name, args)

if __name__ == "__main__":
    print("Available device types:", get_device_types())

    device_type = "Yeelight"

    import constants
    ip = constants.ip
    token = constants.token
    device = get_device(ip, token, device_type)
    methods_info = get_device_methods(device)
    print("Callable methods with parameters:")
    for method, params in methods_info.items():
        print(f"{method}{params}")

    # Call a method that contains the string str in its name
    toggle_methods = [method for method in methods_info if "set_color_temperature" in method]
    if toggle_methods:
        toggle_method = toggle_methods[0]
        result = call_method(device, toggle_method, [2700])
        print(f"Result of calling {toggle_method}: {result}")
    else:
        print("No toggle methods found")