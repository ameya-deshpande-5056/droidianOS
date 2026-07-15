use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;

const RES_STRING_POOL_TYPE: u16 = 0x0001;
const RES_XML_START_ELEMENT_TYPE: u16 = 0x0102;
const UTF8_FLAG: u32 = 0x0000_0100;
const TYPE_STRING: u8 = 0x03;
const TYPE_INT_DEC: u8 = 0x10;
const TYPE_INT_HEX: u8 = 0x11;
const TYPE_REFERENCE: u8 = 0x01;

#[derive(Debug)]
pub struct ApkMetadata {
    pub package_name: Option<String>,
    pub version_name: Option<String>,
    pub version_code: Option<String>,
    pub app_label: Option<String>,
    pub permissions: Vec<Permission>,
    pub apk_size_bytes: u64,
}

#[derive(Debug)]
pub struct Permission {
    pub id: String,
    pub class_name: String,
}

#[derive(Debug)]
struct Attribute {
    name: String,
    value: Option<String>,
}

#[derive(Debug)]
struct Element {
    name: String,
    attributes: Vec<Attribute>,
}

struct Cursor<'a> {
    bytes: &'a [u8],
}

impl ApkMetadata {
    pub fn to_json(&self) -> String {
        let mut json = String::from("{");
        push_json_field(&mut json, "package", self.package_name.as_deref(), false);
        push_json_field(&mut json, "version_name", self.version_name.as_deref(), true);
        push_json_field(&mut json, "version_code", self.version_code.as_deref(), true);
        push_json_field(&mut json, "app_label", self.app_label.as_deref(), true);
        json.push_str(",\"apk_size_bytes\":");
        json.push_str(&self.apk_size_bytes.to_string());
        json.push_str(",\"permissions\":[");

        for (index, permission) in self.permissions.iter().enumerate() {
            if index > 0 {
                json.push(',');
            }
            json.push_str("{\"id\":\"");
            json.push_str(&escape_json(&permission.id));
            json.push_str("\",\"class\":\"");
            json.push_str(&escape_json(&permission.class_name));
            json.push_str("\"}");
        }

        json.push_str("]}");
        json
    }
}

pub fn inspect_apk<P: AsRef<Path>>(path: P) -> io::Result<ApkMetadata> {
    let path = path.as_ref();
    let manifest = extract_manifest(path)?;
    let elements = parse_manifest(&manifest)?;
    let apk_size_bytes = fs::metadata(path)?.len();

    Ok(metadata_from_elements(elements, apk_size_bytes))
}

fn extract_manifest(path: &Path) -> io::Result<Vec<u8>> {
    let output = Command::new("unzip")
        .arg("-p")
        .arg(path)
        .arg("AndroidManifest.xml")
        .output()?;

    if !output.status.success() || output.stdout.is_empty() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            format!("failed to extract AndroidManifest.xml: {}", stderr.trim()),
        ));
    }

    Ok(output.stdout)
}

fn metadata_from_elements(elements: Vec<Element>, apk_size_bytes: u64) -> ApkMetadata {
    let mut metadata = ApkMetadata {
        package_name: None,
        version_name: None,
        version_code: None,
        app_label: None,
        permissions: Vec::new(),
        apk_size_bytes,
    };

    for element in elements {
        match element.name.as_str() {
            "manifest" => {
                metadata.package_name = attr_value(&element, "package");
                metadata.version_name = attr_value(&element, "versionName");
                metadata.version_code = attr_value(&element, "versionCode");
            }
            "application" => {
                if metadata.app_label.is_none() {
                    metadata.app_label = attr_value(&element, "label");
                }
            }
            "uses-permission" | "uses-permission-sdk-23" => {
                if let Some(id) = attr_value(&element, "name") {
                    metadata.permissions.push(Permission {
                        class_name: classify_permission(&id).to_string(),
                        id,
                    });
                }
            }
            _ => {}
        }
    }

    metadata.permissions.sort_by(|left, right| left.id.cmp(&right.id));
    metadata
        .permissions
        .dedup_by(|left, right| left.id.as_str() == right.id.as_str());
    metadata
}

fn attr_value(element: &Element, name: &str) -> Option<String> {
    element
        .attributes
        .iter()
        .find(|attribute| attribute.name == name)
        .and_then(|attribute| attribute.value.clone())
}

