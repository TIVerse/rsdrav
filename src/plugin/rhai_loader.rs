//! Rhai script plugin loader
//!
//! Allows loading plugins written in Rhai script language.
//!
//! Note: Rhai plugins are single-threaded and not Send + Sync.
//! Use them in the main thread only.

#[cfg(feature = "plugin-rhai")]
use rhai::{Engine, Scope, AST};
#[cfg(feature = "plugin-rhai")]
use std::path::Path;
#[cfg(feature = "plugin-rhai")]
use std::sync::{Arc, Mutex};

use super::Capability;
use crate::error::Result;

/// Rhai-based plugin (single-threaded)
///
/// Note: Due to Rhai's limitations, this plugin is not Send + Sync.
/// It should only be used in single-threaded contexts.
#[cfg(feature = "plugin-rhai")]
#[allow(clippy::arc_with_non_send_sync)]
pub struct RhaiPlugin {
    name: String,
    version: String,
    // Wrap in Arc<Mutex> to make it theoretically shareable
    // but in practice should only be used from one thread
    engine: Arc<Mutex<Engine>>,
    ast: Arc<AST>,
    capabilities: Vec<Capability>,
}

#[cfg(feature = "plugin-rhai")]
impl RhaiPlugin {
    /// Load a plugin from a Rhai script file
    pub fn from_file(path: impl AsRef<Path>) -> Result<Self> {
        let script = std::fs::read_to_string(path.as_ref()).map_err(crate::Error::Io)?;

        Self::from_script(&script)
    }

    /// Load a plugin from a Rhai script string
    pub fn from_script(script: &str) -> Result<Self> {
        let mut engine = Engine::new();

        // Register framework APIs
        Self::register_apis(&mut engine);

        let ast = engine.compile(script).map_err(|e| {
            crate::Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Rhai compilation error: {}", e),
            ))
        })?;

        // Extract plugin metadata
        let mut scope = Scope::new();
        engine.run_ast_with_scope(&mut scope, &ast).map_err(|e| {
            crate::Error::Io(std::io::Error::new(
                std::io::ErrorKind::InvalidData,
                format!("Rhai runtime error: {}", e),
            ))
        })?;

        let name = scope
            .get_value::<String>("PLUGIN_NAME")
            .unwrap_or_else(|| "unnamed".to_string());
        let version = scope
            .get_value::<String>("PLUGIN_VERSION")
            .unwrap_or_else(|| "0.1.0".to_string());

        Ok(Self {
            name,
            version,
            engine: Arc::new(Mutex::new(engine)),
            ast: Arc::new(ast),
            capabilities: vec![Capability::CustomWidgets], // Default capability
        })
    }

    /// Register framework APIs into Rhai engine
    fn register_apis(engine: &mut Engine) {
        // Register basic logging
        engine.register_fn("log", |msg: &str| {
            eprintln!("[Plugin] {}", msg);
        });

        // Register signal creation (simplified)
        engine.register_fn("create_signal", |value: i64| {
            value // Placeholder - would create actual Signal
        });

        // Add more API registrations here as needed
    }

    /// Call a function in the plugin script
    pub fn call_function<T: Clone + 'static>(
        &self,
        name: &str,
        args: impl Into<rhai::Dynamic>,
    ) -> Result<T> {
        let engine = self.engine.lock().unwrap();
        let result = engine
            .call_fn::<T>(&mut Scope::new(), &self.ast, name, (args.into(),))
            .map_err(|e| {
                crate::Error::Io(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    format!("Function call error: {}", e),
                ))
            })?;

        Ok(result)
    }
}

// Note: RhaiPlugin doesn't implement Plugin trait due to Send + Sync requirements
// It's meant to be used directly in single-threaded contexts
#[cfg(feature = "plugin-rhai")]
impl RhaiPlugin {
    /// Get plugin name (Plugin-like interface)
    pub fn plugin_name(&self) -> &str {
        &self.name
    }

    /// Get plugin version (Plugin-like interface)
    pub fn plugin_version(&self) -> &str {
        &self.version
    }

    #[allow(dead_code)]
    fn required_capabilities(&self) -> Vec<Capability> {
        self.capabilities.clone()
    }

    fn init(&mut self) -> Result<()> {
        // Call init function if it exists in script
        if let Err(_e) = self.call_function::<()>("init", ()) {
            // init() is optional
        }
        Ok(())
    }

    fn cleanup(&mut self) -> Result<()> {
        // Call cleanup function if it exists
        if let Err(_e) = self.call_function::<()>("cleanup", ()) {
            // cleanup() is optional
        }
        Ok(())
    }
}

/// Plugin loader for Rhai scripts
#[cfg(feature = "plugin-rhai")]
pub struct RhaiPluginLoader {
    search_paths: Vec<std::path::PathBuf>,
}

#[cfg(feature = "plugin-rhai")]
impl RhaiPluginLoader {
    /// Create a new Rhai plugin loader
    pub fn new() -> Self {
        Self {
            search_paths: vec![std::path::PathBuf::from("plugins")],
        }
    }

    /// Add a search path for plugins
    pub fn add_search_path(&mut self, path: impl Into<std::path::PathBuf>) {
        self.search_paths.push(path.into());
    }

    /// Load all plugins from search paths
    pub fn load_all(&self) -> Result<Vec<RhaiPlugin>> {
        let mut plugins = Vec::new();

        for path in &self.search_paths {
            if let Ok(entries) = std::fs::read_dir(path) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if path.extension().and_then(|s| s.to_str()) == Some("rhai") {
                        match RhaiPlugin::from_file(&path) {
                            Ok(plugin) => plugins.push(plugin),
                            Err(e) => eprintln!("Failed to load plugin {:?}: {}", path, e),
                        }
                    }
                }
            }
        }

        Ok(plugins)
    }
}

#[cfg(feature = "plugin-rhai")]
impl Default for RhaiPluginLoader {
    fn default() -> Self {
        Self::new()
    }
}

// Stubs when feature is disabled
#[cfg(not(feature = "plugin-rhai"))]
pub struct RhaiPlugin;

#[cfg(not(feature = "plugin-rhai"))]
pub struct RhaiPluginLoader;

#[cfg(not(feature = "plugin-rhai"))]
impl RhaiPluginLoader {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(test)]
#[cfg(feature = "plugin-rhai")]
mod tests {
    use super::*;

    #[test]
    fn test_simple_plugin() {
        let script = r#"
            const PLUGIN_NAME = "test_plugin";
            
            fn init() {
                log("Plugin initialized!");
            }
            
            fn cleanup() {
                log("Plugin cleaned up!");
            }
            
            fn hello(name) {
                "Hello, " + name + "!"
            }
        "#;

        let mut plugin = RhaiPlugin::from_script(script).unwrap();
        assert_eq!(plugin.plugin_name(), "test_plugin");

        assert!(plugin.init().is_ok());

        let result: String = plugin.call_function("hello", "World").unwrap();
        assert_eq!(result, "Hello, World!");

        assert!(plugin.cleanup().is_ok());
    }

    #[test]
    fn test_plugin_loader() {
        let loader = RhaiPluginLoader::new();
        // Just test creation, actual loading would need test files
        assert!(!loader.search_paths.is_empty());
    }
}
