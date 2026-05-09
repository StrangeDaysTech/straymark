use serde::Serialize;
use std::path::Path;

use crate::document::{StrayMarkDocument, DocType};

/// The 12 NIST AI 600-1 GenAI risk categories (canonical identifiers)
pub const NIST_GENAI_CATEGORIES: &[&str] = &[
    "cbrn",
    "confabulation",
    "dangerous_content",
    "privacy",
    "environmental",
    "bias",
    "human_ai_config",
    "information_integrity",
    "information_security",
    "intellectual_property",
    "obscene_content",
    "value_chain",
];

/// Which regulatory standard to check
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Standard {
    EuAiAct,
    Iso42001,
    NistAiRmf,
    // China regulatory frameworks (regional_scope: china)
    ChinaTc260,
    ChinaPipl,
    ChinaGb45438,
    ChinaCac,
    ChinaGb45652,
    ChinaCsl,
}

impl Standard {
    pub fn label(&self) -> &'static str {
        match self {
            Standard::EuAiAct => "EU AI Act",
            Standard::Iso42001 => "ISO/IEC 42001",
            Standard::NistAiRmf => "NIST AI RMF",
            Standard::ChinaTc260 => "China TC260 v2.0",
            Standard::ChinaPipl => "China PIPL",
            Standard::ChinaGb45438 => "China GB 45438",
            Standard::ChinaCac => "China CAC Algorithm Filing",
            Standard::ChinaGb45652 => "China GB/T 45652",
            Standard::ChinaCsl => "China CSL 2026",
        }
    }

    /// Region this standard belongs to (used for `regional_scope` filtering).
    /// "global" = always available; "eu" = EU AI Act + GDPR; "china" = the 6 Chinese frameworks.
    pub fn region(&self) -> &'static str {
        match self {
            Standard::EuAiAct => "eu",
            Standard::Iso42001 | Standard::NistAiRmf => "global",
            Standard::ChinaTc260
            | Standard::ChinaPipl
            | Standard::ChinaGb45438
            | Standard::ChinaCac
            | Standard::ChinaGb45652
            | Standard::ChinaCsl => "china",
        }
    }
}

/// Status of a single compliance check
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum CheckStatus {
    Pass,
    Partial,
    Fail,
}

/// A single compliance check result
#[derive(Debug, Clone, Serialize)]
pub struct ComplianceCheck {
    pub id: String,
    pub description: String,
    pub status: CheckStatus,
    pub evidence: Vec<String>,
    pub remediation: Option<String>,
}

/// Result of running one standard's checker
#[derive(Debug, Serialize)]
pub struct ComplianceReport {
    pub standard: Standard,
    pub standard_label: String,
    pub checks: Vec<ComplianceCheck>,
    pub score: f64,
}

/// Check if a governance file exists in the straymark directory
fn governance_file_exists(straymark_dir: &Path, filename: &str) -> bool {
    straymark_dir.join("00-governance").join(filename).exists()
}

/// Count documents of a specific type
fn count_type(docs: &[StrayMarkDocument], doc_type: DocType) -> usize {
    docs.iter().filter(|d| d.doc_type == doc_type).count()
}

/// Collect document IDs of a specific type
fn ids_of_type(docs: &[StrayMarkDocument], doc_type: DocType) -> Vec<String> {
    docs.iter()
        .filter(|d| d.doc_type == doc_type)
        .filter_map(|d| d.frontmatter.id.clone())
        .collect()
}

/// Calculate score from checks
fn calculate_score(checks: &[ComplianceCheck]) -> f64 {
    if checks.is_empty() {
        return 0.0;
    }
    let total = checks.len() as f64;
    let passed = checks.iter().filter(|c| c.status == CheckStatus::Pass).count() as f64;
    let partial = checks
        .iter()
        .filter(|c| c.status == CheckStatus::Partial)
        .count() as f64;
    ((passed + partial * 0.5) / total) * 100.0
}

// ---------------------------------------------------------------------------
// EU AI Act checker
// ---------------------------------------------------------------------------

pub fn check_eu_ai_act(docs: &[StrayMarkDocument], _governance_dir: &Path) -> ComplianceReport {
    let mut checks = Vec::new();

    // EU-001: Risk classification — at least one doc classifies EU AI Act risk
    {
        let classified: Vec<String> = docs
            .iter()
            .filter(|d| {
                d.frontmatter
                    .eu_ai_act_risk
                    .as_deref()
                    .is_some_and(|r| r != "not_applicable")
            })
            .filter_map(|d| d.frontmatter.id.clone())
            .collect();

        let status = if classified.is_empty() {
            CheckStatus::Fail
        } else {
            CheckStatus::Pass
        };
        checks.push(ComplianceCheck {
            id: "EU-001".into(),
            description: "AI systems have EU AI Act risk classification".into(),
            status,
            evidence: classified,
            remediation: Some(
                "Set eu_ai_act_risk field in documents involving AI systems".into(),
            ),
        });
    }

    // EU-002: High-risk docs have ETH linked in related
    {
        let high_risk_docs: Vec<&StrayMarkDocument> = docs
            .iter()
            .filter(|d| d.frontmatter.eu_ai_act_risk.as_deref() == Some("high"))
            .collect();

        if high_risk_docs.is_empty() {
            checks.push(ComplianceCheck {
                id: "EU-002".into(),
                description: "High-risk AI systems have ethical review (ETH) linked".into(),
                status: CheckStatus::Pass,
                evidence: vec!["No high-risk systems — check not applicable".into()],
                remediation: None,
            });
        } else {
            let eth_ids: Vec<String> = ids_of_type(docs, DocType::Eth);
            let mut linked = Vec::new();
            let mut unlinked = Vec::new();

            for doc in &high_risk_docs {
                let has_eth_related = doc
                    .frontmatter
                    .related
                    .as_ref()
                    .is_some_and(|rels| rels.iter().any(|r| r.starts_with("ETH-")));
                let doc_id = doc.frontmatter.id.clone().unwrap_or_default();
                if has_eth_related {
                    linked.push(doc_id);
                } else {
                    unlinked.push(doc_id);
                }
            }

            let status = if unlinked.is_empty() {
                CheckStatus::Pass
            } else if linked.is_empty() && !eth_ids.is_empty() {
                CheckStatus::Partial
            } else if eth_ids.is_empty() {
                CheckStatus::Fail
            } else {
                CheckStatus::Partial
            };

            checks.push(ComplianceCheck {
                id: "EU-002".into(),
                description: "High-risk AI systems have ethical review (ETH) linked".into(),
                status,
                evidence: linked,
                remediation: if !unlinked.is_empty() {
                    Some(format!(
                        "Link ETH documents in 'related' for: {}",
                        unlinked.join(", ")
                    ))
                } else {
                    None
                },
            });
        }
    }

    // EU-003: DPIA exists if needed
    {
        let needs_dpia = docs.iter().any(|d| d.frontmatter.gdpr_article_35 == Some(true));
        let has_dpia = count_type(docs, DocType::Dpia) > 0;

        if !needs_dpia {
            checks.push(ComplianceCheck {
                id: "EU-003".into(),
                description: "Data Protection Impact Assessment (DPIA) exists where required"
                    .into(),
                status: CheckStatus::Pass,
                evidence: vec!["No GDPR Art. 35 triggers found — check not applicable".into()],
                remediation: None,
            });
        } else {
            checks.push(ComplianceCheck {
                id: "EU-003".into(),
                description: "Data Protection Impact Assessment (DPIA) exists where required"
                    .into(),
                status: if has_dpia {
                    CheckStatus::Pass
                } else {
                    CheckStatus::Fail
                },
                evidence: ids_of_type(docs, DocType::Dpia),
                remediation: if !has_dpia {
                    Some("Create a DPIA document for processing with gdpr_article_35: true".into())
                } else {
                    None
                },
            });
        }
    }

    // EU-004: Incident reporting compliance
    {
        let inc_docs: Vec<&StrayMarkDocument> = docs
            .iter()
            .filter(|d| d.doc_type == DocType::Inc)
            .collect();

        if inc_docs.is_empty() {
            checks.push(ComplianceCheck {
                id: "EU-004".into(),
                description: "Incident reporting compliant with EU AI Act Art. 73".into(),
                status: CheckStatus::Pass,
                evidence: vec!["No incidents recorded — check not applicable".into()],
                remediation: None,
            });
        } else {
            let with_severity: Vec<String> = inc_docs
                .iter()
                .filter(|d| d.frontmatter.severity.is_some())
                .filter_map(|d| d.frontmatter.id.clone())
                .collect();

            let status = if with_severity.len() == inc_docs.len() {
                CheckStatus::Pass
            } else if !with_severity.is_empty() {
                CheckStatus::Partial
            } else {
                CheckStatus::Fail
            };

            checks.push(ComplianceCheck {
                id: "EU-004".into(),
                description: "Incident reporting compliant with EU AI Act Art. 73".into(),
                status,
                evidence: with_severity,
                remediation: Some(
                    "Ensure all INC documents have severity and report deadlines".into(),
                ),
            });
        }
    }

    let score = calculate_score(&checks);
    ComplianceReport {
        standard: Standard::EuAiAct,
        standard_label: Standard::EuAiAct.label().into(),
        checks,
        score,
    }
}

