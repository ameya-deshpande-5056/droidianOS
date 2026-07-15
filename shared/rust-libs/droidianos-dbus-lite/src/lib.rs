use std::ffi::{CStr, CString};
use std::io;
use std::os::raw::{c_char, c_int, c_uint, c_void};
use std::ptr;

const DBUS_BUS_SESSION: c_int = 0;
const DBUS_TYPE_INVALID: c_int = 0;
const DBUS_TYPE_STRING: c_int = 115;
const DBUS_TYPE_UINT32: c_int = 117;

#[repr(C)]
struct DBusConnection {
    _private: [u8; 0],
}

#[repr(C)]
struct DBusMessage {
    _private: [u8; 0],
}

#[link(name = "dbus-1")]
extern "C" {
    fn dbus_bus_get(bus_type: c_int, error: *mut c_void) -> *mut DBusConnection;
    fn dbus_bus_request_name(
        connection: *mut DBusConnection,
        name: *const c_char,
        flags: c_uint,
        error: *mut c_void,
    ) -> c_int;
    fn dbus_connection_read_write(connection: *mut DBusConnection, timeout_milliseconds: c_int)
        -> c_int;
    fn dbus_connection_pop_message(connection: *mut DBusConnection) -> *mut DBusMessage;
    fn dbus_connection_send(
        connection: *mut DBusConnection,
        message: *mut DBusMessage,
        serial: *mut c_uint,
    ) -> c_int;
    fn dbus_connection_flush(connection: *mut DBusConnection);
    fn dbus_message_is_method_call(
        message: *mut DBusMessage,
        interface: *const c_char,
        method: *const c_char,
    ) -> c_int;
    fn dbus_message_new_method_call(
        destination: *const c_char,
        path: *const c_char,
        interface: *const c_char,
        method: *const c_char,
    ) -> *mut DBusMessage;
    fn dbus_message_new_method_return(message: *mut DBusMessage) -> *mut DBusMessage;
    fn dbus_message_new_signal(
        path: *const c_char,
        interface: *const c_char,
        name: *const c_char,
    ) -> *mut DBusMessage;
    fn dbus_message_new_error(
        message: *mut DBusMessage,
        error_name: *const c_char,
        error_message: *const c_char,
    ) -> *mut DBusMessage;
    fn dbus_message_get_args(message: *mut DBusMessage, error: *mut c_void, first_arg_type: c_int, ...) -> c_int;
    fn dbus_message_append_args(message: *mut DBusMessage, first_arg_type: c_int, ...) -> c_int;
    fn dbus_connection_send_with_reply_and_block(
        connection: *mut DBusConnection,
        message: *mut DBusMessage,
        timeout_milliseconds: c_int,
        error: *mut c_void,
    ) -> *mut DBusMessage;
    fn dbus_message_unref(message: *mut DBusMessage);
}

pub struct Connection {
    raw: *mut DBusConnection,
}

pub struct Message {
    raw: *mut DBusMessage,
}

