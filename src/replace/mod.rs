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

/// A builder for creating a `HookConfig`.
pub struct HookConfigBuilder {
    matcher: Option<String>,
    target: String,
    replacement: String,
    limit: Option<usize>,
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

    /// Allow set a limit
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = Some(limit);
        self
    }

    /// Build the `HookConfig` with the configured settings.
    ///
    /// # Returns
    ///
    /// * `VetisResult<HookConfig>` - The `HookConfig` with the configured settings.
    pub fn build(self) -> VetisResult<HookConfig> {
        Ok(HookConfig {
            matcher: self.matcher,
            target: self.target,
            replacement: self.replacement,
            limit: self.limit,
        })
    }
}

#[derive(Clone, Serialize, Deserialize)]
/// A struct with regex rewrite configuration.
///
/// # Fields
///
/// * `pattern` - Target uri to regex to matchw with.
/// * `replacement` - The replacement uri.
///
/// # Examples
///
/// ```rust,no_run
/// use std::collections::HashMap;
/// use vetis_rewrite::replace::HookConfig;
///
/// let auth = HookConfig::builder()
///     .mode(RewriteModel::Regex)
///     .build();
/// ```
pub struct HookConfig {
    matcher: Option<String>,
    target: String,
    replacement: String,
    limit: Option<usize>,
}

#[typetag::serde(name = "path_replace")]
impl vetis::hook::HookConfig for HookConfig {
    fn matcher(&self) -> Option<&String> {
        self.matcher
            .as_ref()
    }

    fn boxed_clone(&self) -> Box<dyn vetis::hook::HookConfig> {
        Box::new(self.clone())
    }

    fn boxed_hook(&self) -> Box<dyn Hook> {
        Box::new(PathReplaceHook::new(self.clone()))
    }

    fn boxed_sync(&self) -> Box<dyn Hook + Send + Sync> {
        Box::new(PathReplaceHook::new(self.clone()))
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
            target: "/service".into(),
            replacement: "/api".into(),
            limit: None,
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

    /// Return replacement limit
    ///
    /// # Returns
    ///
    /// * `u8` - Rewrite mode.
    pub fn limit(&self) -> Option<usize> {
        self.limit
    }
}

/// PathReplaceHook allow replace uri
pub struct PathReplaceHook {
    matcher: Option<Regex>,
    target: Regex,
    replacement: String,
    limit: usize,
}

impl PathReplaceHook {
    /// Build a new RegexHook out of its configuration
    pub fn new(config: HookConfig) -> Self {
        let matcher = if let Some(matcher) = config.matcher {
            Some(Regex::new(&matcher).expect("Invalid matcher regex pattern"))
        } else {
            None
        };
        let target = Regex::new(&config.target).expect("Invalid target regex pattern");
        Self {
            matcher,
            target,
            replacement: config.replacement,
            limit: config
                .limit
                .unwrap_or(1 as usize),
        }
    }
}

impl Hook for PathReplaceHook {
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

impl RewriteHook for PathReplaceHook {
    fn rewrite(&self, mut request: Request) -> VetisResult<Request> {
        let source_uri = request
            .uri()
            .clone();

        let replacement_uri = self
            .replacement
            .clone();

        let replaced_uri = self
            .target
            .replacen(&source_uri.to_string(), self.limit, replacement_uri)
            .to_string();

        *request.uri_mut() =
            Uri::from_str(&replaced_uri).map_err(|e| VetisError::Hook(e.to_string()))?;

        Ok(request)
    }
}
