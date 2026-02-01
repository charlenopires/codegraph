//! Reporter - Generate benchmark reports in multiple formats
//!
//! Supports Markdown, JSON, and HTML output formats.

use std::path::Path;

use crate::error::Result;
use crate::models::AggregateMetrics;
use crate::runner::BenchmarkComparison;

/// Report format
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportFormat {
    /// Markdown format for documentation
    Markdown,
    /// JSON format for programmatic processing
    Json,
    /// HTML format for web viewing
    Html,
}

/// Benchmark report generator
pub struct Reporter;

impl Reporter {
    /// Generate a report for a single system
    pub fn single_report(metrics: &AggregateMetrics, format: ReportFormat) -> Result<String> {
        match format {
            ReportFormat::Markdown => Self::single_markdown(metrics),
            ReportFormat::Json => Self::single_json(metrics),
            ReportFormat::Html => Self::single_html(metrics),
        }
    }

    /// Generate a comparison report
    pub fn comparison_report(comparison: &BenchmarkComparison, format: ReportFormat) -> Result<String> {
        match format {
            ReportFormat::Markdown => Self::comparison_markdown(comparison),
            ReportFormat::Json => Self::comparison_json(comparison),
            ReportFormat::Html => Self::comparison_html(comparison),
        }
    }

    /// Save report to file
    pub fn save_report(content: &str, path: &Path) -> Result<()> {
        std::fs::write(path, content)?;
        Ok(())
    }

    fn single_markdown(metrics: &AggregateMetrics) -> Result<String> {
        let mut report = String::new();

        report.push_str(&format!("# Benchmark Report: {}\n\n", metrics.system_name));
        report.push_str(&format!("**Queries Executed:** {}\n\n", metrics.query_count));

        report.push_str("## Quality Metrics\n\n");
        report.push_str("| Metric | Value |\n");
        report.push_str("|--------|-------|\n");
        report.push_str(&format!("| Precision | {:.3} |\n", metrics.avg_precision));
        report.push_str(&format!("| Recall | {:.3} |\n", metrics.avg_recall));
        report.push_str(&format!("| F1 Score | {:.3} |\n", metrics.avg_f1_score));
        report.push_str(&format!("| Hallucination Rate | {:.3} |\n", metrics.avg_hallucination_rate));

        report.push_str("\n## Latency Metrics\n\n");
        report.push_str("| Percentile | Latency (ms) |\n");
        report.push_str("|------------|-------------|\n");
        report.push_str(&format!("| P50 | {:.2} |\n", metrics.latency_p50_ms));
        report.push_str(&format!("| P95 | {:.2} |\n", metrics.latency_p95_ms));
        report.push_str(&format!("| P99 | {:.2} |\n", metrics.latency_p99_ms));

        Ok(report)
    }

    fn single_json(metrics: &AggregateMetrics) -> Result<String> {
        Ok(serde_json::to_string_pretty(metrics)?)
    }

    fn single_html(metrics: &AggregateMetrics) -> Result<String> {
        let mut html = String::new();

        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("  <meta charset=\"utf-8\">\n");
        html.push_str(&format!("  <title>Benchmark Report: {}</title>\n", metrics.system_name));
        html.push_str("  <style>\n");
        html.push_str("    body { font-family: system-ui, sans-serif; max-width: 800px; margin: 0 auto; padding: 20px; }\n");
        html.push_str("    table { border-collapse: collapse; width: 100%; margin: 20px 0; }\n");
        html.push_str("    th, td { border: 1px solid #ddd; padding: 12px; text-align: left; }\n");
        html.push_str("    th { background-color: #f4f4f4; }\n");
        html.push_str("    .metric-good { color: #22c55e; }\n");
        html.push_str("    .metric-bad { color: #ef4444; }\n");
        html.push_str("  </style>\n");
        html.push_str("</head>\n<body>\n");

        html.push_str(&format!("<h1>Benchmark Report: {}</h1>\n", metrics.system_name));
        html.push_str(&format!("<p><strong>Queries Executed:</strong> {}</p>\n", metrics.query_count));

        html.push_str("<h2>Quality Metrics</h2>\n");
        html.push_str("<table>\n  <tr><th>Metric</th><th>Value</th></tr>\n");
        html.push_str(&format!("  <tr><td>Precision</td><td>{:.3}</td></tr>\n", metrics.avg_precision));
        html.push_str(&format!("  <tr><td>Recall</td><td>{:.3}</td></tr>\n", metrics.avg_recall));
        html.push_str(&format!("  <tr><td>F1 Score</td><td>{:.3}</td></tr>\n", metrics.avg_f1_score));
        html.push_str(&format!("  <tr><td>Hallucination Rate</td><td>{:.3}</td></tr>\n", metrics.avg_hallucination_rate));
        html.push_str("</table>\n");

        html.push_str("<h2>Latency Metrics</h2>\n");
        html.push_str("<table>\n  <tr><th>Percentile</th><th>Latency (ms)</th></tr>\n");
        html.push_str(&format!("  <tr><td>P50</td><td>{:.2}</td></tr>\n", metrics.latency_p50_ms));
        html.push_str(&format!("  <tr><td>P95</td><td>{:.2}</td></tr>\n", metrics.latency_p95_ms));
        html.push_str(&format!("  <tr><td>P99</td><td>{:.2}</td></tr>\n", metrics.latency_p99_ms));
        html.push_str("</table>\n");

        html.push_str("</body>\n</html>");

        Ok(html)
    }

