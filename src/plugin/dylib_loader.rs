//! Dynamic library plugin loader
//!
//! Loads plugins from shared libraries (.so/.dll/.dylib)
//! Requires the `plugin-dylib` feature flag.

#[cfg(feature = "plugin-dylib")]
use libloading::{Library, Symbol};

use super::{Capability, Plugin};
use crate::error::{PluginError, Result};
use std::path::Path;

/// Type signature for plugin entry point
#[cfg(feature = "plugin-dylib")]
pub type PluginCreate = unsafe fn() -> *mut dyn Plugin;

/// Dynamic library plugin loader
#[cfg(feature = "plugin-dylib")]
pub struct DylibPluginLoader {
    _library: Library,
    plugin: Box<dyn Plugin>,
}

#[cfg(feature = "plugin-dylib")]
impl DylibPluginLoader {
    /// Load a plugin from a dynamic library
    ///
    /// # Safety
    /// This loads arbitrary native code from the filesystem.
    /// Only load plugins from trusted sources!
    pub unsafe fn load<P: AsRef<Path>>(path: P) -> Result<Self> {
        let library = Library::new(path.as_ref())
            .map_err(|e| crate::Error::Plugin(PluginError::LoadFailed(e.to_string())))?;

        // Look for the plugin creation function
        let create: Symbol<PluginCreate> = library.get(b"_plugin_create").map_err(|e| {
            crate::Error::Plugin(PluginError::LoadFailed(format!(
                "Failed to find _plugin_create symbol: {}",
                e
            )))
        })?;

        // Call the creation function
        let plugin_ptr = create();
        if plugin_ptr.is_null() {
            return Err(crate::Error::Plugin(PluginError::LoadFailed(
                "Plugin creation returned null".into(),
            )));
        }

        let plugin = Box::from_raw(plugin_ptr);

        Ok(Self {
            _library: library,
            plugin,
        })
    }

    /// Get a reference to the loaded plugin
    pub fn plugin(&self) -> &dyn Plugin {
        &*self.plugin
    }

    /// Get a mutable reference to the loaded plugin
    pub fn plugin_mut(&mut self) -> &mut dyn Plugin {
        &mut *self.plugin
    }
}

#[cfg(feature = "plugin-dylib")]
impl Plugin for DylibPluginLoader {
    fn name(&self) -> &str {
        self.plugin.name()
    }

    fn version(&self) -> &str {
        self.plugin.version()
    }

    fn required_capabilities(&self) -> Vec<Capability> {
        self.plugin.required_capabilities()
    }

    fn init(&mut self) -> Result<()> {
        self.plugin.init()
    }

    fn cleanup(&mut self) -> Result<()> {
        self.plugin.cleanup()
    }
}

// Stub for when feature is disabled
#[cfg(not(feature = "plugin-dylib"))]
pub struct DylibPluginLoader;

#[cfg(not(feature = "plugin-dylib"))]
impl DylibPluginLoader {
    pub unsafe fn load<P: AsRef<std::path::Path>>(_path: P) -> Result<Self> {
        Err(crate::Error::Plugin(PluginError::LoadFailed(
            "Dylib plugin support requires 'plugin-dylib' feature".into(),
        )))
    }
}

#[cfg(test)]
#[cfg(feature = "plugin-dylib")]
mod tests {
    use super::*;

    // Note: Real tests would require building a test plugin library
    // This is a placeholder for the testing structure

    #[test]
    fn test_dylib_loader_feature_enabled() {
        // Just verify the types exist when feature is enabled
        // Loader created successfully
    }
}