// ---------------------------------------------------------------------------
// ISO/IEC 42001 checker
// ---------------------------------------------------------------------------

pub fn check_iso_42001(docs: &[StrayMarkDocument], governance_dir: &Path) -> ComplianceReport {
    let mut checks = Vec::new();

    // ISO-001: AI Governance Policy exists
    {
        let exists = governance_file_exists(governance_dir, "AI-GOVERNANCE-POLICY.md");
        checks.push(ComplianceCheck {
            id: "ISO-001".into(),
            description: "AI Governance Policy exists (Clauses 4-5)".into(),
            status: if exists {
                CheckStatus::Pass
            } else {
                CheckStatus::Fail
            },
            evidence: if exists {
                vec!["AI-GOVERNANCE-POLICY.md".into()]
            } else {
                vec![]
            },
            remediation: if !exists {
                Some("Run 'straymark init' or 'straymark repair' to restore governance files".into())
            } else {
                None
            },
        });
    }

    // ISO-002: Risk planning — at least one ETH (Clause 6)
    {
        let eth_ids = ids_of_type(docs, DocType::Eth);
        checks.push(ComplianceCheck {
            id: "ISO-002".into(),
            description: "Risk planning documented — ETH reviews exist (Clause 6)".into(),
            status: if eth_ids.is_empty() {
                CheckStatus::Fail
            } else {
                CheckStatus::Pass
            },
            evidence: eth_ids.clone(),
            remediation: if eth_ids.is_empty() {
                Some("Create at least one ETH (Ethical Review) document for risk planning".into())
            } else {
                None
            },
        });
    }

    // ISO-003: Operations documented — AILOG + AIDEC exist (Clause 8)
    {
        let has_ailog = count_type(docs, DocType::Ailog) > 0;
        let has_aidec = count_type(docs, DocType::Aidec) > 0;

        let status = if has_ailog && has_aidec {
            CheckStatus::Pass
        } else if has_ailog || has_aidec {
            CheckStatus::Partial
        } else {
            CheckStatus::Fail
        };

        let mut evidence = ids_of_type(docs, DocType::Ailog);
        evidence.extend(ids_of_type(docs, DocType::Aidec));

        checks.push(ComplianceCheck {
            id: "ISO-003".into(),
            description: "AI lifecycle operations documented — AILOG + AIDEC (Clause 8)".into(),
            status,
            evidence,
            remediation: if status != CheckStatus::Pass {
                Some("Document AI actions (AILOG) and decisions (AIDEC) during development".into())
            } else {
                None
            },
        });
    }

    // ISO-004: Annex A coverage via document types
    {
        // Annex A groups and the doc types that cover them
        let annex_a_groups: &[(&str, &[DocType])] = &[
            ("A.5 Impact Assessment", &[DocType::Eth, DocType::Dpia]),
            (
                "A.6 AI Lifecycle",
                &[DocType::Ailog, DocType::Aidec, DocType::Adr, DocType::Mcard],
            ),
            ("A.7 Data for AI", &[DocType::Sbom, DocType::Mcard]),
            ("A.8 Information", &[DocType::Adr, DocType::Req]),
            ("A.9 Use of AI", &[DocType::Ailog]),
            ("A.10 Third-Party", &[DocType::Sbom]),
        ];

        let mut covered = Vec::new();
        let mut missing = Vec::new();

        for (group_name, types) in annex_a_groups {
            let has_any = types.iter().any(|t| count_type(docs, *t) > 0);
            if has_any {
                covered.push((*group_name).to_string());
            } else {
                missing.push((*group_name).to_string());
            }
        }

        let total = annex_a_groups.len();
        let status = if covered.len() == total {
            CheckStatus::Pass
        } else if !covered.is_empty() {
            CheckStatus::Partial
        } else {
            CheckStatus::Fail
        };

        checks.push(ComplianceCheck {
            id: "ISO-004".into(),
            description: format!(
                "Annex A control coverage ({}/{} groups)",
                covered.len(),
                total
            ),
            status,
            evidence: covered,
            remediation: if !missing.is_empty() {
                Some(format!("Missing coverage for: {}", missing.join(", ")))
            } else {
                None
            },
        });
    }

    let score = calculate_score(&checks);
    ComplianceReport {
        standard: Standard::Iso42001,
        standard_label: Standard::Iso42001.label().into(),
        checks,
        score,
    }
}

// ---------------------------------------------------------------------------
// NIST AI RMF checker
// ---------------------------------------------------------------------------

