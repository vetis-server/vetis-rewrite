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

/// A builder for creating a `HookConfig`.
pub struct HookConfigBuilder {
    matcher: Option<Box<dyn MatcherConfig>>,
    target: Str,
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
        self.target = target.into();
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
        })
    }
}

#[derive(Clone, Serialize, Deserialize)]
/// A struct with strip suffix rewrite configuration.
///
/// # Fields
///
/// * `pattern` - Target uri to regex to matchw with.
///
/// # Examples
///
/// ```rust,no_run
/// use std::collections::HashMap;
/// use vetis_rewrite::strip_suffix::HookConfig;
///
/// let auth = HookConfig::builder()
///     .target("/home")
///     .build();
/// ```
pub struct HookConfig {
    matcher: Option<Box<dyn MatcherConfig>>,
    target: Str,
}

#[typetag::serde(name = "strip_suffix")]
impl vetis::hook::HookConfig for HookConfig {
    fn matcher(&self) -> Option<&Box<dyn MatcherConfig>> {
        self.matcher
            .as_ref()
    }

    fn boxed_clone(&self) -> Box<dyn vetis::hook::HookConfig> {
        Box::new(self.clone())
    }

    fn boxed_hook(&self) -> Box<dyn VetisHook> {
        Box::new(StripSuffixHook::new(self.clone()))
    }
}

impl HookConfig {
    /// Creates a new `HookConfigBuilder` with default settings.
    ///
    /// # Returns
    ///
    /// * `HookConfigBuilder` - The builder.
    pub fn builder() -> HookConfigBuilder {
        HookConfigBuilder { matcher: None, target: "/api".into() }
    }

    /// Return match pattern
    ///
    /// # Returns
    ///
    /// * `&String` - Rewrite mode.
    pub fn target(&self) -> &str {
        &self.target
    }
}

/// HookReplaceHook allow replace uri
pub struct StripSuffixHook {
    matcher: Option<Box<dyn Matcher>>,
    target: Regex,
}

unsafe impl Send for StripSuffixHook {}
unsafe impl Sync for StripSuffixHook {}

impl StripSuffixHook {
    /// Build a new RegexHook out of its configuration
    pub fn new(config: HookConfig) -> Self {
        let target = Regex::new(&config.target).expect("Invalid target regex pattern");
        let matcher = if let Some(matcher_config) = config.matcher {
            Some(matcher_config.boxed_matcher())
        } else {
            None
        };
        Self { matcher, target }
    }
}

impl Hook for StripSuffixHook {
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

            let replaced_uri = self
                .target
                .replace_all(&source_uri.to_string(), String::new())
                .to_string();

            *source_uri = Uri::from_str(&replaced_uri)
                .map_err(|e| VetisError::hook(e.to_compact_string()))?;
            Ok((request, response))
        };

        Box::pin(future)
    }
}
