//! Plugin system for extending rsdrav functionality
//!
//! Provides a capability-based plugin architecture for safe third-party extensions.

#[cfg(feature = "plugin-rhai")]
pub mod rhai_loader;

#[cfg(feature = "plugin-rhai")]
pub use rhai_loader::{RhaiPlugin, RhaiPluginLoader};

#[cfg(feature = "plugin-dylib")]
pub mod dylib_loader;

#[cfg(feature = "plugin-dylib")]
pub use dylib_loader::DylibPluginLoader;

#[cfg(feature = "plugin-wasm")]
pub mod wasm_loader;

#[cfg(feature = "plugin-wasm")]
pub use wasm_loader::{WasmPlugin, WasmPluginLoader};

use crate::error::Result;
use std::collections::HashMap;

/// Plugin trait that all plugins must implement
pub trait Plugin: Send + Sync {
    /// Plugin name
    fn name(&self) -> &str;

    /// Plugin version
    fn version(&self) -> &str;

    /// Required capabilities
    fn required_capabilities(&self) -> Vec<Capability>;

    /// Initialize the plugin
    fn init(&mut self) -> Result<()>;

    /// Cleanup on plugin unload
    fn cleanup(&mut self) -> Result<()>;
}

/// Capabilities that plugins can request
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Capability {
    /// Read from filesystem
    FileRead,
    /// Write to filesystem
    FileWrite,
    /// Make network requests
    Network,
    /// Execute commands
    Execute,
    /// Access environment variables
    Environment,
    /// Register custom widgets
    CustomWidgets,
    /// Register commands
    RegisterCommands,
    /// Access application state
    StateAccess,
}

/// Plugin manager for loading and managing plugins
pub struct PluginManager {
    plugins: HashMap<String, Box<dyn Plugin>>,
    capabilities: HashMap<String, Vec<Capability>>,
}

impl PluginManager {
    /// Create a new plugin manager
    pub fn new() -> Self {
        Self {
            plugins: HashMap::new(),
            capabilities: HashMap::new(),
        }
    }

    /// Register a plugin
    pub fn register(&mut self, plugin: Box<dyn Plugin>) -> Result<()> {
        let name = plugin.name().to_string();
        let caps = plugin.required_capabilities();

        // Validate capabilities
        for cap in &caps {
            if !self.is_capability_allowed(cap) {
                return Err(crate::Error::Plugin(
                    crate::error::PluginError::CapabilityDenied(format!(
                        "Capability {:?} not allowed",
                        cap
                    )),
                ));
            }
        }

        self.capabilities.insert(name.clone(), caps);
        self.plugins.insert(name, plugin);

        Ok(())
    }

    /// Initialize all plugins
    pub fn init_all(&mut self) -> Result<()> {
        for plugin in self.plugins.values_mut() {
            plugin.init()?;
        }
        Ok(())
    }

    /// Cleanup all plugins
    pub fn cleanup_all(&mut self) -> Result<()> {
        for plugin in self.plugins.values_mut() {
            plugin.cleanup()?;
        }
        Ok(())
    }

    /// Get a plugin by name
    pub fn get(&self, name: &str) -> Option<&dyn Plugin> {
        self.plugins.get(name).map(|p| p.as_ref())
    }

    /// Check if a capability is allowed (placeholder for security policy)
    fn is_capability_allowed(&self, cap: &Capability) -> bool {
        // In a real implementation, this would check against a security policy
        // For now, allow all capabilities
        match cap {
            Capability::Execute | Capability::FileWrite => false, // Unsafe by default
            _ => true,
        }
    }

    /// List all registered plugins
    pub fn list_plugins(&self) -> Vec<&str> {
        self.plugins.keys().map(|s| s.as_str()).collect()
    }
}

impl Default for PluginManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Example plugin implementation
pub struct ExamplePlugin {
    name: String,
    initialized: bool,
}

impl ExamplePlugin {
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            initialized: false,
        }
    }
}

impl Plugin for ExamplePlugin {
    fn name(&self) -> &str {
        &self.name
    }

    fn version(&self) -> &str {
        "0.1.0"
    }

    fn required_capabilities(&self) -> Vec<Capability> {
        vec![Capability::CustomWidgets]
    }

    fn init(&mut self) -> Result<()> {
        self.initialized = true;
        Ok(())
    }

    fn cleanup(&mut self) -> Result<()> {
        self.initialized = false;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_manager_creation() {
        let manager = PluginManager::new();
        assert_eq!(manager.list_plugins().len(), 0);
    }

    #[test]
    fn test_plugin_registration() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(ExamplePlugin::new("test"));

        let result = manager.register(plugin);
        assert!(result.is_ok());
        assert_eq!(manager.list_plugins().len(), 1);
    }

    #[test]
    fn test_plugin_init() {
        let mut manager = PluginManager::new();
        let plugin = Box::new(ExamplePlugin::new("test"));

        manager.register(plugin).unwrap();
        let result = manager.init_all();
        assert!(result.is_ok());
    }
}
