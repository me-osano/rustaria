//! URL pattern matching utilities.

use regex::Regex;

/// A compiled URL pattern for matching.
pub struct UrlPattern {
    regex: Regex,
    category: Option<String>,
}

impl UrlPattern {
    /// Create a new URL pattern.
    pub fn new(pattern: &str, category: Option<String>) -> anyhow::Result<Self> {
        Ok(Self {
            regex: Regex::new(pattern)?,
            category,
        })
    }

    /// Check if a URL matches this pattern.
    pub fn matches(&self, url: &str) -> bool {
        self.regex.is_match(url)
    }

    /// Get the category for this pattern.
    pub fn category(&self) -> Option<&str> {
        self.category.as_deref()
    }
}

/// Collection of URL patterns.
pub struct PatternMatcher {
    patterns: Vec<UrlPattern>,
}

impl PatternMatcher {
    /// Create a new pattern matcher.
    pub fn new() -> Self {
        Self {
            patterns: Vec::new(),
        }
    }

    /// Add a pattern.
    pub fn add(&mut self, pattern: UrlPattern) {
        self.patterns.push(pattern);
    }

    /// Find the first matching pattern for a URL.
    pub fn find_match(&self, url: &str) -> Option<&UrlPattern> {
        self.patterns.iter().find(|p| p.matches(url))
    }

    /// Check if any pattern matches the URL.
    pub fn matches(&self, url: &str) -> bool {
        self.patterns.iter().any(|p| p.matches(url))
    }

    /// Get category for a URL based on matching patterns.
    pub fn categorize(&self, url: &str) -> Option<&str> {
        self.find_match(url).and_then(|p| p.category())
    }
}

impl Default for PatternMatcher {
    fn default() -> Self {
        Self::new()
    }
}
