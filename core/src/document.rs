use anyhow::{Context, Result};
use serde::Deserialize;
use std::fmt;
use std::path::{Path, PathBuf};

/// All supported StrayMark document types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DocType {
    Ailog,
    Aidec,
    Adr,
    Eth,
    Req,
    Tes,
    Inc,
    Tde,
    Sec,
    Mcard,
    Sbom,
    Dpia,
    // China regulatory artifacts (regional_scope: china)
    Pipia,
    Cacfile,
    Tc260ra,
    Ailabel,
}

impl DocType {
    /// Prefix used in filenames (e.g., "AILOG", "SEC")
    pub fn prefix(&self) -> &'static str {
        match self {
            DocType::Ailog => "AILOG",
            DocType::Aidec => "AIDEC",
            DocType::Adr => "ADR",
            DocType::Eth => "ETH",
            DocType::Req => "REQ",
            DocType::Tes => "TES",
            DocType::Inc => "INC",
            DocType::Tde => "TDE",
            DocType::Sec => "SEC",
            DocType::Mcard => "MCARD",
            DocType::Sbom => "SBOM",
            DocType::Dpia => "DPIA",
            DocType::Pipia => "PIPIA",
            DocType::Cacfile => "CACFILE",
            DocType::Tc260ra => "TC260RA",
            DocType::Ailabel => "AILABEL",
        }
    }

    /// Parse a DocType from a filename prefix
    pub fn from_prefix(prefix: &str) -> Option<DocType> {
        match prefix {
            "AILOG" => Some(DocType::Ailog),
            "AIDEC" => Some(DocType::Aidec),
            "ADR" => Some(DocType::Adr),
            "ETH" => Some(DocType::Eth),
            "REQ" => Some(DocType::Req),
            "TES" => Some(DocType::Tes),
            "INC" => Some(DocType::Inc),
            "TDE" => Some(DocType::Tde),
            "SEC" => Some(DocType::Sec),
            "MCARD" => Some(DocType::Mcard),
            "SBOM" => Some(DocType::Sbom),
            "DPIA" => Some(DocType::Dpia),
            "PIPIA" => Some(DocType::Pipia),
            "CACFILE" => Some(DocType::Cacfile),
            "TC260RA" => Some(DocType::Tc260ra),
            "AILABEL" => Some(DocType::Ailabel),
            _ => None,
        }
    }

    /// All valid prefixes.
    ///
    /// Adding a DocType? Update BOTH this array and the
    /// `DOC_TYPE_PREFIXES` env var at the top of
    /// `dist/.github/workflows/docs-validation.yml`. The CI workflow uses
    /// that env var as its single source of truth for valid type prefixes,
    /// but it cannot import from Rust — the two must be kept in manual sync.
    pub const ALL_PREFIXES: &'static [&'static str] = &[
        "AILOG", "AIDEC", "ADR", "ETH", "REQ", "TES", "INC", "TDE",
        "SEC", "MCARD", "SBOM", "DPIA",
        "PIPIA", "CACFILE", "TC260RA", "AILABEL",
    ];

    /// All DocType variants in display order
    pub const ALL: &'static [DocType] = &[
        DocType::Ailog, DocType::Aidec, DocType::Adr, DocType::Eth,
        DocType::Req, DocType::Tes, DocType::Inc, DocType::Tde,
        DocType::Sec, DocType::Mcard, DocType::Sbom, DocType::Dpia,
        DocType::Pipia, DocType::Cacfile, DocType::Tc260ra, DocType::Ailabel,
    ];

    /// DocType variants that are only enabled when `regional_scope` includes
    /// "china". They are filtered out of `straymark new` and other UX surfaces
    /// for projects that have not opted into Chinese regulatory coverage.
    pub const CHINA_ONLY: &'static [DocType] = &[
        DocType::Pipia, DocType::Cacfile, DocType::Tc260ra, DocType::Ailabel,
    ];

    /// True if this DocType requires `regional_scope` to include "china".
    pub fn is_china_only(&self) -> bool {
        Self::CHINA_ONLY.contains(self)
    }

    /// Human-readable display name
    pub fn display_name(&self) -> &'static str {
        match self {
            DocType::Ailog => "AI Action Log",
            DocType::Aidec => "AI Decision",
            DocType::Adr => "Architecture Decision Record",
            DocType::Eth => "Ethical Review",
            DocType::Req => "Requirement",
            DocType::Tes => "Test Plan",
            DocType::Inc => "Incident Post-mortem",
            DocType::Tde => "Technical Debt",
            DocType::Sec => "Security Assessment",
            DocType::Mcard => "Model/System Card",
            DocType::Sbom => "Software Bill of Materials",
            DocType::Dpia => "Data Protection Impact Assessment",
            DocType::Pipia => "Personal Information Protection Impact Assessment",
            DocType::Cacfile => "CAC Algorithm Filing",
            DocType::Tc260ra => "TC260 Risk Assessment",
            DocType::Ailabel => "GB 45438 Content Labeling Plan",
        }
    }

    /// Subdirectory under .straymark/ where this document type lives
    pub fn directory(&self) -> &'static str {
        match self {
            DocType::Ailog => "07-ai-audit/agent-logs",
            DocType::Aidec => "07-ai-audit/decisions",
            DocType::Eth => "07-ai-audit/ethical-reviews",
            DocType::Adr => "02-design/decisions",
            DocType::Req => "01-requirements",
            DocType::Tes => "04-testing",
            DocType::Inc => "05-operations/incidents",
            DocType::Tde => "06-evolution/technical-debt",
            DocType::Sec => "08-security",
            DocType::Mcard => "09-ai-models",
            DocType::Sbom => "07-ai-audit",
            DocType::Dpia => "07-ai-audit/ethical-reviews",
            DocType::Pipia => "07-ai-audit/ethical-reviews",
            DocType::Cacfile => "07-ai-audit/regulatory-filings",
            DocType::Tc260ra => "07-ai-audit/risk-assessments",
            DocType::Ailabel => "09-ai-models/labeling",
        }
    }

    /// Parse a DocType from a user-provided string (case-insensitive)
    pub fn from_str_loose(s: &str) -> Option<DocType> {
        match s.to_lowercase().as_str() {
            "ailog" => Some(DocType::Ailog),
            "aidec" => Some(DocType::Aidec),
            "adr" => Some(DocType::Adr),
            "eth" => Some(DocType::Eth),
            "req" => Some(DocType::Req),
            "tes" => Some(DocType::Tes),
            "inc" => Some(DocType::Inc),
            "tde" => Some(DocType::Tde),
            "sec" => Some(DocType::Sec),
            "mcard" => Some(DocType::Mcard),
            "sbom" => Some(DocType::Sbom),
            "dpia" => Some(DocType::Dpia),
            "pipia" => Some(DocType::Pipia),
            "cacfile" => Some(DocType::Cacfile),
            "tc260ra" => Some(DocType::Tc260ra),
            "ailabel" => Some(DocType::Ailabel),
            _ => None,
        }
    }
}

