use cel::Context;
use compact_str::ToCompactString;
use http::Uri;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use vetis::{
    Request, Response, Str, VetisFutureResult, VetisHook, VetisResult,
    errors::VetisError,
    hook::{Hook, HookPhase},
    matcher::{Matcher, MatcherConfig},
};

/// A builder for creating a `RewriteHookConfig`.
pub struct HookConfigBuilder {
    matcher: Option<Box<dyn MatcherConfig>>,
    target: Str,
    replacement: Str,
}

impl HookConfigBuilder {
    /// Allow set a matcher
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn add_matcher<M>(mut self, matcher: M) -> Self
    where
        M: MatcherConfig + Send + Sync + 'static,
    {
        self.matcher = Some(Box::new(matcher));
        self
    }

    /// Allow set a target
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn target(mut self, target: &str) -> Self {
        self.target = target.to_compact_string();
        self
    }

    /// Allow set a replacement
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn replacement(mut self, replacement: &str) -> Self {
        self.replacement = replacement.to_compact_string();
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
            target: self.target,
            replacement: self.replacement,
        })
    }
}

#[derive(Clone, Serialize, Deserialize)]
/// A struct with regex rewrite configuration.
///
/// # Fields
///
/// * `target` - A regex pattern to match uri against to.
/// * `replacement` - The uri replacement.
///
/// # Examples
///
/// ```rust,no_run
/// use std::collections::HashMap;
/// use vetis_rewrite::{regex::HookConfig};
///
/// let auth = HookConfig::builder()
///     .target(r"/(hello*)/(world*)")
///     .replacement("/$2/$1")
///     .build();
/// ```
pub struct HookConfig {
    matcher: Option<Box<dyn MatcherConfig>>,
    target: Str,
    replacement: Str,
}

#[typetag::serde(name = "regex_rewrite")]
impl vetis::hook::HookConfig for HookConfig {
    fn matcher(&self) -> Option<&Box<dyn MatcherConfig>> {
        self.matcher
            .as_ref()
    }

    fn boxed_clone(&self) -> Box<dyn vetis::hook::HookConfig> {
        Box::new(self.clone())
    }

    fn boxed_hook(&self) -> Box<dyn VetisHook> {
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
    matcher: Option<Box<dyn Matcher>>,
    target: Regex,
    replacement: Str,
}

unsafe impl Send for RegexHook {}
unsafe impl Sync for RegexHook {}

impl RegexHook {
    /// Build a new RegexHook out of its configuration
    pub fn new(config: HookConfig) -> Self {
        let target = Regex::new(&config.target).expect("Invalid target regex pattern");
        let matcher = if let Some(matcher_config) = config.matcher {
            Some(matcher_config.boxed_matcher())
        } else {
            None
        };
        Self { matcher, target, replacement: config.replacement }
    }
}

impl Hook for RegexHook {
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
        mut request: Request,
        response: Option<Response>,
    ) -> VetisFutureResult<'a, (Request, Option<Response>)> {
        let future = async move {
            let source_uri = request.uri_mut();

            let target_uri = self
                .replacement
                .clone();

            let replaced_uri = self
                .target
                .replace_all(&source_uri.to_string(), target_uri.to_string())
                .to_string();

            *source_uri = Uri::from_str(&replaced_uri)
                .map_err(|e| VetisError::hook(e.to_compact_string()))?;

            Ok((request, response))
        };

        Box::pin(future)
    }
}