pub fn check_nist_ai_rmf(docs: &[StrayMarkDocument], governance_dir: &Path) -> ComplianceReport {
    let mut checks = Vec::new();

    // NIST-MAP-001: MAP function — AILOG documents exist (context)
    {
        let ids = ids_of_type(docs, DocType::Ailog);
        checks.push(ComplianceCheck {
            id: "NIST-MAP-001".into(),
            description: "MAP function — AI actions documented (AILOG)".into(),
            status: if ids.is_empty() {
                CheckStatus::Fail
            } else {
                CheckStatus::Pass
            },
            evidence: ids.clone(),
            remediation: if ids.is_empty() {
                Some("Create AILOG documents to map AI system context".into())
            } else {
                None
            },
        });
    }

    // NIST-MEASURE-001: MEASURE function — TES documents exist
    {
        let ids = ids_of_type(docs, DocType::Tes);
        checks.push(ComplianceCheck {
            id: "NIST-MEASURE-001".into(),
            description: "MEASURE function — Test plans exist (TES)".into(),
            status: if ids.is_empty() {
                CheckStatus::Fail
            } else {
                CheckStatus::Pass
            },
            evidence: ids.clone(),
            remediation: if ids.is_empty() {
                Some("Create TES documents to measure AI system trustworthiness".into())
            } else {
                None
            },
        });
    }

    // NIST-MANAGE-001: MANAGE function — ETH + INC exist
    {
        let has_eth = count_type(docs, DocType::Eth) > 0;
        let has_inc = count_type(docs, DocType::Inc) > 0;

        let status = if has_eth && has_inc {
            CheckStatus::Pass
        } else if has_eth || has_inc {
            CheckStatus::Partial
        } else {
            CheckStatus::Fail
        };

        let mut evidence = ids_of_type(docs, DocType::Eth);
        evidence.extend(ids_of_type(docs, DocType::Inc));

        checks.push(ComplianceCheck {
            id: "NIST-MANAGE-001".into(),
            description: "MANAGE function — Risk management documented (ETH + INC)".into(),
            status,
            evidence,
            remediation: if status != CheckStatus::Pass {
                Some("Create ETH and INC documents for risk management".into())
            } else {
                None
            },
        });
    }

    // NIST-GOVERN-001: GOVERN function — governance policy + ADR exist
    {
        let has_policy = governance_file_exists(governance_dir, "AI-GOVERNANCE-POLICY.md");
        let has_adr = count_type(docs, DocType::Adr) > 0;

        let status = if has_policy && has_adr {
            CheckStatus::Pass
        } else if has_policy || has_adr {
            CheckStatus::Partial
        } else {
            CheckStatus::Fail
        };

        let mut evidence: Vec<String> = Vec::new();
        if has_policy {
            evidence.push("AI-GOVERNANCE-POLICY.md".into());
        }
        evidence.extend(ids_of_type(docs, DocType::Adr));

        checks.push(ComplianceCheck {
            id: "NIST-GOVERN-001".into(),
            description: "GOVERN function — Governance policy and decisions documented".into(),
            status,
            evidence,
            remediation: if status != CheckStatus::Pass {
                Some("Ensure AI-GOVERNANCE-POLICY.md exists and create ADR documents".into())
            } else {
                None
            },
        });
    }

    // NIST-GENAI-001: GenAI risk coverage (12 NIST 600-1 categories)
    {
        let mut covered_categories: Vec<String> = Vec::new();
        for doc in docs {
            if let Some(risks) = &doc.frontmatter.nist_genai_risks {
                for risk in risks {
                    let r = risk.to_lowercase();
                    if NIST_GENAI_CATEGORIES.contains(&r.as_str())
                        && !covered_categories.contains(&r)
                    {
                        covered_categories.push(r);
                    }
                }
            }
        }

        let coverage = covered_categories.len();
        let total = NIST_GENAI_CATEGORIES.len();

        let status = if coverage == total {
            CheckStatus::Pass
        } else if coverage > 0 {
            CheckStatus::Partial
        } else {
            CheckStatus::Fail
        };

        let missing: Vec<&str> = NIST_GENAI_CATEGORIES
            .iter()
            .filter(|c| !covered_categories.contains(&c.to_string()))
            .copied()
            .collect();

        checks.push(ComplianceCheck {
            id: "NIST-GENAI-001".into(),
            description: format!(
                "GenAI risk coverage — NIST AI 600-1 ({}/{} categories)",
                coverage, total
            ),
            status,
            evidence: covered_categories,
            remediation: if !missing.is_empty() {
                Some(format!(
                    "Add nist_genai_risks entries for: {}",
                    missing.join(", ")
                ))
            } else {
                None
            },
        });
    }

    let score = calculate_score(&checks);
    ComplianceReport {
        standard: Standard::NistAiRmf,
        standard_label: Standard::NistAiRmf.label().into(),
        checks,
        score,
    }
}

// ===========================================================================
// China regulatory checkers (regional_scope: china)
// ===========================================================================

/// True if any document in the corpus declares the given doc type.
fn has_type(docs: &[StrayMarkDocument], doc_type: DocType) -> bool {
    count_type(docs, doc_type) > 0
}

/// Documents whose `related:` list references at least one of the given prefixes.
fn related_includes_prefix(doc: &StrayMarkDocument, prefix: &str) -> bool {
    doc.frontmatter
        .related
        .as_ref()
        .is_some_and(|r| r.iter().any(|s| s.starts_with(prefix)))
}

// ---------------------------------------------------------------------------
// TC260 v2.0 checker
// ---------------------------------------------------------------------------

const TC260_HIGH_LEVELS: &[&str] = &["high", "very_high", "extremely_severe"];

pub fn check_china_tc260(
    docs: &[StrayMarkDocument],
    _governance_dir: &Path,
) -> ComplianceReport {
    let mut checks = Vec::new();

    // TC260-001: at least one TC260RA exists
    {
        let ids = ids_of_type(docs, DocType::Tc260ra);
        checks.push(ComplianceCheck {
            id: "TC260-001".into(),
            description: "At least one TC260 Risk Assessment (TC260RA) is present".into(),
            status: if ids.is_empty() {
                CheckStatus::Fail
            } else {
                CheckStatus::Pass
            },
            evidence: ids.clone(),
            remediation: if ids.is_empty() {
                Some("Run 'straymark new tc260ra' for each AI system in scope".into())
            } else {
                None
            },
        });
    }

    // TC260-002: high / very_high / extremely_severe levels require review_required: true
    {
        let high_risk: Vec<&StrayMarkDocument> = docs
            .iter()
            .filter(|d| {
                d.frontmatter
                    .tc260_risk_level
                    .as_deref()
                    .is_some_and(|l| TC260_HIGH_LEVELS.contains(&l))
            })
            .collect();

        if high_risk.is_empty() {
            checks.push(ComplianceCheck {
                id: "TC260-002".into(),
                description: "High / very-high / extremely-severe TC260 levels mandate review"
                    .into(),
                status: CheckStatus::Pass,
                evidence: vec!["No high-risk TC260 levels declared — check not applicable".into()],
                remediation: None,
            });
        } else {
            let mut reviewed = Vec::new();
            let mut unreviewed = Vec::new();
            for d in &high_risk {
                let id = d.frontmatter.id.clone().unwrap_or_default();
                if d.frontmatter.review_required == Some(true) {
                    reviewed.push(id);
                } else {
                    unreviewed.push(id);
                }
            }
            let status = if unreviewed.is_empty() {
                CheckStatus::Pass
            } else if !reviewed.is_empty() {
                CheckStatus::Partial
            } else {
                CheckStatus::Fail
            };
            checks.push(ComplianceCheck {
                id: "TC260-002".into(),
                description: "High / very-high / extremely-severe TC260 levels mandate review"
                    .into(),
                status,
                evidence: reviewed,
                remediation: if !unreviewed.is_empty() {
                    Some(format!(
                        "Set review_required: true on: {}",
                        unreviewed.join(", ")
                    ))
                } else {
                    None
                },
            });
        }
    }

    // TC260-003: TC260RA documents have all three grading criteria populated
    {
        let tc260_docs: Vec<&StrayMarkDocument> =
            docs.iter().filter(|d| d.doc_type == DocType::Tc260ra).collect();
        if tc260_docs.is_empty() {
            checks.push(ComplianceCheck {
                id: "TC260-003".into(),
                description: "TC260RA documents specify scenario × intelligence × scale".into(),
                status: CheckStatus::Fail,
                evidence: vec![],
                remediation: Some(
                    "Create a TC260RA document with the three grading criteria populated".into(),
                ),
            });
        } else {
            let mut complete = Vec::new();
            let mut incomplete = Vec::new();
            for d in &tc260_docs {
                let id = d.frontmatter.id.clone().unwrap_or_default();
                let ok = d.frontmatter.tc260_application_scenario.is_some()
                    && d.frontmatter.tc260_intelligence_level.is_some()
                    && d.frontmatter.tc260_application_scale.is_some();
                if ok {
                    complete.push(id);
                } else {
                    incomplete.push(id);
                }
            }
            let status = if incomplete.is_empty() {
                CheckStatus::Pass
            } else if !complete.is_empty() {
                CheckStatus::Partial
            } else {
                CheckStatus::Fail
            };
            checks.push(ComplianceCheck {
                id: "TC260-003".into(),
                description: "TC260RA documents specify scenario × intelligence × scale".into(),
                status,
                evidence: complete,
                remediation: if !incomplete.is_empty() {
                    Some(format!(
                        "Populate tc260_application_scenario / intelligence_level / application_scale on: {}",
                        incomplete.join(", ")
                    ))
                } else {
                    None
                },
            });
        }
    }

    let score = calculate_score(&checks);
    ComplianceReport {
        standard: Standard::ChinaTc260,
        standard_label: Standard::ChinaTc260.label().into(),
        checks,
        score,
    }
}