impl Connection {
    pub fn session_with_name(name: &str) -> io::Result<Self> {
        let name = cstring(name)?;
        let raw = unsafe { dbus_bus_get(DBUS_BUS_SESSION, ptr::null_mut()) };
        if raw.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionRefused,
                "failed to connect to the session bus",
            ));
        }

        let request_result = unsafe { dbus_bus_request_name(raw, name.as_ptr(), 0, ptr::null_mut()) };
        if request_result <= 0 {
            return Err(io::Error::new(
                io::ErrorKind::AddrInUse,
                "failed to own D-Bus service name",
            ));
        }

        Ok(Self { raw })
    }

    pub fn session() -> io::Result<Self> {
        let raw = unsafe { dbus_bus_get(DBUS_BUS_SESSION, ptr::null_mut()) };
        if raw.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::ConnectionRefused,
                "failed to connect to the session bus",
            ));
        }

        Ok(Self { raw })
    }

    pub fn call_string_method_one_arg(
        &self,
        destination: &str,
        path: &str,
        interface: &str,
        method: &str,
        argument: &str,
        timeout_milliseconds: i32,
    ) -> io::Result<String> {
        let destination = cstring(destination)?;
        let path = cstring(path)?;
        let interface = cstring(interface)?;
        let method = cstring(method)?;
        let argument = cstring(argument)?;
        let mut argument_ptr = argument.as_ptr();

        unsafe {
            let request = dbus_message_new_method_call(
                destination.as_ptr(),
                path.as_ptr(),
                interface.as_ptr(),
                method.as_ptr(),
            );
            if request.is_null() {
                return Err(io::Error::new(io::ErrorKind::Other, "failed to create D-Bus method call"));
            }

            dbus_message_append_args(
                request,
                DBUS_TYPE_STRING,
                &mut argument_ptr as *mut *const c_char,
                DBUS_TYPE_INVALID,
            );

            let reply = dbus_connection_send_with_reply_and_block(
                self.raw,
                request,
                timeout_milliseconds,
                ptr::null_mut(),
            );
            dbus_message_unref(request);

            if reply.is_null() {
                return Err(io::Error::new(io::ErrorKind::TimedOut, "D-Bus method call failed"));
            }

            let message = Message { raw: reply };
            message.string_arg()
        }
    }

    pub fn next_message(&self, timeout_milliseconds: i32) -> Option<Message> {
        unsafe {
            dbus_connection_read_write(self.raw, timeout_milliseconds);
            let raw = dbus_connection_pop_message(self.raw);
            if raw.is_null() {
                None
            } else {
                Some(Message { raw })
            }
        }
    }

    pub fn send_empty_reply(&self, message: &Message) {
        unsafe {
            let reply = dbus_message_new_method_return(message.raw);
            self.send_raw(reply);
        }
    }

    pub fn send_string_reply(&self, message: &Message, value: &str) {
        let value = match cstring(value) {
            Ok(value) => value,
            Err(error) => {
                self.send_error_reply(message, &error.to_string());
                return;
            }
        };
        let mut value_ptr = value.as_ptr();

        unsafe {
            let reply = dbus_message_new_method_return(message.raw);
            if reply.is_null() {
                return;
            }
            dbus_message_append_args(
                reply,
                DBUS_TYPE_STRING,
                &mut value_ptr as *mut *const c_char,
                DBUS_TYPE_INVALID,
            );
            self.send_raw(reply);
        }
    }

    pub fn send_error_reply(&self, message: &Message, error: &str) {
        let error_name = match cstring("org.droidianos.Error.Failed") {
            Ok(value) => value,
            Err(_) => return,
        };
        let error_message = match cstring(error) {
            Ok(value) => value,
            Err(_) => match cstring("operation failed") {
                Ok(value) => value,
                Err(_) => return,
            },
        };

        unsafe {
            let reply = dbus_message_new_error(message.raw, error_name.as_ptr(), error_message.as_ptr());
            self.send_raw(reply);
        }
    }

    pub fn send_string_pair_signal(&self, path: &str, interface: &str, name: &str, first: &str, second: &str) {
        let path = match cstring(path) {
            Ok(value) => value,
            Err(_) => return,
        };
        let interface = match cstring(interface) {
            Ok(value) => value,
            Err(_) => return,
        };
        let name = match cstring(name) {
            Ok(value) => value,
            Err(_) => return,
        };
        let first = match cstring(first) {
            Ok(value) => value,
            Err(_) => return,
        };
        let second = match cstring(second) {
            Ok(value) => value,
            Err(_) => return,
        };
        let mut first_ptr = first.as_ptr();
        let mut second_ptr = second.as_ptr();

        unsafe {
            let signal = dbus_message_new_signal(path.as_ptr(), interface.as_ptr(), name.as_ptr());
            if signal.is_null() {
                return;
            }
            dbus_message_append_args(
                signal,
                DBUS_TYPE_STRING,
                &mut first_ptr as *mut *const c_char,
                DBUS_TYPE_STRING,
                &mut second_ptr as *mut *const c_char,
                DBUS_TYPE_INVALID,
            );
            self.send_raw(signal);
        }
    }

    pub fn send_string_signal(&self, path: &str, interface: &str, name: &str, value: &str) {
        let path = match cstring(path) {
            Ok(value) => value,
            Err(_) => return,
        };
        let interface = match cstring(interface) {
            Ok(value) => value,
            Err(_) => return,
        };
        let name = match cstring(name) {
            Ok(value) => value,
            Err(_) => return,
        };
        let value = match cstring(value) {
            Ok(value) => value,
            Err(_) => return,
        };
        let mut value_ptr = value.as_ptr();

        unsafe {
            let signal = dbus_message_new_signal(path.as_ptr(), interface.as_ptr(), name.as_ptr());
            if signal.is_null() {
                return;
            }
            dbus_message_append_args(
                signal,
                DBUS_TYPE_STRING,
                &mut value_ptr as *mut *const c_char,
                DBUS_TYPE_INVALID,
            );
            self.send_raw(signal);
        }
    }


    pub fn send_progress_signal(
        &self,
        path: &str,
        interface: &str,
        name: &str,
        transaction_id: &str,
        percent: u32,
        message: &str,
    ) {
        let path = match cstring(path) {
            Ok(value) => value,
            Err(_) => return,
        };
        let interface = match cstring(interface) {
            Ok(value) => value,
            Err(_) => return,
        };
        let name = match cstring(name) {
            Ok(value) => value,
            Err(_) => return,
        };
        let transaction_id = match cstring(transaction_id) {
            Ok(value) => value,
            Err(_) => return,
        };
        let message = match cstring(message) {
            Ok(value) => value,
            Err(_) => return,
        };
        let mut transaction_id_ptr = transaction_id.as_ptr();
        let mut percent_value = percent;
        let mut message_ptr = message.as_ptr();

        unsafe {
            let signal = dbus_message_new_signal(path.as_ptr(), interface.as_ptr(), name.as_ptr());
            if signal.is_null() {
                return;
            }
            dbus_message_append_args(
                signal,
                DBUS_TYPE_STRING,
                &mut transaction_id_ptr as *mut *const c_char,
                DBUS_TYPE_UINT32,
                &mut percent_value as *mut u32,
                DBUS_TYPE_STRING,
                &mut message_ptr as *mut *const c_char,
                DBUS_TYPE_INVALID,
            );
            self.send_raw(signal);
        }
    }

    unsafe fn send_raw(&self, message: *mut DBusMessage) {
        if message.is_null() {
            return;
        }
        dbus_connection_send(self.raw, message, ptr::null_mut());
        dbus_connection_flush(self.raw);
        dbus_message_unref(message);
    }
}

