use chrono::NaiveDate;
use serde::Serialize;

use straymark_core::document::{StrayMarkDocument, DocType};

/// Time period for metrics calculation
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum Period {
    Last7Days,
    Last30Days,
    Last90Days,
    All,
}

impl Period {
    pub fn label(&self) -> &'static str {
        match self {
            Period::Last7Days => "Last 7 days",
            Period::Last30Days => "Last 30 days",
            Period::Last90Days => "Last 90 days",
            Period::All => "All time",
        }
    }

    pub fn days(&self) -> Option<i64> {
        match self {
            Period::Last7Days => Some(7),
            Period::Last30Days => Some(30),
            Period::Last90Days => Some(90),
            Period::All => None,
        }
    }

    pub fn from_str(s: &str) -> Period {
        match s {
            "last-7-days" => Period::Last7Days,
            "last-30-days" => Period::Last30Days,
            "last-90-days" => Period::Last90Days,
            "all" => Period::All,
            _ => Period::Last30Days,
        }
    }
}

/// Direction of a trend
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum TrendDirection {
    Up,
    Down,
    Stable,
}

impl TrendDirection {
    pub fn symbol(&self) -> &'static str {
        match self {
            TrendDirection::Up => "↑",
            TrendDirection::Down => "↓",
            TrendDirection::Stable => "→",
        }
    }
}

/// A single trend comparison
#[derive(Debug, Clone, Serialize)]
pub struct Trend {
    pub metric: String,
    pub current: usize,
    pub previous: usize,
    pub direction: TrendDirection,
}

/// Review compliance stats
#[derive(Debug, Clone, Serialize)]
pub struct ReviewCompliance {
    pub total_requiring_review: usize,
    pub reviewed: usize,
    pub rate: f64,
}

/// Complete metrics report
#[derive(Debug, Serialize)]
pub struct MetricsReport {
    pub period: Period,
    pub period_label: String,
    pub period_start: String,
    pub period_end: String,
    pub doc_counts: Vec<(String, usize)>,
    pub total_docs: usize,
    pub review_compliance: ReviewCompliance,
    pub risk_distribution: Vec<(String, usize)>,
    pub agent_activity: Vec<(String, usize)>,
    pub trends: Vec<Trend>,
}

/// Parse a date string in YYYY-MM-DD format
fn parse_date(s: &str) -> Option<NaiveDate> {
    NaiveDate::parse_from_str(s, "%Y-%m-%d").ok()
}

/// Check if a document falls within a date range
fn is_in_range(doc: &StrayMarkDocument, start: NaiveDate, end: NaiveDate) -> bool {
    doc.frontmatter
        .created
        .as_deref()
        .and_then(parse_date)
        .is_some_and(|d| d >= start && d <= end)
}

/// Filter documents by date range
fn filter_by_range(docs: &[StrayMarkDocument], start: NaiveDate, end: NaiveDate) -> Vec<&StrayMarkDocument> {
    docs.iter().filter(|d| is_in_range(d, start, end)).collect()
}

