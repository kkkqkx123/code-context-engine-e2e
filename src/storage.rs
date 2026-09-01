//! Storage backend configuration for E2E tests

/// Storage backend configuration
#[derive(Debug, Clone)]
pub struct StorageConfig {
    /// Use in-memory storage
    pub in_memory: bool,
    /// Qdrant endpoint (None to skip)
    pub qdrant_endpoint: Option<String>,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self::in_memory()
    }
}

impl StorageConfig {
    /// Create in-memory storage configuration
    pub fn in_memory() -> Self {
        Self {
            in_memory: true,
            qdrant_endpoint: None,
        }
    }

    /// Add Qdrant endpoint
    pub fn with_qdrant(mut self, endpoint: impl Into<String>) -> Self {
        self.qdrant_endpoint = Some(endpoint.into());
        self
    }

    /// Check if Qdrant is configured
    pub fn has_qdrant(&self) -> bool {
        self.qdrant_endpoint.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_in_memory_config() {
        let config = StorageConfig::in_memory();
        assert!(config.in_memory);
        assert!(!config.has_qdrant());
    }

    #[test]
    fn test_with_qdrant() {
        let config = StorageConfig::in_memory().with_qdrant("http://localhost:6333");
        assert!(config.has_qdrant());
    }
}