    fn comparison_markdown(comparison: &BenchmarkComparison) -> Result<String> {
        let improvements = comparison.improvements();
        let mut report = String::new();

        report.push_str("# Benchmark Comparison Report\n\n");
        report.push_str(&format!(
            "Comparing **{}** (baseline) vs **{}** (hybrid)\n\n",
            comparison.baseline.system_name, comparison.hybrid.system_name
        ));

        report.push_str("## Quality Metrics Comparison\n\n");
        report.push_str("| Metric | Baseline | Hybrid | Improvement |\n");
        report.push_str("|--------|----------|--------|-------------|\n");
        report.push_str(&format!(
            "| Precision | {:.3} | {:.3} | {:+.1}% |\n",
            comparison.baseline.avg_precision,
            comparison.hybrid.avg_precision,
            improvements.precision_improvement
        ));
        report.push_str(&format!(
            "| Recall | {:.3} | {:.3} | {:+.1}% |\n",
            comparison.baseline.avg_recall,
            comparison.hybrid.avg_recall,
            improvements.recall_improvement
        ));
        report.push_str(&format!(
            "| F1 Score | {:.3} | {:.3} | {:+.1}% |\n",
            comparison.baseline.avg_f1_score,
            comparison.hybrid.avg_f1_score,
            improvements.f1_improvement
        ));
        report.push_str(&format!(
            "| Hallucination Rate | {:.3} | {:.3} | {:+.1}% reduction |\n",
            comparison.baseline.avg_hallucination_rate,
            comparison.hybrid.avg_hallucination_rate,
            improvements.hallucination_reduction
        ));

        report.push_str("\n## Latency Comparison\n\n");
        report.push_str("| Percentile | Baseline (ms) | Hybrid (ms) |\n");
        report.push_str("|------------|---------------|-------------|\n");
        report.push_str(&format!(
            "| P50 | {:.2} | {:.2} |\n",
            comparison.baseline.latency_p50_ms, comparison.hybrid.latency_p50_ms
        ));
        report.push_str(&format!(
            "| P95 | {:.2} | {:.2} |\n",
            comparison.baseline.latency_p95_ms, comparison.hybrid.latency_p95_ms
        ));
        report.push_str(&format!(
            "| P99 | {:.2} | {:.2} |\n",
            comparison.baseline.latency_p99_ms, comparison.hybrid.latency_p99_ms
        ));

        Ok(report)
    }

    fn comparison_json(comparison: &BenchmarkComparison) -> Result<String> {
        let improvements = comparison.improvements();
        let output = serde_json::json!({
            "baseline": comparison.baseline,
            "hybrid": comparison.hybrid,
            "improvements": improvements
        });
        Ok(serde_json::to_string_pretty(&output)?)
    }

