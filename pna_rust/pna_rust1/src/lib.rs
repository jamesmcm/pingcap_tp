use std::collections::HashMap;

pub struct KvStore {
    map: HashMap<String, String>,
}

impl KvStore {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn get(&self, key: String) -> Option<String> {
        self.map.get(&key).map(|v| v.to_owned())
    }
    pub fn set(&mut self, key: String, val: String) {
        self.map
            .entry(key)
            .and_modify(|v| *v = val.clone())
            .or_insert(val);
    }
    pub fn remove(&mut self, key: String) {
        self.map.remove(&key);
    }
}

impl std::default::Default for KvStore {
    fn default() -> Self {
        KvStore {
            map: HashMap::new(),
        }
    }
}
