use kiwi_plugin_api::StaticStr;

use crate::error::LoadError;

pub(crate) fn decode_static_str(value: StaticStr) -> Result<String, LoadError> {
    if value.ptr.is_null() {
        return Err(LoadError::InvalidDescriptor("null static string"));
    }
    // SAFETY: Plugin descriptors are built from valid UTF-8 `&'static str` in plugin code.
    let bytes = unsafe { std::slice::from_raw_parts(value.ptr, value.len) };
    std::str::from_utf8(bytes)
        .map(str::to_string)
        .map_err(|_| LoadError::InvalidDescriptor("invalid UTF-8"))
}
