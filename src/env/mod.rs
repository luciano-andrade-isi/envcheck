use std::collections::BTreeMap;
use std::path::PathBuf;

pub(crate) mod parser;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EnvEntry {
    pub(crate) key: String,
    pub(crate) value: String,
    pub(crate) line: usize,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub(crate) struct EnvDocument {
    pub(crate) path: PathBuf,
    pub(crate) entries: Vec<EnvEntry>,
    pub(crate) key_index: BTreeMap<String, Vec<usize>>,
}

impl EnvDocument {
    pub(crate) fn new(path: PathBuf, entries: Vec<EnvEntry>) -> Self {
        let mut key_index = BTreeMap::<String, Vec<usize>>::new();
        for (index, entry) in entries.iter().enumerate() {
            key_index.entry(entry.key.clone()).or_default().push(index);
        }

        Self {
            path,
            entries,
            key_index,
        }
    }
}
