use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RegistryError {
    DuplicateId(String),
    MissingId(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Registry<T> {
    entries: BTreeMap<String, T>,
}

impl<T> Default for Registry<T> {
    fn default() -> Self {
        Self {
            entries: BTreeMap::new(),
        }
    }
}

impl<T> Registry<T> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn register(&mut self, id: impl Into<String>, value: T) -> Result<(), RegistryError> {
        let id = id.into();
        if self.entries.contains_key(&id) {
            return Err(RegistryError::DuplicateId(id));
        }

        self.entries.insert(id, value);
        Ok(())
    }

    pub fn get(&self, id: &str) -> Result<&T, RegistryError> {
        self.entries
            .get(id)
            .ok_or_else(|| RegistryError::MissingId(id.to_owned()))
    }

    pub fn available_ids(&self) -> Vec<&str> {
        self.entries.keys().map(String::as_str).collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn registers_and_resolves_adapter_by_id() {
        let mut registry = Registry::new();

        registry.register("npm", "npm-adapter").unwrap();

        assert_eq!(registry.get("npm"), Ok(&"npm-adapter"));
    }

    #[test]
    fn duplicate_ids_are_rejected() {
        let mut registry = Registry::new();

        registry.register("npm", "first").unwrap();

        assert_eq!(
            registry.register("npm", "second"),
            Err(RegistryError::DuplicateId("npm".to_owned()))
        );
        assert_eq!(registry.get("npm"), Ok(&"first"));
    }

    #[test]
    fn missing_ids_return_typed_error() {
        let registry = Registry::<&str>::new();

        assert_eq!(
            registry.get("pip"),
            Err(RegistryError::MissingId("pip".to_owned()))
        );
    }

    #[test]
    fn available_ids_are_deterministic() {
        let mut registry = Registry::new();

        registry.register("uv", "uv-adapter").unwrap();
        registry.register("npm", "npm-adapter").unwrap();

        assert_eq!(registry.available_ids(), vec!["npm", "uv"]);
    }
}