impl fmt::Display for DocType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.prefix())
    }
}

/// Frontmatter fields extracted from a StrayMark document.
/// All fields are optional so the validator can report which are missing.
#[derive(Debug, Clone, Deserialize, Default)]
#[allow(dead_code)]
pub struct Frontmatter {
    pub id: Option<String>,
    pub title: Option<String>,
    pub status: Option<String>,
    pub created: Option<String>,
    pub agent: Option<String>,
    pub confidence: Option<String>,
    pub review_required: Option<bool>,
    /// Reviewer identity (email | github-handle | DID). Set by `straymark approve`
    /// and by `## 3.5 Recording Approval` of the framework's documentation policy.
    pub reviewed_by: Option<String>,
    /// Date of formal approval (must be >= `created`).
    pub reviewed_at: Option<String>,
    /// Closure signal — one of: `approved | revisions_requested | rejected`.
    /// Presence of this field is the canonical "human has reviewed" signal.
    pub review_outcome: Option<String>,
    pub risk_level: Option<String>,
    pub eu_ai_act_risk: Option<String>,
    pub nist_genai_risks: Option<Vec<String>>,
    pub iso_42001_clause: Option<Vec<u8>>,
    pub tags: Option<Vec<String>>,
    pub related: Option<Vec<String>>,
    /// Documents this one supersedes (graph edge `SUPERSEDES`).
    pub supersedes: Option<Vec<String>>,
    /// Alternatives documented elsewhere (graph edge `DOCUMENTS_ALTERNATIVE`).
    pub alternatives_documented: Option<Vec<String>>,
    /// AILOGs a document originates from (graph edge `ORIGINATES_FROM`).
    pub originating_ailogs: Option<Vec<String>>,
    // INC-specific
    pub severity: Option<String>,
    // ETH-specific
    pub gdpr_legal_basis: Option<String>,
    // SEC-specific
    pub threat_model_methodology: Option<String>,
    pub owasp_asvs_level: Option<serde_yaml::Value>,
    // MCARD-specific
    pub model_name: Option<String>,
    pub model_type: Option<String>,
    pub model_version: Option<String>,
    pub provider: Option<String>,
    pub license: Option<String>,
    // SBOM-specific
    pub sbom_format_reference: Option<String>,
    pub system_name: Option<String>,
    // DPIA-specific
    pub gdpr_article_35: Option<bool>,
    pub dpo_consulted: Option<bool>,
    pub supervisory_authority_consulted: Option<bool>,
    // ADR-specific
    pub api_changes: Option<Vec<String>>,
    // REQ-specific (OpenAPI/AsyncAPI)
    pub api_spec_path: Option<String>,

