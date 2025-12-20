//! Plugin registration and initialization

use tracing::instrument;

#[cfg(feature = "plugins")]
use form_factor_plugins::PluginManager;

#[cfg(all(feature = "plugins", feature = "plugin-canvas"))]
use form_factor_plugins::CanvasPlugin;

#[cfg(all(feature = "plugins", feature = "plugin-layers"))]
use form_factor_plugins::LayersPlugin;

#[cfg(all(feature = "plugins", feature = "plugin-file"))]
use form_factor_plugins::FilePlugin;

#[cfg(all(feature = "plugins", feature = "plugin-detection"))]
use form_factor_plugins::DetectionPlugin;

#[cfg(all(feature = "plugins", feature = "plugin-properties"))]
use form_factor_plugins::PropertiesPlugin;

/// Plugin setup utilities
pub struct PluginSetup;

impl PluginSetup {
    /// Create and initialize plugin manager with all enabled plugins
    #[cfg(feature = "plugins")]
    #[instrument]
    pub fn create_manager() -> PluginManager {
        tracing::info!("Creating plugin manager");
        let mut manager = PluginManager::new();
        let mut count = 0;

        #[cfg(feature = "plugin-canvas")]
        {
            Self::register_canvas_plugin(&mut manager);
            count += 1;
        }

        #[cfg(feature = "plugin-layers")]
        {
            Self::register_layers_plugin(&mut manager);
            count += 1;
        }

        #[cfg(feature = "plugin-file")]
        {
            Self::register_file_plugin(&mut manager);
            count += 1;
        }

        #[cfg(feature = "plugin-detection")]
        {
            Self::register_detection_plugin(&mut manager);
            count += 1;
        }

        #[cfg(feature = "plugin-properties")]
        {
            Self::register_properties_plugin(&mut manager);
            count += 1;
        }

        // Register templates plugin (always enabled)
        Self::register_templates_plugin(&mut manager);
        count += 1;

        tracing::info!(count, "Plugin manager created with {} plugin(s)", count);
        manager
    }

    #[cfg(all(feature = "plugins", feature = "plugin-canvas"))]
    #[instrument(skip(manager))]
    fn register_canvas_plugin(manager: &mut PluginManager) {
        manager.register(Box::new(CanvasPlugin::new()));
        tracing::info!("Registered canvas plugin");
    }

    #[cfg(all(feature = "plugins", feature = "plugin-layers"))]
    #[instrument(skip(manager))]
    fn register_layers_plugin(manager: &mut PluginManager) {
        manager.register(Box::new(LayersPlugin::new()));
        tracing::info!("Registered layers plugin");
    }

    #[cfg(all(feature = "plugins", feature = "plugin-file"))]
    #[instrument(skip(manager))]
    fn register_file_plugin(manager: &mut PluginManager) {
        manager.register(Box::new(FilePlugin::new()));
        tracing::info!("Registered file plugin");
    }

    #[cfg(all(feature = "plugins", feature = "plugin-detection"))]
    #[instrument(skip(manager))]
    fn register_detection_plugin(manager: &mut PluginManager) {
        manager.register(Box::new(DetectionPlugin::new()));
        tracing::info!("Registered detection plugin");
    }

    #[cfg(all(feature = "plugins", feature = "plugin-properties"))]
    #[instrument(skip(manager))]
    fn register_properties_plugin(manager: &mut PluginManager) {
        manager.register(Box::new(PropertiesPlugin::new()));
        tracing::info!("Registered properties plugin");
    }

    #[cfg(feature = "plugins")]
    #[instrument(skip(manager))]
    fn register_templates_plugin(manager: &mut PluginManager) {
        manager.register(Box::new(form_factor_plugins::TemplatesPlugin::new()));
        tracing::info!("Registered templates plugin");
    }
}