/// Calculate metrics for a set of documents.
/// `now` is injectable for testing determinism.
pub fn calculate_metrics(
    docs: &[StrayMarkDocument],
    period: Period,
    now: NaiveDate,
) -> MetricsReport {
    // Determine date range
    let (start, end) = match period.days() {
        Some(days) => {
            let start = now - chrono::Duration::days(days);
            (start, now)
        }
        None => {
            // All time: find earliest doc date, or use a very old date
            let earliest = docs
                .iter()
                .filter_map(|d| d.frontmatter.created.as_deref())
                .filter_map(parse_date)
                .min()
                .unwrap_or(now);
            (earliest, now)
        }
    };

    let filtered = filter_by_range(docs, start, end);

    // Document counts by type (16 types: 12 base + 4 China-specific)
    let type_names = [
        "AILOG", "AIDEC", "ADR", "ETH", "REQ", "TES", "INC", "TDE", "SEC", "MCARD", "SBOM",
        "DPIA",
        // China regulatory artifacts (only present when regional_scope: china)
        "PIPIA", "CACFILE", "TC260RA", "AILABEL",
    ];
    let doc_counts: Vec<(String, usize)> = type_names
        .iter()
        .map(|name| {
            let dt = DocType::from_prefix(name).unwrap();
            let count = filtered.iter().filter(|d| d.doc_type == dt).count();
            (name.to_string(), count)
        })
        .collect();

    let total_docs = filtered.len();

    // Review compliance
    let requiring_review: Vec<&&StrayMarkDocument> = filtered
        .iter()
        .filter(|d| d.frontmatter.review_required == Some(true))
        .collect();
    let reviewed = requiring_review
        .iter()
        .filter(|d| {
            d.frontmatter
                .status
                .as_deref()
                .is_some_and(|s| s == "accepted" || s == "superseded")
        })
        .count();
    let review_rate = if requiring_review.is_empty() {
        100.0
    } else {
        (reviewed as f64 / requiring_review.len() as f64) * 100.0
    };

    let review_compliance = ReviewCompliance {
        total_requiring_review: requiring_review.len(),
        reviewed,
        rate: review_rate,
    };

    // Risk distribution
    let risk_levels = ["low", "medium", "high", "critical"];
    let risk_distribution: Vec<(String, usize)> = risk_levels
        .iter()
        .map(|level| {
            let count = filtered
                .iter()
                .filter(|d| d.frontmatter.risk_level.as_deref() == Some(level))
                .count();
            (level.to_string(), count)
        })
        .collect();

    // Agent activity
    let mut agent_map: std::collections::BTreeMap<String, usize> = std::collections::BTreeMap::new();
    for doc in &filtered {
        if let Some(agent) = &doc.frontmatter.agent {
            *agent_map.entry(agent.clone()).or_insert(0) += 1;
        }
    }
    let agent_activity: Vec<(String, usize)> = agent_map.into_iter().collect();

    // Trends: compare current vs previous period
    let trends = if let Some(days) = period.days() {
        let prev_end = start - chrono::Duration::days(1);
        let prev_start = prev_end - chrono::Duration::days(days);
        let prev_filtered = filter_by_range(docs, prev_start, prev_end);

        let current_total = filtered.len();
        let prev_total = prev_filtered.len();

        let mut trends = vec![Trend {
            metric: "Total documents".into(),
            current: current_total,
            previous: prev_total,
            direction: trend_direction(current_total, prev_total),
        }];

        // Review completion trend
        let prev_requiring: Vec<&&StrayMarkDocument> = prev_filtered
            .iter()
            .filter(|d| d.frontmatter.review_required == Some(true))
            .collect();
        let prev_reviewed = prev_requiring
            .iter()
            .filter(|d| {
                d.frontmatter
                    .status
                    .as_deref()
                    .is_some_and(|s| s == "accepted" || s == "superseded")
            })
            .count();

        trends.push(Trend {
            metric: "Reviews completed".into(),
            current: reviewed,
            previous: prev_reviewed,
            direction: trend_direction(reviewed, prev_reviewed),
        });

        // High risk docs trend
        let current_high = filtered
            .iter()
            .filter(|d| {
                d.frontmatter
                    .risk_level
                    .as_deref()
                    .is_some_and(|r| r == "high" || r == "critical")
            })
            .count();
        let prev_high = prev_filtered
            .iter()
            .filter(|d| {
                d.frontmatter
                    .risk_level
                    .as_deref()
                    .is_some_and(|r| r == "high" || r == "critical")
            })
            .count();

        trends.push(Trend {
            metric: "High/critical risk".into(),
            current: current_high,
            previous: prev_high,
            direction: trend_direction(current_high, prev_high),
        });

        trends
    } else {
        vec![]
    };

    MetricsReport {
        period,
        period_label: period.label().into(),
        period_start: start.format("%Y-%m-%d").to_string(),
        period_end: end.format("%Y-%m-%d").to_string(),
        doc_counts,
        total_docs,
        review_compliance,
        risk_distribution,
        agent_activity,
        trends,
    }
}