// ---------------------------------------------------------------------------
// PIPL / PIPIA checker
// ---------------------------------------------------------------------------

pub fn check_china_pipl(
    docs: &[StrayMarkDocument],
    _governance_dir: &Path,
) -> ComplianceReport {
    let mut checks = Vec::new();

    // PIPL-001: PIPIA exists when any document declares pipl_applicable: true
    {
        let need_pipia: Vec<&StrayMarkDocument> = docs
            .iter()
            .filter(|d| d.frontmatter.pipl_applicable == Some(true))
            .collect();
        if need_pipia.is_empty() {
            checks.push(ComplianceCheck {
                id: "PIPL-001".into(),
                description: "PIPIA exists when pipl_applicable is true".into(),
                status: CheckStatus::Pass,
                evidence: vec!["No PIPL-applicable documents — check not applicable".into()],
                remediation: None,
            });
        } else {
            let has_pipia = has_type(docs, DocType::Pipia);
            checks.push(ComplianceCheck {
                id: "PIPL-001".into(),
                description: "PIPIA exists when pipl_applicable is true".into(),
                status: if has_pipia {
                    CheckStatus::Pass
                } else {
                    CheckStatus::Fail
                },
                evidence: ids_of_type(docs, DocType::Pipia),
                remediation: if !has_pipia {
                    Some("Run 'straymark new pipia' to create the PIPL impact assessment".into())
                } else {
                    None
                },
            });
        }
    }

    // PIPL-002: Sensitive personal information requires a PIPIA + linkage
    {
        let need_link: Vec<&StrayMarkDocument> = docs
            .iter()
            .filter(|d| d.frontmatter.pipl_sensitive_data == Some(true))
            .filter(|d| d.doc_type != DocType::Pipia)
            .collect();
        if need_link.is_empty() {
            checks.push(ComplianceCheck {
                id: "PIPL-002".into(),
                description:
                    "Documents handling sensitive personal info link to a PIPIA via 'related'"
                        .into(),
                status: CheckStatus::Pass,
                evidence: vec!["No sensitive PIPL data declared — check not applicable".into()],
                remediation: None,
            });
        } else {
            let mut linked = Vec::new();
            let mut unlinked = Vec::new();
            for d in &need_link {
                let id = d.frontmatter.id.clone().unwrap_or_default();
                if related_includes_prefix(d, "PIPIA-") {
                    linked.push(id);
                } else {
                    unlinked.push(id);
                }
            }
            let status = if unlinked.is_empty() {
                CheckStatus::Pass
            } else if !linked.is_empty() {
                CheckStatus::Partial
            } else {
                CheckStatus::Fail
            };
            checks.push(ComplianceCheck {
                id: "PIPL-002".into(),
                description:
                    "Documents handling sensitive personal info link to a PIPIA via 'related'"
                        .into(),
                status,
                evidence: linked,
                remediation: if !unlinked.is_empty() {
                    Some(format!(
                        "Add a PIPIA-... entry to 'related' on: {}",
                        unlinked.join(", ")
                    ))
                } else {
                    None
                },
            });
        }
    }

    // PIPL-003: Cross-border transfer documents must have a PIPIA addressing it
    {
        let cb_docs: Vec<&StrayMarkDocument> = docs
            .iter()
            .filter(|d| d.frontmatter.pipl_cross_border_transfer == Some(true))
            .collect();
        if cb_docs.is_empty() {
            checks.push(ComplianceCheck {
                id: "PIPL-003".into(),
                description: "Cross-border personal info transfer is documented in a PIPIA".into(),
                status: CheckStatus::Pass,
                evidence: vec!["No cross-border PIPL transfers declared — check not applicable"
                    .into()],
                remediation: None,
            });
        } else {
            let pipia_with_cb: Vec<String> = docs
                .iter()
                .filter(|d| d.doc_type == DocType::Pipia)
                .filter(|d| d.frontmatter.pipl_cross_border_transfer == Some(true))
                .filter_map(|d| d.frontmatter.id.clone())
                .collect();
            let status = if pipia_with_cb.is_empty() {
                CheckStatus::Fail
            } else {
                CheckStatus::Pass
            };
            checks.push(ComplianceCheck {
                id: "PIPL-003".into(),
                description: "Cross-border personal info transfer is documented in a PIPIA".into(),
                status,
                evidence: pipia_with_cb,
                remediation: if status != CheckStatus::Pass {
                    Some(
                        "Create a PIPIA with pipl_cross_border_transfer: true and document the chosen transfer mechanism (security assessment / certification / standard contract)".into(),
                    )
                } else {
                    None
                },
            });
        }
    }

    // PIPL-004: PIPIA retention is at least 3 years from `created`
    {
        let pipia_docs: Vec<&StrayMarkDocument> =
            docs.iter().filter(|d| d.doc_type == DocType::Pipia).collect();
        if pipia_docs.is_empty() {
            checks.push(ComplianceCheck {
                id: "PIPL-004".into(),
                description: "PIPIA retention is ≥ 3 years per PIPL Art. 56".into(),
                status: CheckStatus::Pass,
                evidence: vec!["No PIPIA documents — check not applicable".into()],
                remediation: None,
            });
        } else {
            let mut compliant = Vec::new();
            let mut violating = Vec::new();
            for d in &pipia_docs {
                let id = d.frontmatter.id.clone().unwrap_or_default();
                if pipia_retention_ok(
                    d.frontmatter.created.as_deref(),
                    d.frontmatter.pipl_retention_until.as_deref(),
                ) {
                    compliant.push(id);
                } else {
                    violating.push(id);
                }
            }
            let status = if violating.is_empty() {
                CheckStatus::Pass
            } else if !compliant.is_empty() {
                CheckStatus::Partial
            } else {
                CheckStatus::Fail
            };
            checks.push(ComplianceCheck {
                id: "PIPL-004".into(),
                description: "PIPIA retention is ≥ 3 years per PIPL Art. 56".into(),
                status,
                evidence: compliant,
                remediation: if !violating.is_empty() {
                    Some(format!(
                        "Set pipl_retention_until ≥ created + 3 years on: {}",
                        violating.join(", ")
                    ))
                } else {
                    None
                },
            });
        }
    }

    let score = calculate_score(&checks);
    ComplianceReport {
        standard: Standard::ChinaPipl,
        standard_label: Standard::ChinaPipl.label().into(),
        checks,
        score,
    }
}

