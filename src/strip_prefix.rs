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
pub struct HookConfigBuilder<'a> {
    matcher: Option<Box<dyn MatcherConfig>>,
    target: &'a str,
}

impl<'a> HookConfigBuilder<'a> {
    /// Allow set a matcher
    ///
    /// # Returns
    ///
    /// * `Self` - The builder.
    pub fn matcher<M>(mut self, matcher: M) -> Self
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
    pub fn target(mut self, target: &'a str) -> Self {
        self.target = target;
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
            target: self
                .target
                .to_compact_string(),
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
/// use vetis_rewrite::strip_prefix::HookConfig;
///
/// let auth = HookConfig::builder()
///     .target("/home")
///     .build();
/// ```
pub struct HookConfig {
    matcher: Option<Box<dyn MatcherConfig>>,
    target: Str,
}

#[typetag::serde(name = "strip_prefix")]
impl vetis::hook::HookConfig for HookConfig {
    fn matcher(&self) -> Option<&Box<dyn MatcherConfig>> {
        self.matcher
            .as_ref()
    }

    fn boxed_clone(&self) -> Box<dyn vetis::hook::HookConfig> {
        Box::new(self.clone())
    }

    fn boxed_hook(&self) -> Box<dyn VetisHook> {
        Box::new(StripPrefixHook::new(self.clone()))
    }
}

impl HookConfig {
    /// Creates a new `HookConfigBuilder` with default settings.
    ///
    /// # Returns
    ///
    /// * `HookConfigBuilder` - The builder.
    pub fn builder<'a>() -> HookConfigBuilder<'a> {
        HookConfigBuilder { matcher: None, target: "/api" }
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
pub struct StripPrefixHook {
    matcher: Option<Box<dyn Matcher>>,
    target: Option<Regex>,
    config: HookConfig,
}

unsafe impl Send for StripPrefixHook {}
unsafe impl Sync for StripPrefixHook {}

impl StripPrefixHook {
    /// Build a new RegexHook out of its configuration
    pub fn new(config: HookConfig) -> Self {
        Self { matcher: None, target: None, config }
    }
}

impl Hook for StripPrefixHook {
    fn setup<'a>(&'a mut self) -> VetisFutureResult<'a, ()> {
        Box::pin(async move {
            let target = Regex::new(&self.config.target)
                .map_err(|e| VetisError::hook(e.to_compact_string()))?;

            let matcher = if let Some(matcher_config) = &self.config.matcher {
                Some(matcher_config.boxed_matcher())
            } else {
                None
            };
            self.target = Some(target);
            self.matcher = matcher;
            Ok(())
        })
    }

    fn has_matches(&self, context: &Context) -> bool {
        if let Some(matcher) = &self.matcher { matcher.is_match(context) } else { true }
    }

    fn phase(&self) -> HookPhase {
        HookPhase::Rewrite
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

            if let Some(target) = &self.target {
                let replaced_uri = target
                    .replace_all(&source_uri.to_string(), String::new())
                    .to_string();

                *source_uri = Uri::from_str(&replaced_uri)
                    .map_err(|e| VetisError::hook(e.to_compact_string()))?;
            };

            Ok((request, response))
        };

        Box::pin(future)
    }
}