fn parse_manifest(bytes: &[u8]) -> io::Result<Vec<Element>> {
    let cursor = Cursor { bytes };
    let mut offset = 8usize;
    let mut strings = Vec::new();
    let mut elements = Vec::new();

    if bytes.len() < 8 {
        return Err(invalid_data("manifest is too small"));
    }

    while offset + 8 <= bytes.len() {
        let chunk_type = cursor.u16(offset)?;
        let header_size = cursor.u16(offset + 2)? as usize;
        let chunk_size = cursor.u32(offset + 4)? as usize;

        if chunk_size < 8 || offset + chunk_size > bytes.len() {
            return Err(invalid_data("manifest chunk is invalid"));
        }

        if chunk_type == RES_STRING_POOL_TYPE {
            strings = parse_string_pool(bytes, offset)?;
        } else if chunk_type == RES_XML_START_ELEMENT_TYPE {
            if let Some(element) = parse_start_element(bytes, offset, header_size, &strings)? {
                elements.push(element);
            }
        }

        offset += chunk_size;
    }

    if strings.is_empty() {
        return Err(invalid_data("manifest string pool not found"));
    }

    Ok(elements)
}

fn parse_string_pool(bytes: &[u8], chunk_offset: usize) -> io::Result<Vec<String>> {
    let cursor = Cursor { bytes };
    let header_size = cursor.u16(chunk_offset + 2)? as usize;
    let string_count = cursor.u32(chunk_offset + 8)? as usize;
    let flags = cursor.u32(chunk_offset + 16)?;
    let strings_start = cursor.u32(chunk_offset + 20)? as usize;
    let offsets_start = chunk_offset + header_size;
    let string_data_start = chunk_offset + strings_start;
    let utf8 = flags & UTF8_FLAG != 0;
    let mut strings = Vec::with_capacity(string_count);

    for index in 0..string_count {
        let string_offset = cursor.u32(offsets_start + index * 4)? as usize;
        let value_offset = string_data_start + string_offset;
        let value = if utf8 {
            read_utf8_string(bytes, value_offset)?
        } else {
            read_utf16_string(bytes, value_offset)?
        };
        strings.push(value);
    }

    Ok(strings)
}

fn parse_start_element(
    bytes: &[u8],
    chunk_offset: usize,
    header_size: usize,
    strings: &[String],
) -> io::Result<Option<Element>> {
    let cursor = Cursor { bytes };
    let name_index = cursor.u32(chunk_offset + 20)? as usize;
    let attribute_start = cursor.u16(chunk_offset + 24)? as usize;
    let attribute_size = cursor.u16(chunk_offset + 26)? as usize;
    let attribute_count = cursor.u16(chunk_offset + 28)? as usize;
    let attributes_offset = chunk_offset + header_size + attribute_start;

    let name = match strings.get(name_index) {
        Some(name) => name.clone(),
        None => return Ok(None),
    };
    let mut attributes = Vec::new();

    for index in 0..attribute_count {
        let offset = attributes_offset + index * attribute_size;
        let name_index = cursor.u32(offset + 4)? as usize;
        let raw_value_index = cursor.u32(offset + 8)?;
        let data_type = cursor.u8(offset + 15)?;
        let data = cursor.u32(offset + 16)?;
        let name = match strings.get(name_index) {
            Some(name) => name.clone(),
            None => continue,
        };
        let value = typed_value(strings, raw_value_index, data_type, data);
        attributes.push(Attribute { name, value });
    }

    Ok(Some(Element { name, attributes }))
}

fn typed_value(
    strings: &[String],
    raw_value_index: u32,
    data_type: u8,
    data: u32,
) -> Option<String> {
    if raw_value_index != u32::MAX {
        return strings.get(raw_value_index as usize).cloned();
    }

    match data_type {
        TYPE_STRING => strings.get(data as usize).cloned(),
        TYPE_INT_DEC => Some(data.to_string()),
        TYPE_INT_HEX => Some(format!("0x{:x}", data)),
        TYPE_REFERENCE => Some(format!("@0x{:08x}", data)),
        _ => None,
    }
}

fn read_utf8_string(bytes: &[u8], offset: usize) -> io::Result<String> {
    let (_, next_offset) = read_length8(bytes, offset)?;
    let (byte_length, data_offset) = read_length8(bytes, next_offset)?;
    let end = data_offset + byte_length;

    if end > bytes.len() {
        return Err(invalid_data("UTF-8 string is out of bounds"));
    }

    String::from_utf8(bytes[data_offset..end].to_vec())
        .map_err(|_| invalid_data("UTF-8 string is invalid"))
}