/// True when `until` is at least 3 years after `created` (both ISO YYYY-MM-DD).
fn pipia_retention_ok(created: Option<&str>, until: Option<&str>) -> bool {
    let (created, until) = match (created, until) {
        (Some(c), Some(u)) => (c, u),
        _ => return false,
    };
    let parse_year_month_day = |s: &str| -> Option<(i32, u32, u32)> {
        if s.len() < 10 {
            return None;
        }
        let y: i32 = s[..4].parse().ok()?;
        let m: u32 = s[5..7].parse().ok()?;
        let d: u32 = s[8..10].parse().ok()?;
        Some((y, m, d))
    };
    let (cy, cm, cd) = match parse_year_month_day(created) {
        Some(t) => t,
        None => return false,
    };
    let (uy, um, ud) = match parse_year_month_day(until) {
        Some(t) => t,
        None => return false,
    };
    let target = (cy + 3, cm, cd);
    (uy, um, ud) >= target
}

// ---------------------------------------------------------------------------
// GB 45438 checker
// ---------------------------------------------------------------------------

pub fn check_china_gb45438(
    docs: &[StrayMarkDocument],
    _governance_dir: &Path,
) -> ComplianceReport {
    let mut checks = Vec::new();

    // GB45438-001: AILABEL exists when any document declares gb45438_applicable: true
    {
        let need_label: Vec<&StrayMarkDocument> = docs
            .iter()
            .filter(|d| d.frontmatter.gb45438_applicable == Some(true))
            .filter(|d| d.doc_type != DocType::Ailabel)
            .collect();
        if need_label.is_empty() {
            checks.push(ComplianceCheck {
                id: "GB45438-001".into(),
                description:
                    "AILABEL exists when generative AI content labeling applies (GB 45438)".into(),
                status: CheckStatus::Pass,
                evidence: vec!["No documents declare gb45438_applicable — check not applicable"
                    .into()],
                remediation: None,
            });
        } else {
            let mut linked = Vec::new();
            let mut unlinked = Vec::new();
            for d in &need_label {
                let id = d.frontmatter.id.clone().unwrap_or_default();
                if related_includes_prefix(d, "AILABEL-") {
                    linked.push(id);
                } else {
                    unlinked.push(id);
                }
            }
            let any_ailabel = has_type(docs, DocType::Ailabel);
            let status = if unlinked.is_empty() {
                CheckStatus::Pass
            } else if any_ailabel {
                CheckStatus::Partial
            } else {
                CheckStatus::Fail
            };
            checks.push(ComplianceCheck {
                id: "GB45438-001".into(),
                description:
                    "AILABEL exists when generative AI content labeling applies (GB 45438)".into(),
                status,
                evidence: linked,
                remediation: if !unlinked.is_empty() {
                    Some(format!(
                        "Create an AILABEL and link via 'related' on: {}",
                        unlinked.join(", ")
                    ))
                } else {
                    None
                },
            });
        }
    }

    // GB45438-002: AILABEL declares both an explicit and an implicit labeling track
    {
        let ailabels: Vec<&StrayMarkDocument> = docs
            .iter()
            .filter(|d| d.doc_type == DocType::Ailabel)
            .collect();
        if ailabels.is_empty() {
            checks.push(ComplianceCheck {
                id: "GB45438-002".into(),
                description: "AILABEL declares both explicit and implicit label strategy".into(),
                status: CheckStatus::Pass,
                evidence: vec!["No AILABEL documents — check not applicable".into()],
                remediation: None,
            });
        } else {
            let mut complete = Vec::new();
            let mut partial = Vec::new();
            for d in &ailabels {
                let id = d.frontmatter.id.clone().unwrap_or_default();
                let has_explicit = d
                    .frontmatter
                    .gb45438_explicit_label_strategy
                    .as_deref()
                    .is_some_and(|v| !v.is_empty());
                let has_implicit = d
                    .frontmatter
                    .gb45438_implicit_metadata_format
                    .as_deref()
                    .is_some_and(|v| !v.is_empty() && v != "none");
                if has_explicit && has_implicit {
                    complete.push(id);
                } else {
                    partial.push(id);
                }
            }
            let status = if partial.is_empty() {
                CheckStatus::Pass
            } else if !complete.is_empty() {
                CheckStatus::Partial
            } else {
                CheckStatus::Fail
            };
            checks.push(ComplianceCheck {
                id: "GB45438-002".into(),
                description: "AILABEL declares both explicit and implicit label strategy".into(),
                status,
                evidence: complete,
                remediation: if !partial.is_empty() {
                    Some(format!(
                        "Populate gb45438_explicit_label_strategy and gb45438_implicit_metadata_format on: {}",
                        partial.join(", ")
                    ))
                } else {
                    None
                },
            });
        }
    }

    // GB45438-003: AILABEL covers at least one content type
    {
        let ailabels: Vec<&StrayMarkDocument> = docs
            .iter()
            .filter(|d| d.doc_type == DocType::Ailabel)
            .collect();
        if ailabels.is_empty() {
            checks.push(ComplianceCheck {
                id: "GB45438-003".into(),
                description: "AILABEL declares at least one content type covered".into(),
                status: CheckStatus::Pass,
                evidence: vec!["No AILABEL documents — check not applicable".into()],
                remediation: None,
            });
        } else {
            let mut ok = Vec::new();
            let mut empty = Vec::new();
            for d in &ailabels {
                let id = d.frontmatter.id.clone().unwrap_or_default();
                let count = d
                    .frontmatter
                    .gb45438_content_types
                    .as_ref()
                    .map(|v| v.len())
                    .unwrap_or(0);
                if count > 0 {
                    ok.push(id);
                } else {
                    empty.push(id);
                }
            }
            let status = if empty.is_empty() {
                CheckStatus::Pass
            } else if !ok.is_empty() {
                CheckStatus::Partial
            } else {
                CheckStatus::Fail
            };
            checks.push(ComplianceCheck {
                id: "GB45438-003".into(),
                description: "AILABEL declares at least one content type covered".into(),
                status,
                evidence: ok,
                remediation: if !empty.is_empty() {
                    Some(format!(
                        "Populate gb45438_content_types on: {}",
                        empty.join(", ")
                    ))
                } else {
                    None
                },
            });
        }
    }

    let score = calculate_score(&checks);
    ComplianceReport {
        standard: Standard::ChinaGb45438,
        standard_label: Standard::ChinaGb45438.label().into(),
        checks,
        score,
    }
}

// ---------------------------------------------------------------------------
// CAC algorithm filing checker
// ---------------------------------------------------------------------------

const CAC_APPROVED_STATUSES: &[&str] = &["provincial_approved", "national_approved"];