    // ----- China regulatory profile (regional_scope: china) -----

    // TC260 v2.0 (AI Safety Governance Framework)
    /// One of: "low" | "medium" | "high" | "very_high" | "extremely_severe" | "not_applicable"
    pub tc260_risk_level: Option<String>,
    pub tc260_application_scenario: Option<String>,
    /// One of: "narrow" | "foundation" | "agentic" | "general"
    pub tc260_intelligence_level: Option<String>,
    /// One of: "individual" | "organization" | "societal" | "cross_border"
    pub tc260_application_scale: Option<String>,
    pub tc260_endogenous_risks: Option<Vec<String>>,
    pub tc260_application_risks: Option<Vec<String>>,
    pub tc260_derivative_risks: Option<Vec<String>>,

    // PIPL / PIPIA (Personal Information Protection Law, Art. 55-56)
    pub pipl_applicable: Option<bool>,
    /// One of: "sensitive_data" | "automated_decision" | "third_party_disclosure"
    /// | "cross_border" | "public_disclosure" | "other"
    pub pipl_article_55_trigger: Option<String>,
    pub pipl_sensitive_data: Option<bool>,
    pub pipl_cross_border_transfer: Option<bool>,
    /// YYYY-MM-DD — minimum 3 years from `created` per PIPL.
    pub pipl_retention_until: Option<String>,

    // GB 45438-2025 — Cybersecurity Technology — Labeling Method for AI-Generated Content
    pub gb45438_applicable: Option<bool>,
    /// Subset of: "text" | "image" | "audio" | "video" | "virtual_scene"
    pub gb45438_content_types: Option<Vec<String>>,
    /// One of: "disclaimer" | "watermark" | "caption" | "audio_cue" | "banner"
    pub gb45438_explicit_label_strategy: Option<String>,
    /// One of: "C2PA" | "XMP" | "EXIF" | "custom" | "none"
    pub gb45438_implicit_metadata_format: Option<String>,
    pub gb45438_distributor_obligations_documented: Option<bool>,

    // CAC Algorithm Filing (Cyberspace Administration of China)
    pub cac_filing_required: Option<bool>,
    pub cac_filing_number: Option<String>,
    /// One of: "pending" | "provincial_submitted" | "provincial_approved"
    /// | "national_submitted" | "national_approved" | "rejected" | "not_required"
    pub cac_filing_status: Option<String>,
    /// One of: "algorithm" | "generative_ai" | "dual"
    pub cac_filing_type: Option<String>,
    pub cac_provincial_authority: Option<String>,
    pub cac_national_decision_date: Option<String>,

    // GB/T 45652-2025 — Pre-training & fine-tuning data security
    pub gb45652_training_data_compliance: Option<bool>,

    // CSL 2026 — Cybersecurity Law amendments + incident reporting administrative measures
    /// One of: "particularly_serious" | "relatively_major" | "major" | "general" | "not_applicable"
    pub csl_severity_level: Option<String>,
    /// 1 (particularly serious) | 4 (relatively major) | 24 (general)
    pub csl_report_deadline_hours: Option<u32>,
}

/// A parsed StrayMark document
#[derive(Debug)]
pub struct StrayMarkDocument {
    pub path: PathBuf,
    pub filename: String,
    pub doc_type: DocType,
    pub frontmatter: Frontmatter,
    pub body: String,
}

/// Parse a StrayMark document from a file path
pub fn parse_document(path: &Path) -> Result<StrayMarkDocument> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read {}", path.display()))?;

    let filename = path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("")
        .to_string();

    // Determine doc type from filename prefix
    let doc_type = detect_doc_type(&filename)
        .with_context(|| format!("Cannot determine document type for {}", filename))?;

    // Extract frontmatter
    let (frontmatter, body) = extract_frontmatter(&content)
        .with_context(|| format!("Failed to parse frontmatter in {}", path.display()))?;

    Ok(StrayMarkDocument {
        path: path.to_path_buf(),
        filename,
        doc_type,
        frontmatter,
        body,
    })
}