fn read_utf16_string(bytes: &[u8], offset: usize) -> io::Result<String> {
    let (char_length, data_offset) = read_length16(bytes, offset)?;
    let mut values = Vec::with_capacity(char_length);

    for index in 0..char_length {
        let value_offset = data_offset + index * 2;
        if value_offset + 2 > bytes.len() {
            return Err(invalid_data("UTF-16 string is out of bounds"));
        }
        values.push(u16::from_le_bytes([bytes[value_offset], bytes[value_offset + 1]]));
    }

    String::from_utf16(&values).map_err(|_| invalid_data("UTF-16 string is invalid"))
}

fn read_length8(bytes: &[u8], offset: usize) -> io::Result<(usize, usize)> {
    if offset >= bytes.len() {
        return Err(invalid_data("length is out of bounds"));
    }

    let first = bytes[offset] as usize;
    if first & 0x80 == 0 {
        Ok((first, offset + 1))
    } else {
        if offset + 1 >= bytes.len() {
            return Err(invalid_data("length is out of bounds"));
        }
        Ok((((first & 0x7f) << 8) | bytes[offset + 1] as usize, offset + 2))
    }
}

fn read_length16(bytes: &[u8], offset: usize) -> io::Result<(usize, usize)> {
    if offset + 2 > bytes.len() {
        return Err(invalid_data("length is out of bounds"));
    }

    let first = u16::from_le_bytes([bytes[offset], bytes[offset + 1]]) as usize;
    if first & 0x8000 == 0 {
        Ok((first, offset + 2))
    } else {
        if offset + 4 > bytes.len() {
            return Err(invalid_data("length is out of bounds"));
        }
        let second = u16::from_le_bytes([bytes[offset + 2], bytes[offset + 3]]) as usize;
        Ok((((first & 0x7fff) << 16) | second, offset + 4))
    }
}

impl<'a> Cursor<'a> {
    fn u8(&self, offset: usize) -> io::Result<u8> {
        self.bytes
            .get(offset)
            .copied()
            .ok_or_else(|| invalid_data("read is out of bounds"))
    }

    fn u16(&self, offset: usize) -> io::Result<u16> {
        if offset + 2 > self.bytes.len() {
            return Err(invalid_data("read is out of bounds"));
        }
        Ok(u16::from_le_bytes([self.bytes[offset], self.bytes[offset + 1]]))
    }

    fn u32(&self, offset: usize) -> io::Result<u32> {
        if offset + 4 > self.bytes.len() {
            return Err(invalid_data("read is out of bounds"));
        }
        Ok(u32::from_le_bytes([
            self.bytes[offset],
            self.bytes[offset + 1],
            self.bytes[offset + 2],
            self.bytes[offset + 3],
        ]))
    }
}

fn classify_permission(permission: &str) -> &'static str {
    match permission {
        "android.permission.CAMERA"
        | "android.permission.RECORD_AUDIO"
        | "android.permission.ACCESS_FINE_LOCATION"
        | "android.permission.ACCESS_COARSE_LOCATION"
        | "android.permission.READ_CONTACTS"
        | "android.permission.WRITE_CONTACTS"
        | "android.permission.READ_CALENDAR"
        | "android.permission.WRITE_CALENDAR"
        | "android.permission.READ_EXTERNAL_STORAGE"
        | "android.permission.WRITE_EXTERNAL_STORAGE"
        | "android.permission.READ_MEDIA_IMAGES"
        | "android.permission.READ_MEDIA_VIDEO"
        | "android.permission.READ_MEDIA_AUDIO"
        | "android.permission.POST_NOTIFICATIONS" => "sensitive",
        "android.permission.RECEIVE_BOOT_COMPLETED"
        | "android.permission.REQUEST_IGNORE_BATTERY_OPTIMIZATIONS"
        | "android.permission.FOREGROUND_SERVICE" => "background",
        "android.permission.INTERNET"
        | "android.permission.ACCESS_NETWORK_STATE"
        | "android.permission.VIBRATE"
        | "android.permission.WAKE_LOCK" => "normal",
        permission if permission.starts_with("android.permission.") => "normal",
        _ => "unknown",
    }
}

fn push_json_field(json: &mut String, name: &str, value: Option<&str>, comma: bool) {
    if comma {
        json.push(',');
    }
    json.push('"');
    json.push_str(name);
    json.push_str("\":");
    match value {
        Some(value) => {
            json.push('"');
            json.push_str(&escape_json(value));
            json.push('"');
        }
        None => json.push_str("null"),
    }
}

pub fn escape_json(value: &str) -> String {
    let mut escaped = String::new();

    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            character if character.is_control() => {
                escaped.push_str(&format!("\\u{:04x}", character as u32));
            }
            character => escaped.push(character),
        }
    }

    escaped
}

fn invalid_data(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