pub fn check_china_cac(
    docs: &[StrayMarkDocument],
    _governance_dir: &Path,
) -> ComplianceReport {
    let mut checks = Vec::new();

    // CAC-001: CACFILE exists when any document declares cac_filing_required: true
    {
        let need_filing: Vec<&StrayMarkDocument> = docs
            .iter()
            .filter(|d| d.frontmatter.cac_filing_required == Some(true))
            .filter(|d| d.doc_type != DocType::Cacfile)
            .collect();
        if need_filing.is_empty() {
            checks.push(ComplianceCheck {
                id: "CAC-001".into(),
                description: "CACFILE exists when CAC algorithm filing is required".into(),
                status: CheckStatus::Pass,
                evidence: vec!["No documents declare cac_filing_required — check not applicable"
                    .into()],
                remediation: None,
            });
        } else {
            let mut linked = Vec::new();
            let mut unlinked = Vec::new();
            for d in &need_filing {
                let id = d.frontmatter.id.clone().unwrap_or_default();
                if related_includes_prefix(d, "CACFILE-") {
                    linked.push(id);
                } else {
                    unlinked.push(id);
                }
            }
            let status = if unlinked.is_empty() {
                CheckStatus::Pass
            } else if !linked.is_empty() {
                CheckStatus::Partial
            } else {
                CheckStatus::Fail
            };
            checks.push(ComplianceCheck {
                id: "CAC-001".into(),
                description: "CACFILE exists when CAC algorithm filing is required".into(),
                status,
                evidence: linked,
                remediation: if !unlinked.is_empty() {
                    Some(format!(
                        "Create a CACFILE and link via 'related' on: {}",
                        unlinked.join(", ")
                    ))
                } else {
                    None
                },
            });
        }
    }

    // CAC-002: CACFILE has a status set (not silently undecided)
    {
        let cacfiles: Vec<&StrayMarkDocument> = docs
            .iter()
            .filter(|d| d.doc_type == DocType::Cacfile)
            .collect();
        if cacfiles.is_empty() {
            checks.push(ComplianceCheck {
                id: "CAC-002".into(),
                description: "CACFILE documents have an explicit cac_filing_status".into(),
                status: CheckStatus::Pass,
                evidence: vec!["No CACFILE documents — check not applicable".into()],
                remediation: None,
            });
        } else {
            let mut with_status = Vec::new();
            let mut missing = Vec::new();
            for d in &cacfiles {
                let id = d.frontmatter.id.clone().unwrap_or_default();
                if d.frontmatter.cac_filing_status.is_some() {
                    with_status.push(id);
                } else {
                    missing.push(id);
                }
            }
            let status = if missing.is_empty() {
                CheckStatus::Pass
            } else if !with_status.is_empty() {
                CheckStatus::Partial
            } else {
                CheckStatus::Fail
            };
            checks.push(ComplianceCheck {
                id: "CAC-002".into(),
                description: "CACFILE documents have an explicit cac_filing_status".into(),
                status,
                evidence: with_status,
                remediation: if !missing.is_empty() {
                    Some(format!(
                        "Set cac_filing_status on: {}",
                        missing.join(", ")
                    ))
                } else {
                    None
                },
            });
        }
    }

    // CAC-003: Approved status implies cac_filing_number is populated
    {
        let approved: Vec<&StrayMarkDocument> = docs
            .iter()
            .filter(|d| {
                d.frontmatter
                    .cac_filing_status
                    .as_deref()
                    .is_some_and(|s| CAC_APPROVED_STATUSES.contains(&s))
            })
            .collect();
        if approved.is_empty() {
            checks.push(ComplianceCheck {
                id: "CAC-003".into(),
                description: "Approved CAC filings have a cac_filing_number".into(),
                status: CheckStatus::Pass,
                evidence: vec!["No approved CAC filings yet — check not applicable".into()],
                remediation: None,
            });
        } else {
            let mut ok = Vec::new();
            let mut missing_number = Vec::new();
            for d in &approved {
                let id = d.frontmatter.id.clone().unwrap_or_default();
                if d.frontmatter
                    .cac_filing_number
                    .as_deref()
                    .is_some_and(|n| !n.is_empty())
                {
                    ok.push(id);
                } else {
                    missing_number.push(id);
                }
            }
            let status = if missing_number.is_empty() {
                CheckStatus::Pass
            } else if !ok.is_empty() {
                CheckStatus::Partial
            } else {
                CheckStatus::Fail
            };
            checks.push(ComplianceCheck {
                id: "CAC-003".into(),
                description: "Approved CAC filings have a cac_filing_number".into(),
                status,
                evidence: ok,
                remediation: if !missing_number.is_empty() {
                    Some(format!(
                        "Populate cac_filing_number on: {}",
                        missing_number.join(", ")
                    ))
                } else {
                    None
                },
            });
        }
    }

    let score = calculate_score(&checks);
    ComplianceReport {
        standard: Standard::ChinaCac,
        standard_label: Standard::ChinaCac.label().into(),
        checks,
        score,
    }
}

// ---------------------------------------------------------------------------
// GB/T 45652 training data security checker
// ---------------------------------------------------------------------------

pub fn check_china_gb45652(
    docs: &[StrayMarkDocument],
    _governance_dir: &Path,
) -> ComplianceReport {
    let mut checks = Vec::new();

    // GB45652-001: At least one SBOM declares training-data compliance
    {
        let sbom_docs: Vec<&StrayMarkDocument> = docs
            .iter()
            .filter(|d| d.doc_type == DocType::Sbom)
            .collect();
        if sbom_docs.is_empty() {
            checks.push(ComplianceCheck {
                id: "GB45652-001".into(),
                description: "Training-data security declared on at least one SBOM".into(),
                status: CheckStatus::Fail,
                evidence: vec![],
                remediation: Some(
                    "Create an SBOM and set gb45652_training_data_compliance: true after documenting the GB/T 45652 section"
                        .into(),
                ),
            });
        } else {
            let compliant: Vec<String> = sbom_docs
                .iter()
                .filter(|d| d.frontmatter.gb45652_training_data_compliance == Some(true))
                .filter_map(|d| d.frontmatter.id.clone())
                .collect();
            let status = if compliant.is_empty() {
                CheckStatus::Fail
            } else if compliant.len() == sbom_docs.len() {
                CheckStatus::Pass
            } else {
                CheckStatus::Partial
            };
            checks.push(ComplianceCheck {
                id: "GB45652-001".into(),
                description: "Training-data security declared on at least one SBOM".into(),
                status,
                evidence: compliant,
                remediation: if status != CheckStatus::Pass {
                    Some(
                        "Set gb45652_training_data_compliance: true on each SBOM after completing the GB/T 45652 section"
                            .into(),
                    )
                } else {
                    None
                },
            });
        }
    }

    // GB45652-002: MCARD references training-data compliance when applicable
    {
        let mcards: Vec<&StrayMarkDocument> = docs
            .iter()
            .filter(|d| d.doc_type == DocType::Mcard)
            .collect();
        if mcards.is_empty() {
            checks.push(ComplianceCheck {
                id: "GB45652-002".into(),
                description: "MCARD declares gb45652_training_data_compliance for in-scope models"
                    .into(),
                status: CheckStatus::Pass,
                evidence: vec!["No MCARD documents — check not applicable".into()],
                remediation: None,
            });
        } else {
            let compliant: Vec<String> = mcards
                .iter()
                .filter(|d| d.frontmatter.gb45652_training_data_compliance == Some(true))
                .filter_map(|d| d.frontmatter.id.clone())
                .collect();
            let status = if compliant.is_empty() {
                CheckStatus::Fail
            } else if compliant.len() == mcards.len() {
                CheckStatus::Pass
            } else {
                CheckStatus::Partial
            };
            checks.push(ComplianceCheck {
                id: "GB45652-002".into(),
                description: "MCARD declares gb45652_training_data_compliance for in-scope models"
                    .into(),
                status,
                evidence: compliant,
                remediation: if status != CheckStatus::Pass {
                    Some(
                        "Set gb45652_training_data_compliance: true on each MCARD with documented training-data security"
                            .into(),
                    )
                } else {
                    None
                },
            });
        }
    }

    let score = calculate_score(&checks);
    ComplianceReport {
        standard: Standard::ChinaGb45652,
        standard_label: Standard::ChinaGb45652.label().into(),
        checks,
        score,
    }
}

