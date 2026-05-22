//! Privacy strip — the **only** way to construct an [`Observation`]
//! from an inbound hook payload.
//!
//! The type [`Sanitized<T>`] is a thin newtype wrapper. Callers obtain
//! it exclusively via [`Sanitized::new`], which scrubs the inner value's
//! string fields of common secrets. There is no other constructor and
//! the inner value is `pub(crate)` — preventing the "hook handler forgot
//! to call `sanitize()` before persisting" failure mode entirely.

use std::sync::OnceLock;

use ai_memory_core::NewObservation;
use regex::Regex;
use tracing::debug;

/// Marker carried by every observation-shaped value once it has passed
/// through the privacy strip.
#[derive(Debug, Clone)]
pub struct Sanitized<T>(T);

impl<T> Sanitized<T> {
    /// Borrow the inner sanitized value.
    pub fn inner(&self) -> &T {
        &self.0
    }

    /// Consume and return the inner sanitized value.
    pub fn into_inner(self) -> T {
        self.0
    }
}

impl Sanitized<NewObservation> {
    /// Run privacy strip rules over an inbound observation.
    #[must_use]
    pub fn new(mut obs: NewObservation) -> Self {
        obs.title = scrub(&obs.title);
        obs.body = scrub(&obs.body);
        Self(obs)
    }
}

/// Run privacy strip rules over an arbitrary text payload.
#[must_use]
pub fn scrub(input: &str) -> String {
    let mut out = input.to_string();
    for re in regex_set() {
        let before_len = out.len();
        out = re.replace_all(&out, "[REDACTED]").into_owned();
        if out.len() != before_len {
            debug!(pattern = %re.as_str(), "sanitize: redacted match");
        }
    }
    out
}

fn regex_set() -> &'static [Regex] {
    static SET: OnceLock<Vec<Regex>> = OnceLock::new();
    SET.get_or_init(|| {
        // Order matters: more-specific patterns first. We intentionally
        // accept some false positives in v1 — better to redact a stray
        // hash than to leak an API key.
        let raw_patterns: &[&str] = &[
            // Bearer-style tokens.
            r#"(?i)bearer\s+[A-Za-z0-9._\-+/=]{16,}"#,
            // sk-..., pk_live_..., GitHub PATs, AWS access keys, etc.
            r"sk-[A-Za-z0-9_\-]{16,}",
            r"sk_live_[A-Za-z0-9_\-]{16,}",
            r"ghp_[A-Za-z0-9]{20,}",
            r"github_pat_[A-Za-z0-9_]{20,}",
            r"AKIA[0-9A-Z]{12,}",
            // JWTs.
            r"eyJ[A-Za-z0-9_\-]{16,}\.[A-Za-z0-9_\-]{16,}\.[A-Za-z0-9_\-]{16,}",
            // Explicit env-var assignments commonly containing secrets.
            r#"(?i)(ANTHROPIC_API_KEY|OPENAI_API_KEY|VOYAGE_API_KEY|MISTRAL_API_KEY|GROQ_API_KEY|HF_TOKEN|HUGGINGFACE_TOKEN|AWS_(SECRET_)?ACCESS_KEY[A-Z_]*|GITHUB_TOKEN|GH_TOKEN|GITLAB_TOKEN|GOOGLE_API_KEY|GEMINI_API_KEY)\s*[=:]\s*\S+"#,
            // Paths under ~/.ssh.
            r"(?:/[^/\s]+)*/\.ssh(?:/[^\s]+)?",
        ];
        raw_patterns
            .iter()
            .map(|p| Regex::new(p).expect("compile sanitizer regex"))
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ai_memory_core::{NewObservation, ObservationKind, ProjectId, SessionId, WorkspaceId};

    #[test]
    fn scrubs_bearer_tokens() {
        let s = scrub("Authorization: Bearer abcdef0123456789ABCDEF0123456789");
        assert!(s.contains("[REDACTED]"));
        assert!(!s.contains("abcdef"));
    }

    #[test]
    fn scrubs_anthropic_key() {
        let s = scrub("env: ANTHROPIC_API_KEY=sk-ant-xxxxxxxxxxxxxxxxxxxxxxxxxxxx");
        assert!(s.contains("[REDACTED]"));
        assert!(!s.contains("sk-ant"));
    }

    #[test]
    fn scrubs_jwt() {
        let s = scrub("token: eyJhbGciOiJIUzI1NiJ9.eyJzdWIiOiIxMjM0NTY3ODkw.abcdefABCDEF123456");
        assert!(s.contains("[REDACTED]"));
    }

    #[test]
    fn scrubs_ssh_paths() {
        let s = scrub("found in /home/user/.ssh/id_ed25519");
        assert!(s.contains("[REDACTED]"));
        assert!(!s.contains("id_ed25519"));
    }

    #[test]
    fn observation_round_trip_through_sanitized() {
        let raw = NewObservation {
            session_id: SessionId::new(),
            workspace_id: WorkspaceId::new(),
            project_id: ProjectId::new(),
            kind: ObservationKind::UserPrompt,
            title: "fix the OPENAI_API_KEY=sk-1234567890abcdef in env".into(),
            body: "see ~/.ssh/secret.pem".into(),
            importance: 5,
        };
        let sanitized = Sanitized::new(raw);
        let obs = sanitized.into_inner();
        assert!(obs.title.contains("[REDACTED]"));
        assert!(obs.body.contains("[REDACTED]"));
    }
}