    fn comparison_html(comparison: &BenchmarkComparison) -> Result<String> {
        let improvements = comparison.improvements();
        let mut html = String::new();

        html.push_str("<!DOCTYPE html>\n<html>\n<head>\n");
        html.push_str("  <meta charset=\"utf-8\">\n");
        html.push_str("  <title>Benchmark Comparison Report</title>\n");
        html.push_str("  <style>\n");
        html.push_str("    body { font-family: system-ui, sans-serif; max-width: 900px; margin: 0 auto; padding: 20px; }\n");
        html.push_str("    table { border-collapse: collapse; width: 100%; margin: 20px 0; }\n");
        html.push_str("    th, td { border: 1px solid #ddd; padding: 12px; text-align: left; }\n");
        html.push_str("    th { background-color: #f4f4f4; }\n");
        html.push_str("    .positive { color: #22c55e; font-weight: bold; }\n");
        html.push_str("    .negative { color: #ef4444; font-weight: bold; }\n");
        html.push_str("    .chart { margin: 20px 0; padding: 20px; background: #f9f9f9; border-radius: 8px; }\n");
        html.push_str("    .bar { height: 24px; background: #3b82f6; border-radius: 4px; margin: 4px 0; }\n");
        html.push_str("    .bar-label { display: flex; justify-content: space-between; }\n");
        html.push_str("  </style>\n");
        html.push_str("</head>\n<body>\n");

        html.push_str("<h1>Benchmark Comparison Report</h1>\n");
        html.push_str(&format!(
            "<p>Comparing <strong>{}</strong> (baseline) vs <strong>{}</strong> (hybrid)</p>\n",
            comparison.baseline.system_name, comparison.hybrid.system_name
        ));

        // Quality metrics table
        html.push_str("<h2>Quality Metrics</h2>\n");
        html.push_str("<table>\n");
        html.push_str("  <tr><th>Metric</th><th>Baseline</th><th>Hybrid</th><th>Improvement</th></tr>\n");

        let format_improvement = |val: f64| -> String {
            if val > 0.0 {
                format!("<span class=\"positive\">{:+.1}%</span>", val)
            } else if val < 0.0 {
                format!("<span class=\"negative\">{:+.1}%</span>", val)
            } else {
                format!("{:.1}%", val)
            }
        };

        html.push_str(&format!(
            "  <tr><td>Precision</td><td>{:.3}</td><td>{:.3}</td><td>{}</td></tr>\n",
            comparison.baseline.avg_precision,
            comparison.hybrid.avg_precision,
            format_improvement(improvements.precision_improvement)
        ));
        html.push_str(&format!(
            "  <tr><td>Recall</td><td>{:.3}</td><td>{:.3}</td><td>{}</td></tr>\n",
            comparison.baseline.avg_recall,
            comparison.hybrid.avg_recall,
            format_improvement(improvements.recall_improvement)
        ));
        html.push_str(&format!(
            "  <tr><td>F1 Score</td><td>{:.3}</td><td>{:.3}</td><td>{}</td></tr>\n",
            comparison.baseline.avg_f1_score,
            comparison.hybrid.avg_f1_score,
            format_improvement(improvements.f1_improvement)
        ));
        html.push_str(&format!(
            "  <tr><td>Hallucination Rate</td><td>{:.3}</td><td>{:.3}</td><td>{}</td></tr>\n",
            comparison.baseline.avg_hallucination_rate,
            comparison.hybrid.avg_hallucination_rate,
            format_improvement(improvements.hallucination_reduction)
        ));
        html.push_str("</table>\n");

        // Bar chart for quality metrics
        html.push_str("<div class=\"chart\">\n");
        html.push_str("  <h3>Quality Metrics Comparison</h3>\n");

        let bar_width = |val: f64| -> f64 { (val * 100.0).max(0.0).min(100.0) };

        html.push_str("  <div class=\"bar-label\"><span>Precision</span></div>\n");
        html.push_str(&format!(
            "  <div class=\"bar\" style=\"width: {}%; background: #94a3b8;\"></div>\n",
            bar_width(comparison.baseline.avg_precision)
        ));
        html.push_str(&format!(
            "  <div class=\"bar\" style=\"width: {}%;\"></div>\n",
            bar_width(comparison.hybrid.avg_precision)
        ));

        html.push_str("  <div class=\"bar-label\"><span>Recall</span></div>\n");
        html.push_str(&format!(
            "  <div class=\"bar\" style=\"width: {}%; background: #94a3b8;\"></div>\n",
            bar_width(comparison.baseline.avg_recall)
        ));
        html.push_str(&format!(
            "  <div class=\"bar\" style=\"width: {}%;\"></div>\n",
            bar_width(comparison.hybrid.avg_recall)
        ));

        html.push_str("  <div class=\"bar-label\"><span>F1 Score</span></div>\n");
        html.push_str(&format!(
            "  <div class=\"bar\" style=\"width: {}%; background: #94a3b8;\"></div>\n",
            bar_width(comparison.baseline.avg_f1_score)
        ));
        html.push_str(&format!(
            "  <div class=\"bar\" style=\"width: {}%;\"></div>\n",
            bar_width(comparison.hybrid.avg_f1_score)
        ));

        html.push_str("  <p><span style=\"color: #94a3b8;\">■</span> Baseline &nbsp; <span style=\"color: #3b82f6;\">■</span> Hybrid</p>\n");
        html.push_str("</div>\n");

        // Latency table
        html.push_str("<h2>Latency Metrics</h2>\n");
        html.push_str("<table>\n");
        html.push_str("  <tr><th>Percentile</th><th>Baseline (ms)</th><th>Hybrid (ms)</th></tr>\n");
        html.push_str(&format!(
            "  <tr><td>P50</td><td>{:.2}</td><td>{:.2}</td></tr>\n",
            comparison.baseline.latency_p50_ms, comparison.hybrid.latency_p50_ms
        ));
        html.push_str(&format!(
            "  <tr><td>P95</td><td>{:.2}</td><td>{:.2}</td></tr>\n",
            comparison.baseline.latency_p95_ms, comparison.hybrid.latency_p95_ms
        ));
        html.push_str(&format!(
            "  <tr><td>P99</td><td>{:.2}</td><td>{:.2}</td></tr>\n",
            comparison.baseline.latency_p99_ms, comparison.hybrid.latency_p99_ms
        ));
        html.push_str("</table>\n");

        html.push_str("</body>\n</html>");

        Ok(html)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_metrics() -> AggregateMetrics {
        AggregateMetrics {
            system_name: "TestSystem".to_string(),
            query_count: 100,
            avg_precision: 0.85,
            avg_recall: 0.75,
            avg_f1_score: 0.80,
            avg_hallucination_rate: 0.15,
            latency_p50_ms: 10.5,
            latency_p95_ms: 25.0,
            latency_p99_ms: 50.0,
            query_metrics: vec![],
        }
    }

    fn sample_comparison() -> BenchmarkComparison {
        BenchmarkComparison {
            baseline: AggregateMetrics {
                system_name: "SimpleVectorRAG".to_string(),
                query_count: 100,
                avg_precision: 0.70,
                avg_recall: 0.60,
                avg_f1_score: 0.65,
                avg_hallucination_rate: 0.30,
                latency_p50_ms: 8.0,
                latency_p95_ms: 20.0,
                latency_p99_ms: 40.0,
                query_metrics: vec![],
            },
            hybrid: AggregateMetrics {
                system_name: "GraphRAG+NARS".to_string(),
                query_count: 100,
                avg_precision: 0.85,
                avg_recall: 0.80,
                avg_f1_score: 0.82,
                avg_hallucination_rate: 0.15,
                latency_p50_ms: 15.0,
                latency_p95_ms: 35.0,
                latency_p99_ms: 60.0,
                query_metrics: vec![],
            },
        }
    }

    #[test]
    fn test_single_markdown_report() {
        let metrics = sample_metrics();
        let report = Reporter::single_report(&metrics, ReportFormat::Markdown).unwrap();

        assert!(report.contains("# Benchmark Report: TestSystem"));
        assert!(report.contains("| Precision | 0.850 |"));
        assert!(report.contains("| P50 | 10.50 |"));
    }

    #[test]
    fn test_single_json_report() {
        let metrics = sample_metrics();
        let report = Reporter::single_report(&metrics, ReportFormat::Json).unwrap();

        let parsed: serde_json::Value = serde_json::from_str(&report).unwrap();
        assert_eq!(parsed["system_name"], "TestSystem");
        assert_eq!(parsed["query_count"], 100);
    }

    #[test]
    fn test_single_html_report() {
        let metrics = sample_metrics();
        let report = Reporter::single_report(&metrics, ReportFormat::Html).unwrap();

        assert!(report.contains("<!DOCTYPE html>"));
        assert!(report.contains("<title>Benchmark Report: TestSystem</title>"));
        assert!(report.contains("0.850"));
    }

    #[test]
    fn test_comparison_markdown_report() {
        let comparison = sample_comparison();
        let report = Reporter::comparison_report(&comparison, ReportFormat::Markdown).unwrap();

        assert!(report.contains("# Benchmark Comparison Report"));
        assert!(report.contains("SimpleVectorRAG"));
        assert!(report.contains("GraphRAG+NARS"));
        assert!(report.contains("Improvement"));
    }

    #[test]
    fn test_comparison_html_report() {
        let comparison = sample_comparison();
        let report = Reporter::comparison_report(&comparison, ReportFormat::Html).unwrap();

        assert!(report.contains("<!DOCTYPE html>"));
        assert!(report.contains("class=\"positive\""));  // Should have positive improvements
    }
}