fn trend_direction(current: usize, previous: usize) -> TrendDirection {
    if current > previous {
        TrendDirection::Up
    } else if current < previous {
        TrendDirection::Down
    } else {
        TrendDirection::Stable
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use straymark_core::document::Frontmatter;
    use std::path::PathBuf;

    fn make_doc(
        filename: &str,
        doc_type: DocType,
        fm: Frontmatter,
    ) -> StrayMarkDocument {
        StrayMarkDocument {
            path: PathBuf::from(format!(".straymark/test/{}", filename)),
            filename: filename.to_string(),
            doc_type,
            frontmatter: fm,
            body: String::new(),
        }
    }

    #[test]
    fn test_empty_docs() {
        let now = NaiveDate::from_ymd_opt(2026, 3, 26).unwrap();
        let report = calculate_metrics(&[], Period::Last30Days, now);
        assert_eq!(report.total_docs, 0);
        assert_eq!(report.review_compliance.rate, 100.0);
    }

    #[test]
    fn test_doc_counts() {
        let now = NaiveDate::from_ymd_opt(2026, 3, 26).unwrap();
        let docs = vec![
            make_doc("AILOG-2026-03-20-001-test.md", DocType::Ailog, Frontmatter {
                created: Some("2026-03-20".into()),
                ..Default::default()
            }),
            make_doc("AILOG-2026-03-21-002-test2.md", DocType::Ailog, Frontmatter {
                created: Some("2026-03-21".into()),
                ..Default::default()
            }),
            make_doc("ETH-2026-03-22-001-review.md", DocType::Eth, Frontmatter {
                created: Some("2026-03-22".into()),
                ..Default::default()
            }),
        ];
        let report = calculate_metrics(&docs, Period::Last30Days, now);
        assert_eq!(report.total_docs, 3);
        let ailog_count = report.doc_counts.iter().find(|(t, _)| t == "AILOG").unwrap().1;
        assert_eq!(ailog_count, 2);
    }

    #[test]
    fn test_review_compliance() {
        let now = NaiveDate::from_ymd_opt(2026, 3, 26).unwrap();
        let docs = vec![
            make_doc("ETH-2026-03-20-001-review.md", DocType::Eth, Frontmatter {
                created: Some("2026-03-20".into()),
                review_required: Some(true),
                status: Some("accepted".into()),
                ..Default::default()
            }),
            make_doc("SEC-2026-03-21-001-assess.md", DocType::Sec, Frontmatter {
                created: Some("2026-03-21".into()),
                review_required: Some(true),
                status: Some("draft".into()),
                ..Default::default()
            }),
        ];
        let report = calculate_metrics(&docs, Period::Last30Days, now);
        assert_eq!(report.review_compliance.total_requiring_review, 2);
        assert_eq!(report.review_compliance.reviewed, 1);
        assert!((report.review_compliance.rate - 50.0).abs() < 0.01);
    }

    #[test]
    fn test_period_filtering() {
        let now = NaiveDate::from_ymd_opt(2026, 3, 26).unwrap();
        let docs = vec![
            make_doc("AILOG-2026-03-25-001-recent.md", DocType::Ailog, Frontmatter {
                created: Some("2026-03-25".into()),
                ..Default::default()
            }),
            make_doc("AILOG-2025-01-01-001-old.md", DocType::Ailog, Frontmatter {
                created: Some("2025-01-01".into()),
                ..Default::default()
            }),
        ];
        let report = calculate_metrics(&docs, Period::Last7Days, now);
        assert_eq!(report.total_docs, 1);
    }

    #[test]
    fn test_trend_direction() {
        assert_eq!(trend_direction(5, 3), TrendDirection::Up);
        assert_eq!(trend_direction(2, 4), TrendDirection::Down);
        assert_eq!(trend_direction(3, 3), TrendDirection::Stable);
    }
}
