use cel::Context;
use compact_str::ToCompactString;
use serde::{Deserialize, Serialize};
use vetis::{
    Request, Response, Str, VetisFutureResult, VetisHook, VetisResult,
    hook::{Hook, HookPhase},
    matcher::{Matcher, MatcherConfig},
};

/// A builder for creating a `RewriteModuleConfig`.
pub struct HookConfigBuilder<'a> {
    matcher: Option<Box<dyn MatcherConfig>>,
    pattern: &'a str,
    replacement: &'a str,
}

impl<'a> HookConfigBuilder<'a> {
    /// Allow set a pattern
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn pattern(mut self, pattern: &'a str) -> Self {
        self.pattern = pattern;
        self
    }

    /// Allow set a replacement
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn replacement(mut self, replacement: &'a str) -> Self {
        self.replacement = replacement;
        self
    }

    /// Build the `HookConfig` with the configured settings.
    ///
    /// # Returns
    ///
    /// * `VetisResult<HookConfig>` - The `HookConfig` with the configured settings.
    pub fn build(self) -> VetisResult<HookConfig> {
        Ok(HookConfig {
            matcher: self
                .matcher
                .map_or(None, |val| Some(val)),
            pattern: self
                .pattern
                .to_compact_string(),
            replacement: self
                .replacement
                .to_compact_string(),
        })
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
pub struct HookConfig {
    matcher: Option<Box<dyn MatcherConfig>>,
    pattern: Str,
    replacement: Str,
}

#[typetag::serde(name = "req_headers_rewrite")]
impl vetis::hook::HookConfig for HookConfig {
    fn matcher(&self) -> Option<&Box<dyn MatcherConfig>> {
        self.matcher
            .as_ref()
    }

    fn boxed_clone(&self) -> Box<dyn vetis::hook::HookConfig> {
        Box::new(self.clone())
    }

    fn boxed_hook(&self) -> Box<dyn VetisHook> {
        Box::new(ReqHeaderHook::new(self.clone()))
    }
}

impl HookConfig {
    /// Creates a new `HookConfigBuilder` with default settings.
    ///
    /// # Returns
    ///
    /// * `HookConfigBuilder` - The builder.
    pub fn builder<'a>() -> HookConfigBuilder<'a> {
        HookConfigBuilder {
            matcher: None,
            pattern: "/api/([a-z]*)",
            replacement: "/api.php?action=$1",
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

/// ReqHeaderHook allow rewrite uri using regex
pub struct ReqHeaderHook {
    matcher: Option<Box<dyn Matcher>>,
    #[allow(dead_code)]
    config: HookConfig,
}

unsafe impl Send for ReqHeaderHook {}
unsafe impl Sync for ReqHeaderHook {}

impl ReqHeaderHook {
    /// Build a new RegexHook out of its configuration
    pub fn new(config: HookConfig) -> Self {
        let matcher = if let Some(matcher_config) = &config.matcher {
            Some(matcher_config.boxed_matcher())
        } else {
            None
        };
        Self { matcher, config }
    }
}

impl Hook for ReqHeaderHook {
    fn has_matches(&self, context: &Context) -> bool {
        if let Some(matcher) = &self.matcher { matcher.is_match(context) } else { true }
    }

    fn phase(&self) -> &HookPhase {
        &HookPhase::Rewrite
    }

    fn matcher_mut(&mut self) -> Option<&mut Box<dyn Matcher>> {
        self.matcher
            .as_mut()
    }

    fn handle<'a>(
        &'a self,
        request: Request,
        response: Option<Response>,
    ) -> VetisFutureResult<'a, (Request, Option<Response>)> {
        let future = async move {
            let _source_uri = request
                .uri()
                .clone();

            Ok((request, response))
        };

        Box::pin(future)
    }
}
