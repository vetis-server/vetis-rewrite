use std::str::FromStr;

use http::Uri;
use regex::Regex;
use serde::{Deserialize, Serialize};
use vetis::{
    Request, VetisResult,
    errors::VetisError,
    hook::{
        AccessHook, ContentHook, Hook, HookPhase, PostContentHook, PreProcessHook, RewriteHook,
    },
};

/// A builder for creating a `RewriteHookConfig`.
pub struct HookConfigBuilder {
    matcher: Option<String>,
    target: String,
    replacement: String,
}

impl HookConfigBuilder {
    /// Allow set a matcher
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn matcher(mut self, matcher: &str) -> Self {
        self.matcher = Some(matcher.into());
        self
    }

    /// Allow set a target
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn target(mut self, target: &str) -> Self {
        self.target = target.into();
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

    /// Build the `HookConfig` with the configured settings.
    ///
    /// # Returns
    ///
    /// * `VetisResult<HookConfig>` - The `HookConfig` with the configured settings.
    pub fn build(self) -> VetisResult<HookConfig> {
        Ok(HookConfig { matcher: self.matcher, target: self.target, replacement: self.replacement })
    }
}

#[derive(Clone, Serialize, Deserialize)]
/// A struct with regex rewrite configuration.
///
/// # Fields
///
/// * `pattern` - A regex pattern to match uri against to.
/// * `replacement` - The uri replacement.
///
/// # Examples
///
/// ```rust,no_run
/// use std::collections::HashMap;
/// use vetis_rewrite::{regex::HookConfig;
///
/// let auth = HookConfig::builder()
///     .mode(RewriteModel::Regex)
///     .build();
/// ```
pub struct HookConfig {
    matcher: Option<String>,
    target: String,
    replacement: String,
}

#[typetag::serde(name = "regex_rewrite")]
impl vetis::hook::HookConfig for HookConfig {
    fn matcher(&self) -> Option<&String> {
        self.matcher
            .as_ref()
    }

    fn boxed_clone(&self) -> Box<dyn vetis::hook::HookConfig> {
        Box::new(self.clone())
    }

    fn boxed_hook(&self) -> Box<dyn Hook> {
        Box::new(RegexHook::new(self.clone()))
    }

    fn boxed_sync(&self) -> Box<dyn Hook + Send + Sync> {
        Box::new(RegexHook::new(self.clone()))
    }
}

impl HookConfig {
    /// Creates a new `HookConfigBuilder` with default settings.
    ///
    /// # Returns
    ///
    /// * `HookConfigBuilder` - The builder.
    pub fn builder() -> HookConfigBuilder {
        HookConfigBuilder {
            matcher: None,
            target: "/api".into(),
            replacement: "/api.php?action=$1".into(),
        }
    }

    /// Return match pattern
    ///
    /// # Returns
    ///
    /// * `&String` - Rewrite mode.
    pub fn target(&self) -> &str {
        &self.target
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

/// RegexHook allow rewrite uri using regex
pub struct RegexHook {
    matcher: Option<Regex>,
    target: Regex,
    replacement: String,
}

impl RegexHook {
    /// Build a new RegexHook out of its configuration
    pub fn new(config: HookConfig) -> Self {
        let matcher = if let Some(matcher) = config.matcher {
            Some(Regex::new(&matcher).expect("Invalid matcher regex pattern"))
        } else {
            None
        };
        let target = Regex::new(&config.target).expect("Invalid target regex pattern");
        Self { matcher, target, replacement: config.replacement }
    }
}

impl Hook for RegexHook {
    fn is_match(&self, path: &str) -> bool {
        if let Some(regex) = &self.matcher { regex.is_match(path) } else { true }
    }

    fn phase(&self) -> &HookPhase {
        &HookPhase::Rewrite
    }

    fn as_preprocess_hook(&self) -> Option<&dyn PreProcessHook> {
        None
    }

    fn as_rewrite_hook(&self) -> Option<&dyn RewriteHook> {
        Some(self)
    }

    fn as_access_hook(&self) -> Option<&dyn AccessHook> {
        None
    }

    fn as_content_hook(&self) -> Option<&dyn ContentHook> {
        None
    }

    fn as_postcontent_hook(&self) -> Option<&dyn PostContentHook> {
        None
    }
}

impl RewriteHook for RegexHook {
    fn rewrite(&self, mut request: Request) -> VetisResult<Request> {
        let source_uri = request
            .uri()
            .clone();

        let target_uri = self
            .replacement
            .clone();

        let replaced_uri = self
            .target
            .replace_all(&source_uri.to_string(), target_uri)
            .to_string();

        *request.uri_mut() =
            Uri::from_str(&replaced_uri).map_err(|e| VetisError::Hook(e.to_string()))?;

        Ok(request)
    }
}