// ---------------------------------------------------------------------------
// CSL 2026 incident reporting checker
// ---------------------------------------------------------------------------

pub fn check_china_csl(
    docs: &[StrayMarkDocument],
    _governance_dir: &Path,
) -> ComplianceReport {
    let mut checks = Vec::new();
    let inc_docs: Vec<&StrayMarkDocument> =
        docs.iter().filter(|d| d.doc_type == DocType::Inc).collect();

    // CSL-001: Every INC has csl_severity_level populated
    if inc_docs.is_empty() {
        checks.push(ComplianceCheck {
            id: "CSL-001".into(),
            description: "INC documents declare csl_severity_level".into(),
            status: CheckStatus::Pass,
            evidence: vec!["No incidents recorded — check not applicable".into()],
            remediation: None,
        });
    } else {
        let mut with_sev = Vec::new();
        let mut missing = Vec::new();
        for d in &inc_docs {
            let id = d.frontmatter.id.clone().unwrap_or_default();
            if d.frontmatter.csl_severity_level.is_some() {
                with_sev.push(id);
            } else {
                missing.push(id);
            }
        }
        let status = if missing.is_empty() {
            CheckStatus::Pass
        } else if !with_sev.is_empty() {
            CheckStatus::Partial
        } else {
            CheckStatus::Fail
        };
        checks.push(ComplianceCheck {
            id: "CSL-001".into(),
            description: "INC documents declare csl_severity_level".into(),
            status,
            evidence: with_sev,
            remediation: if !missing.is_empty() {
                Some(format!(
                    "Populate csl_severity_level (or 'not_applicable') on: {}",
                    missing.join(", ")
                ))
            } else {
                None
            },
        });
    }

    // CSL-002: Severity ↔ deadline coherence
    if inc_docs.is_empty() {
        checks.push(ComplianceCheck {
            id: "CSL-002".into(),
            description: "csl_report_deadline_hours matches csl_severity_level (1h / 4h / 24h)"
                .into(),
            status: CheckStatus::Pass,
            evidence: vec!["No incidents recorded — check not applicable".into()],
            remediation: None,
        });
    } else {
        let mut coherent = Vec::new();
        let mut incoherent = Vec::new();
        for d in &inc_docs {
            let id = d.frontmatter.id.clone().unwrap_or_default();
            let sev = d.frontmatter.csl_severity_level.as_deref();
            let hours = d.frontmatter.csl_report_deadline_hours;
            let ok = match (sev, hours) {
                (Some("particularly_serious"), Some(1)) => true,
                (Some("relatively_major"), Some(4)) => true,
                (Some("major"), Some(h)) if h <= 24 => true,
                (Some("general"), _) => true,
                (Some("not_applicable") | None, _) => true,
                _ => false,
            };
            if ok {
                coherent.push(id);
            } else {
                incoherent.push(id);
            }
        }
        let status = if incoherent.is_empty() {
            CheckStatus::Pass
        } else if !coherent.is_empty() {
            CheckStatus::Partial
        } else {
            CheckStatus::Fail
        };
        checks.push(ComplianceCheck {
            id: "CSL-002".into(),
            description: "csl_report_deadline_hours matches csl_severity_level (1h / 4h / 24h)"
                .into(),
            status,
            evidence: coherent,
            remediation: if !incoherent.is_empty() {
                Some(format!(
                    "particularly_serious → 1h, relatively_major → 4h. Incoherent on: {}",
                    incoherent.join(", ")
                ))
            } else {
                None
            },
        });
    }

    // CSL-003: Major+ incidents have a documented 30-day post-mortem (resolved_date present)
    {
        let major_or_above: Vec<&StrayMarkDocument> = inc_docs
            .iter()
            .filter(|d| {
                d.frontmatter
                    .csl_severity_level
                    .as_deref()
                    .is_some_and(|s| matches!(s, "particularly_serious" | "relatively_major" | "major"))
            })
            .copied()
            .collect();
        if major_or_above.is_empty() {
            checks.push(ComplianceCheck {
                id: "CSL-003".into(),
                description: "Major+ incidents have a 30-day post-mortem documented".into(),
                status: CheckStatus::Pass,
                evidence: vec!["No major+ incidents — check not applicable".into()],
                remediation: None,
            });
        } else {
            // We use status=accepted as a proxy for "post-mortem closed".
            let mut closed = Vec::new();
            let mut open = Vec::new();
            for d in &major_or_above {
                let id = d.frontmatter.id.clone().unwrap_or_default();
                let is_closed = d
                    .frontmatter
                    .status
                    .as_deref()
                    .is_some_and(|s| s == "accepted" || s == "superseded");
                if is_closed {
                    closed.push(id);
                } else {
                    open.push(id);
                }
            }
            let status = if open.is_empty() {
                CheckStatus::Pass
            } else if !closed.is_empty() {
                CheckStatus::Partial
            } else {
                CheckStatus::Fail
            };
            checks.push(ComplianceCheck {
                id: "CSL-003".into(),
                description: "Major+ incidents have a 30-day post-mortem documented".into(),
                status,
                evidence: closed,
                remediation: if !open.is_empty() {
                    Some(format!(
                        "Close the post-mortem (status: accepted) within 30 days on: {}",
                        open.join(", ")
                    ))
                } else {
                    None
                },
            });
        }
    }

    let score = calculate_score(&checks);
    ComplianceReport {
        standard: Standard::ChinaCsl,
        standard_label: Standard::ChinaCsl.label().into(),
        checks,
        score,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::Frontmatter;
    use std::path::PathBuf;

    fn make_doc(
        filename: &str,
        doc_type: DocType,
        fm: Frontmatter,
        body: &str,
    ) -> StrayMarkDocument {
        StrayMarkDocument {
            path: PathBuf::from(format!(".straymark/test/{}", filename)),
            filename: filename.to_string(),
            doc_type,
            frontmatter: fm,
            body: body.to_string(),
        }
    }

    #[test]
    fn test_eu_ai_act_empty_docs() {
        let report = check_eu_ai_act(&[], &PathBuf::from("/tmp"));
        assert_eq!(report.standard, Standard::EuAiAct);
        // EU-001 should fail with no docs
        assert_eq!(report.checks[0].status, CheckStatus::Fail);
    }

    #[test]
    fn test_eu_ai_act_with_classified_doc() {
        let docs = vec![make_doc(
            "AILOG-2026-01-01-001-test.md",
            DocType::Ailog,
            Frontmatter {
                id: Some("AILOG-2026-01-01-001".into()),
                eu_ai_act_risk: Some("high".into()),
                ..Default::default()
            },
            "",
        )];
        let report = check_eu_ai_act(&docs, &PathBuf::from("/tmp"));
        assert_eq!(report.checks[0].status, CheckStatus::Pass);
    }

    #[test]
    fn test_iso_42001_partial() {
        let docs = vec![make_doc(
            "ETH-2026-01-01-001-review.md",
            DocType::Eth,
            Frontmatter {
                id: Some("ETH-2026-01-01-001".into()),
                ..Default::default()
            },
            "",
        )];
        let report = check_iso_42001(&docs, &PathBuf::from("/tmp"));
        // ISO-001 should fail (no governance dir)
        assert_eq!(report.checks[0].status, CheckStatus::Fail);
        // ISO-002 should pass (ETH exists)
        assert_eq!(report.checks[1].status, CheckStatus::Pass);
    }

    #[test]
    fn test_nist_genai_coverage() {
        let docs = vec![make_doc(
            "ETH-2026-01-01-001-review.md",
            DocType::Eth,
            Frontmatter {
                nist_genai_risks: Some(vec!["bias".into(), "privacy".into()]),
                ..Default::default()
            },
            "",
        )];
        let report = check_nist_ai_rmf(&docs, &PathBuf::from("/tmp"));
        // NIST-GENAI-001 should be partial (2/12)
        let genai_check = report.checks.iter().find(|c| c.id == "NIST-GENAI-001").unwrap();
        assert_eq!(genai_check.status, CheckStatus::Partial);
        assert_eq!(genai_check.evidence.len(), 2);
    }

    #[test]
    fn test_calculate_score() {
        let checks = vec![
            ComplianceCheck {
                id: "T-001".into(),
                description: "test".into(),
                status: CheckStatus::Pass,
                evidence: vec![],
                remediation: None,
            },
            ComplianceCheck {
                id: "T-002".into(),
                description: "test".into(),
                status: CheckStatus::Fail,
                evidence: vec![],
                remediation: None,
            },
        ];
        let score = calculate_score(&checks);
        assert!((score - 50.0).abs() < 0.01);
    }

    // ---------- China checkers ----------

    #[test]
    fn test_china_tc260_no_docs_fails() {
        let report = check_china_tc260(&[], &PathBuf::from("/tmp"));
        assert_eq!(report.standard, Standard::ChinaTc260);
        // TC260-001 fails when no TC260RA exists
        let tc001 = report.checks.iter().find(|c| c.id == "TC260-001").unwrap();
        assert_eq!(tc001.status, CheckStatus::Fail);
    }

    #[test]
    fn test_china_tc260_complete() {
        let docs = vec![make_doc(
            "TC260RA-2026-04-25-001-test.md",
            DocType::Tc260ra,
            Frontmatter {
                id: Some("TC260RA-2026-04-25-001".into()),
                review_required: Some(true),
                tc260_risk_level: Some("high".into()),
                tc260_application_scenario: Some("healthcare".into()),
                tc260_intelligence_level: Some("foundation".into()),
                tc260_application_scale: Some("organization".into()),
                ..Default::default()
            },
            "",
        )];
        let report = check_china_tc260(&docs, &PathBuf::from("/tmp"));
        assert!((report.score - 100.0).abs() < 0.01);
    }

    #[test]
    fn test_china_pipl_sensitive_data_requires_pipia() {
        let docs = vec![make_doc(
            "MCARD-2026-04-25-001-test.md",
            DocType::Mcard,
            Frontmatter {
                id: Some("MCARD-2026-04-25-001".into()),
                pipl_applicable: Some(true),
                pipl_sensitive_data: Some(true),
                ..Default::default()
            },
            "",
        )];
        let report = check_china_pipl(&docs, &PathBuf::from("/tmp"));
        let pipl001 = report.checks.iter().find(|c| c.id == "PIPL-001").unwrap();
        assert_eq!(pipl001.status, CheckStatus::Fail);
    }

    #[test]
    fn test_china_pipl_retention_under_three_years() {
        let docs = vec![make_doc(
            "PIPIA-2026-04-25-001-test.md",
            DocType::Pipia,
            Frontmatter {
                id: Some("PIPIA-2026-04-25-001".into()),
                created: Some("2026-04-25".into()),
                pipl_retention_until: Some("2027-04-25".into()),
                ..Default::default()
            },
            "",
        )];
        let report = check_china_pipl(&docs, &PathBuf::from("/tmp"));
        let pipl004 = report.checks.iter().find(|c| c.id == "PIPL-004").unwrap();
        assert_eq!(pipl004.status, CheckStatus::Fail);
    }

    #[test]
    fn test_china_gb45438_requires_both_tracks() {
        let docs = vec![make_doc(
            "AILABEL-2026-04-25-001-test.md",
            DocType::Ailabel,
            Frontmatter {
                id: Some("AILABEL-2026-04-25-001".into()),
                gb45438_explicit_label_strategy: Some("watermark".into()),
                gb45438_implicit_metadata_format: Some("none".into()),
                gb45438_content_types: Some(vec!["image".into()]),
                ..Default::default()
            },
            "",
        )];
        let report = check_china_gb45438(&docs, &PathBuf::from("/tmp"));
        let g002 = report.checks.iter().find(|c| c.id == "GB45438-002").unwrap();
        assert_eq!(g002.status, CheckStatus::Fail);
    }

    #[test]
    fn test_china_cac_approved_without_number_fails() {
        let docs = vec![make_doc(
            "CACFILE-2026-04-25-001-test.md",
            DocType::Cacfile,
            Frontmatter {
                id: Some("CACFILE-2026-04-25-001".into()),
                cac_filing_status: Some("national_approved".into()),
                cac_filing_number: None,
                ..Default::default()
            },
            "",
        )];
        let report = check_china_cac(&docs, &PathBuf::from("/tmp"));
        let cac003 = report.checks.iter().find(|c| c.id == "CAC-003").unwrap();
        assert_eq!(cac003.status, CheckStatus::Fail);
    }

    #[test]
    fn test_china_csl_severity_deadline_coherence() {
        let docs = vec![make_doc(
            "INC-2026-04-25-001-test.md",
            DocType::Inc,
            Frontmatter {
                id: Some("INC-2026-04-25-001".into()),
                csl_severity_level: Some("particularly_serious".into()),
                csl_report_deadline_hours: Some(4),
                ..Default::default()
            },
            "",
        )];
        let report = check_china_csl(&docs, &PathBuf::from("/tmp"));
        let csl002 = report.checks.iter().find(|c| c.id == "CSL-002").unwrap();
        assert_eq!(csl002.status, CheckStatus::Fail);
    }

    #[test]
    fn test_china_gb45652_requires_sbom_compliance() {
        let docs = vec![make_doc(
            "SBOM-2026-04-25-001-test.md",
            DocType::Sbom,
            Frontmatter {
                id: Some("SBOM-2026-04-25-001".into()),
                gb45652_training_data_compliance: Some(false),
                ..Default::default()
            },
            "",
        )];
        let report = check_china_gb45652(&docs, &PathBuf::from("/tmp"));
        let g001 = report.checks.iter().find(|c| c.id == "GB45652-001").unwrap();
        assert_eq!(g001.status, CheckStatus::Fail);
    }

    #[test]
    fn test_pipia_retention_helper_basic() {
        assert!(pipia_retention_ok(Some("2026-04-25"), Some("2029-04-25")));
        assert!(pipia_retention_ok(Some("2026-04-25"), Some("2030-01-01")));
        assert!(!pipia_retention_ok(Some("2026-04-25"), Some("2027-04-25")));
        assert!(!pipia_retention_ok(None, Some("2030-01-01")));
        assert!(!pipia_retention_ok(Some("2026-04-25"), None));
    }
}
