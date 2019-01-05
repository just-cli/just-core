use serde::{Deserialize, Serialize};
use std::path::Path;

pub fn read_toml<'de, T: Default + Deserialize<'de>>(path: &Path, buf: &'de mut String) -> T {
    use std::fs::File;
    use std::io::Read;

    buf.clear();

    if let Ok(mut file) = File::open(path) {
        file.read_to_string(buf)
            .unwrap_or_else(|e| panic!("Could not read file {:?}: {:?}", path, e));
    }

    if buf.is_empty() {
        T::default()
    } else {
        toml::from_str(buf)
            .unwrap_or_else(|e| panic!("Failed to interpret file {:?} as toml: {:?}", path, e))
    }
}

pub fn write_toml<T: Serialize>(path: &Path, data: &T) {
    use std::fs::File;
    use std::io::Write;

    let json = toml::to_string_pretty(data).unwrap_or_else(|e| {
        panic!(
            "Could not convert toml-data to string ({:?}): {:?}",
            path, e
        )
    });

    File::create(path)
        .or_else(|e| panic!("Could not create toml-file {:?}: {:?}", path, e))
        .and_then(|mut file| file.write_all(json.as_bytes()))
        .unwrap_or_else(|e| panic!("Could not write toml-data into file {:?}: {:?}", path, e));
}
