use std::str::FromStr;

use http::Uri;
use regex::Regex;
use serde::{Deserialize, Serialize};
use vetis::{
    Request, VetisResult,
    errors::VetisError,
    module::{
        AccessModule, Module, ModuleConfig, ModulePhase, PostContentModule, PreProcessModule,
        RewriteModule,
    },
};

/// A builder for creating a `RewriteModuleConfig`.
pub struct ModuleConfigBuilder {
    pattern: String,
    replacement: String,
}

impl ModuleConfigBuilder {
    /// Allow set a pattern
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn pattern(mut self, pattern: &str) -> Self {
        self.pattern = pattern.into();
        self
    }

    /// Allow set a replacement
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn replacement(mut self, replacement: &str) -> Self {
        self.replacement = replacement.into();
        self
    }

    /// Build the `RegexModuleConfig` with the configured settings.
    ///
    /// # Returns
    ///
    /// * `VetisResult<RegexModuleConfig>` - The `RegexModuleConfig` with the configured settings.
    pub fn build(self) -> VetisResult<RegexModuleConfig> {
        Ok(RegexModuleConfig { pattern: self.pattern, replacement: self.replacement })
    }
}

#[derive(Clone, Serialize, Deserialize)]
/// A struct with basic authentication configuration.
///
/// # Fields
///
/// * `users` - A map of username to hashed password.
/// * `algorithm` - The algorithm used for password hashing.
/// * `htpasswd` - The path to the htpasswd file.
///
/// # Examples
///
/// ```rust,no_run
/// use std::collections::HashMap;
/// use vetis_rewrite::{RewriteMode, RegexRewriteConfig};
///
/// let auth = RegexRewriteConfig::builder()
///     .mode(RewriteModel::Regex)
///     .build();
/// ```
pub struct RegexModuleConfig {
    pattern: String,
    replacement: String,
}

#[typetag::serde(name = "regex_rewrite")]
impl ModuleConfig for RegexModuleConfig {
    fn boxed_clone(&self) -> Box<dyn ModuleConfig> {
        Box::new(self.clone())
    }

    fn boxed_module(&self) -> Box<dyn Module> {
        Box::new(RegexModule::new(self.clone()))
    }

    fn boxed_sync(&self) -> Box<dyn Module + Send + Sync> {
        Box::new(RegexModule::new(self.clone()))
    }
}

impl RegexModuleConfig {
    /// Creates a new `ModuleConfigBuilder` with default settings.
    ///
    /// # Returns
    ///
    /// * `ModuleConfigBuilder` - The builder.
    pub fn builder() -> ModuleConfigBuilder {
        ModuleConfigBuilder {
            pattern: "/api/([a-z]*)".into(),
            replacement: "/api.php?action=$1".into(),
        }
    }

    /// Return match pattern
    ///
    /// # Returns
    ///
    /// * `&String` - Rewrite mode.
    pub fn pattern(&self) -> &str {
        &self.pattern
    }

    /// Return replacement expression
    ///
    /// # Returns
    ///
    /// * `&String` - Rewrite mode.
    pub fn replacement(&self) -> &str {
        &self.replacement
    }
}

/// RegexModule allow rewrite uri using regex
pub struct RegexModule {
    pattern: Regex,
    replacement: String,
}

impl RegexModule {
    /// Build a new RegexModule out of its configuration
    pub fn new(config: RegexModuleConfig) -> Self {
        let pattern = Regex::new(&config.pattern).expect("Invalid regex pattern");
        Self { pattern: pattern, replacement: config.replacement }
    }
}

impl Module for RegexModule {
    fn is_match(&self, _path: &str) -> bool {
        true
    }

    fn phase(&self) -> &ModulePhase {
        &ModulePhase::Rewrite
    }

    fn as_preprocess_module(&self) -> Option<&dyn PreProcessModule> {
        None
    }

    fn as_rewrite_module(&self) -> Option<&dyn RewriteModule> {
        Some(self)
    }

    fn as_access_module(&self) -> Option<&dyn AccessModule> {
        None
    }

    fn as_postcontent_module(&self) -> Option<&dyn PostContentModule> {
        None
    }
}

impl RewriteModule for RegexModule {
    fn rewrite(&self, mut request: Request) -> VetisResult<Request> {
        let source_uri = request
            .uri()
            .clone();

        let target_uri = self
            .replacement
            .clone();

        let replaced_uri = self
            .pattern
            .replace_all(&source_uri.to_string(), target_uri)
            .to_string();

        *request.uri_mut() =
            Uri::from_str(&replaced_uri).map_err(|e| VetisError::Module(e.to_string()))?;

        Ok(request)
    }
}