/// Detect document type from filename prefix
pub fn detect_doc_type(filename: &str) -> Option<DocType> {
    for prefix in DocType::ALL_PREFIXES {
        if filename.starts_with(&format!("{}-", prefix)) {
            return DocType::from_prefix(prefix);
        }
    }
    None
}

/// Extract YAML frontmatter (between --- delimiters) and body
fn extract_frontmatter(content: &str) -> Result<(Frontmatter, String)> {
    let trimmed = content.trim_start();
    if !trimmed.starts_with("---") {
        anyhow::bail!("No frontmatter found (missing opening ---)");
    }

    let after_first = &trimmed[3..];
    let end_pos = after_first
        .find("\n---")
        .ok_or_else(|| anyhow::anyhow!("No closing --- found for frontmatter"))?;

    let yaml_str = &after_first[..end_pos];
    let body_start = end_pos + 4; // skip "\n---"
    let body = if body_start < after_first.len() {
        after_first[body_start..].to_string()
    } else {
        String::new()
    };

    let frontmatter: Frontmatter = serde_yaml::from_str(yaml_str)
        .with_context(|| "Failed to deserialize frontmatter YAML")?;

    Ok((frontmatter, body))
}

/// Discover all user-generated StrayMark documents under a .straymark/ directory
pub fn discover_documents(straymark_dir: &Path) -> Vec<PathBuf> {
    let mut results = Vec::new();
    walk_for_documents(straymark_dir, &mut results);
    results.sort();
    results
}

fn walk_for_documents(dir: &Path, results: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };

    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            // Skip templates directory
            if path.ends_with("templates") {
                continue;
            }
            walk_for_documents(&path, results);
        } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
            let filename = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("");
            // Match pattern: TYPE-YYYY-MM-DD-NNN-*.md
            if detect_doc_type(filename).is_some() && is_dated_document(filename) {
                results.push(path);
            }
        }
    }
}