impl Message {
    pub fn is_method(&self, interface: &str, method: &str) -> bool {
        let interface = match cstring(interface) {
            Ok(value) => value,
            Err(_) => return false,
        };
        let method = match cstring(method) {
            Ok(value) => value,
            Err(_) => return false,
        };

        unsafe { dbus_message_is_method_call(self.raw, interface.as_ptr(), method.as_ptr()) != 0 }
    }

    pub fn string_arg(&self) -> io::Result<String> {
        let mut value_ptr: *const c_char = ptr::null();
        let result = unsafe {
            dbus_message_get_args(
                self.raw,
                ptr::null_mut(),
                DBUS_TYPE_STRING,
                &mut value_ptr as *mut *const c_char,
                DBUS_TYPE_INVALID,
            )
        };

        if result == 0 || value_ptr.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "method requires one string argument",
            ));
        }

        unsafe {
            CStr::from_ptr(value_ptr)
                .to_str()
                .map(|value| value.to_string())
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "argument is not valid UTF-8"))
        }
    }

    pub fn string_pair_args(&self) -> io::Result<(String, String)> {
        let mut first_ptr: *const c_char = ptr::null();
        let mut second_ptr: *const c_char = ptr::null();
        let result = unsafe {
            dbus_message_get_args(
                self.raw,
                ptr::null_mut(),
                DBUS_TYPE_STRING,
                &mut first_ptr as *mut *const c_char,
                DBUS_TYPE_STRING,
                &mut second_ptr as *mut *const c_char,
                DBUS_TYPE_INVALID,
            )
        };

        if result == 0 || first_ptr.is_null() || second_ptr.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "method requires two string arguments",
            ));
        }

        unsafe {
            let first = CStr::from_ptr(first_ptr)
                .to_str()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "argument is not valid UTF-8"))?
                .to_string();
            let second = CStr::from_ptr(second_ptr)
                .to_str()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "argument is not valid UTF-8"))?
                .to_string();
            Ok((first, second))
        }
    }

    pub fn string_triple_args(&self) -> io::Result<(String, String, String)> {
        let mut first_ptr: *const c_char = ptr::null();
        let mut second_ptr: *const c_char = ptr::null();
        let mut third_ptr: *const c_char = ptr::null();
        let result = unsafe {
            dbus_message_get_args(
                self.raw,
                ptr::null_mut(),
                DBUS_TYPE_STRING,
                &mut first_ptr as *mut *const c_char,
                DBUS_TYPE_STRING,
                &mut second_ptr as *mut *const c_char,
                DBUS_TYPE_STRING,
                &mut third_ptr as *mut *const c_char,
                DBUS_TYPE_INVALID,
            )
        };

        if result == 0 || first_ptr.is_null() || second_ptr.is_null() || third_ptr.is_null() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "method requires three string arguments",
            ));
        }

        unsafe {
            let first = CStr::from_ptr(first_ptr)
                .to_str()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "argument is not valid UTF-8"))?
                .to_string();
            let second = CStr::from_ptr(second_ptr)
                .to_str()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "argument is not valid UTF-8"))?
                .to_string();
            let third = CStr::from_ptr(third_ptr)
                .to_str()
                .map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "argument is not valid UTF-8"))?
                .to_string();
            Ok((first, second, third))
        }
    }
}

impl Drop for Message {
    fn drop(&mut self) {
        unsafe {
            dbus_message_unref(self.raw);
        }
    }
}

fn cstring(value: &str) -> io::Result<CString> {
    CString::new(value).map_err(|_| {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            "string contains an unsupported NUL byte",
        )
    })
}