/// Check if filename follows the dated pattern TYPE-YYYY-MM-DD-NNN-*.md
fn is_dated_document(filename: &str) -> bool {
    // Find the first '-' (after the type prefix)
    let after_prefix = match filename.find('-') {
        Some(pos) => &filename[pos + 1..],
        None => return false,
    };
    // Should start with a date pattern YYYY-MM-DD
    if after_prefix.len() < 10 {
        return false;
    }
    let date_part = &after_prefix[..10];
    // Basic date pattern check: NNNN-NN-NN
    date_part.len() == 10
        && date_part.chars().nth(4) == Some('-')
        && date_part.chars().nth(7) == Some('-')
        && date_part[..4].chars().all(|c| c.is_ascii_digit())
        && date_part[5..7].chars().all(|c| c.is_ascii_digit())
        && date_part[8..10].chars().all(|c| c.is_ascii_digit())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_doc_type() {
        assert_eq!(detect_doc_type("AILOG-2025-01-01-001-test.md"), Some(DocType::Ailog));
        assert_eq!(detect_doc_type("SEC-2025-01-01-001-auth.md"), Some(DocType::Sec));
        assert_eq!(detect_doc_type("MCARD-2025-01-01-001-gpt.md"), Some(DocType::Mcard));
        assert_eq!(detect_doc_type("SBOM-2025-01-01-001-deps.md"), Some(DocType::Sbom));
        assert_eq!(detect_doc_type("DPIA-2025-01-01-001-gdpr.md"), Some(DocType::Dpia));
        assert_eq!(detect_doc_type("README.md"), None);
        assert_eq!(detect_doc_type("TEMPLATE-SEC.md"), None);
    }

    #[test]
    fn test_is_dated_document() {
        assert!(is_dated_document("AILOG-2025-01-27-001-implement-auth.md"));
        assert!(is_dated_document("SEC-2026-03-24-001-api-review.md"));
        assert!(!is_dated_document("TEMPLATE-SEC.md"));
        assert!(!is_dated_document("README.md"));
    }

    #[test]
    fn test_extract_frontmatter() {
        let content = "---\nid: AILOG-2025-01-01-001\ntitle: Test\nstatus: draft\n---\n\n# Body";
        let (fm, body) = extract_frontmatter(content).unwrap();
        assert_eq!(fm.id.as_deref(), Some("AILOG-2025-01-01-001"));
        assert_eq!(fm.title.as_deref(), Some("Test"));
        assert!(body.contains("# Body"));
    }

    #[test]
    fn test_doc_type_all_has_16_entries() {
        // 12 base types + 4 China-specific (PIPIA, CACFILE, TC260RA, AILABEL)
        assert_eq!(DocType::ALL.len(), 16);
        assert_eq!(DocType::ALL_PREFIXES.len(), 16);
    }

    #[test]
    fn test_china_only_doc_types() {
        assert_eq!(DocType::CHINA_ONLY.len(), 4);
        assert!(DocType::Pipia.is_china_only());
        assert!(DocType::Cacfile.is_china_only());
        assert!(DocType::Tc260ra.is_china_only());
        assert!(DocType::Ailabel.is_china_only());
        assert!(!DocType::Ailog.is_china_only());
        assert!(!DocType::Dpia.is_china_only());
    }

    #[test]
    fn test_china_doc_type_detection() {
        assert_eq!(detect_doc_type("PIPIA-2026-04-25-001-chatbot.md"), Some(DocType::Pipia));
        assert_eq!(detect_doc_type("CACFILE-2026-04-25-001-chatbot.md"), Some(DocType::Cacfile));
        assert_eq!(detect_doc_type("TC260RA-2026-04-25-001-chatbot.md"), Some(DocType::Tc260ra));
        assert_eq!(detect_doc_type("AILABEL-2026-04-25-001-chatbot.md"), Some(DocType::Ailabel));
    }

    #[test]
    fn test_china_doc_type_directories() {
        assert_eq!(DocType::Pipia.directory(), "07-ai-audit/ethical-reviews");
        assert_eq!(DocType::Cacfile.directory(), "07-ai-audit/regulatory-filings");
        assert_eq!(DocType::Tc260ra.directory(), "07-ai-audit/risk-assessments");
        assert_eq!(DocType::Ailabel.directory(), "09-ai-models/labeling");
    }

    #[test]
    fn test_china_frontmatter_parsing() {
        let content = "---\n\
            id: PIPIA-2026-04-25-001\n\
            title: Test PIPIA\n\
            pipl_applicable: true\n\
            pipl_sensitive_data: true\n\
            pipl_cross_border_transfer: false\n\
            pipl_retention_until: 2029-04-25\n\
            tc260_risk_level: high\n\
            cac_filing_number: CAC-2026-00123\n\
            cac_filing_status: national_approved\n\
            gb45438_content_types: [text, image]\n\
            csl_severity_level: relatively_major\n\
            csl_report_deadline_hours: 4\n\
            ---\n\nbody";
        let (fm, _) = extract_frontmatter(content).unwrap();
        assert_eq!(fm.pipl_applicable, Some(true));
        assert_eq!(fm.pipl_sensitive_data, Some(true));
        assert_eq!(fm.pipl_retention_until.as_deref(), Some("2029-04-25"));
        assert_eq!(fm.tc260_risk_level.as_deref(), Some("high"));
        assert_eq!(fm.cac_filing_number.as_deref(), Some("CAC-2026-00123"));
        assert_eq!(fm.cac_filing_status.as_deref(), Some("national_approved"));
        assert_eq!(fm.gb45438_content_types.as_ref().unwrap().len(), 2);
        assert_eq!(fm.csl_severity_level.as_deref(), Some("relatively_major"));
        assert_eq!(fm.csl_report_deadline_hours, Some(4));
    }

    #[test]
    fn test_doc_type_directory_mapping() {
        for dt in DocType::ALL {
            let dir = dt.directory();
            assert!(!dir.is_empty(), "{} has empty directory", dt.prefix());
            assert!(!dir.starts_with('/'), "{} directory should be relative", dt.prefix());
        }
    }

    #[test]
    fn test_doc_type_display_names() {
        for dt in DocType::ALL {
            let name = dt.display_name();
            assert!(!name.is_empty(), "{} has empty display_name", dt.prefix());
        }
    }

    #[test]
    fn test_doc_type_from_str_loose() {
        assert_eq!(DocType::from_str_loose("ailog"), Some(DocType::Ailog));
        assert_eq!(DocType::from_str_loose("AILOG"), Some(DocType::Ailog));
        assert_eq!(DocType::from_str_loose("AiLog"), Some(DocType::Ailog));
        assert_eq!(DocType::from_str_loose("sec"), Some(DocType::Sec));
        assert_eq!(DocType::from_str_loose("mcard"), Some(DocType::Mcard));
        assert_eq!(DocType::from_str_loose("invalid"), None);
    }
}
