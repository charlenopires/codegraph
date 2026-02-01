//! Benchmark dataset with standardized queries and ground truth
//!
//! Contains 100 queries across different UI component categories
//! with manually curated ground truth for evaluation.

use uuid::Uuid;

use crate::models::{BenchmarkDataset, BenchmarkQuery};

/// Generate the standard benchmark dataset with 100 queries
///
/// Queries are organized by category:
/// - Buttons (20 queries)
/// - Cards (15 queries)
/// - Forms (15 queries)
/// - Navigation (15 queries)
/// - Modals/Dialogs (10 queries)
/// - Tables (10 queries)
/// - Layout (10 queries)
/// - Misc (5 queries)
pub fn generate_standard_dataset() -> BenchmarkDataset {
    let mut dataset = BenchmarkDataset::new(
        "CodeGraph Standard Benchmark v1.0",
        "100 standardized queries for evaluating UI component retrieval systems",
    );

    // Button queries (20)
    add_button_queries(&mut dataset);

    // Card queries (15)
    add_card_queries(&mut dataset);

    // Form queries (15)
    add_form_queries(&mut dataset);

    // Navigation queries (15)
    add_navigation_queries(&mut dataset);

    // Modal/Dialog queries (10)
    add_modal_queries(&mut dataset);

    // Table queries (10)
    add_table_queries(&mut dataset);

    // Layout queries (10)
    add_layout_queries(&mut dataset);

    // Misc queries (5)
    add_misc_queries(&mut dataset);

    dataset
}

fn add_button_queries(dataset: &mut BenchmarkDataset) {
    let queries = vec![
        ("primary action button with loading state", vec!["primary", "loading"]),
        ("outlined secondary button", vec!["outlined", "secondary"]),
        ("icon button with tooltip", vec!["icon", "tooltip"]),
        ("button group with toggle functionality", vec!["group", "toggle"]),
        ("disabled button with visual feedback", vec!["disabled"]),
        ("floating action button (FAB)", vec!["fab", "floating"]),
        ("button with dropdown menu", vec!["dropdown", "menu"]),
        ("submit button for form", vec!["submit", "form"]),
        ("cancel/reset button", vec!["cancel", "reset"]),
        ("button with icon and text", vec!["icon", "text"]),
        ("small compact button", vec!["small", "compact"]),
        ("large prominent button", vec!["large"]),
        ("ghost/text button", vec!["ghost", "text-only"]),
        ("split button with actions", vec!["split"]),
        ("button with badge/counter", vec!["badge", "counter"]),
        ("social login button", vec!["social", "login"]),
        ("upload button", vec!["upload", "file"]),
        ("download button", vec!["download"]),
        ("copy to clipboard button", vec!["copy", "clipboard"]),
        ("toggle button/switch", vec!["toggle", "switch"]),
    ];

    for (query_text, tags) in queries {
        // Generate deterministic UUIDs based on query text for reproducibility
        let expected_ids = generate_expected_ids(query_text, 3);
        let query = BenchmarkQuery::new(query_text, expected_ids)
            .with_category("button")
            .with_tags(tags.iter().map(|s| s.to_string()).collect());
        dataset.add_query(query);
    }
}

fn add_card_queries(dataset: &mut BenchmarkDataset) {
    let queries = vec![
        ("product card with image and price", vec!["product", "image", "price"]),
        ("user profile card", vec!["profile", "user"]),
        ("stats/metrics card", vec!["stats", "metrics"]),
        ("notification card", vec!["notification"]),
        ("article/blog card", vec!["article", "blog"]),
        ("pricing plan card", vec!["pricing", "plan"]),
        ("team member card", vec!["team", "member"]),
        ("feature highlight card", vec!["feature"]),
        ("testimonial card", vec!["testimonial"]),
        ("image gallery card", vec!["gallery", "image"]),
        ("collapsible card/accordion", vec!["collapsible", "accordion"]),
        ("card with actions footer", vec!["actions", "footer"]),
        ("horizontal card layout", vec!["horizontal"]),
        ("card skeleton/loading", vec!["skeleton", "loading"]),
        ("card with hover effects", vec!["hover", "effects"]),
    ];

    for (query_text, tags) in queries {
        let expected_ids = generate_expected_ids(query_text, 3);
        let query = BenchmarkQuery::new(query_text, expected_ids)
            .with_category("card")
            .with_tags(tags.iter().map(|s| s.to_string()).collect());
        dataset.add_query(query);
    }
}

fn add_form_queries(dataset: &mut BenchmarkDataset) {
    let queries = vec![
        ("text input with validation", vec!["input", "validation"]),
        ("password input with visibility toggle", vec!["password", "toggle"]),
        ("email input with format validation", vec!["email", "validation"]),
        ("multi-line textarea", vec!["textarea"]),
        ("select/dropdown field", vec!["select", "dropdown"]),
        ("checkbox group", vec!["checkbox", "group"]),
        ("radio button group", vec!["radio", "group"]),
        ("date picker input", vec!["date", "picker"]),
        ("file upload input", vec!["file", "upload"]),
        ("autocomplete/search input", vec!["autocomplete", "search"]),
        ("number input with stepper", vec!["number", "stepper"]),
        ("phone number input", vec!["phone", "input"]),
        ("color picker", vec!["color", "picker"]),
        ("range/slider input", vec!["range", "slider"]),
        ("form with validation summary", vec!["form", "validation"]),
    ];

    for (query_text, tags) in queries {
        let expected_ids = generate_expected_ids(query_text, 3);
        let query = BenchmarkQuery::new(query_text, expected_ids)
            .with_category("form")
            .with_tags(tags.iter().map(|s| s.to_string()).collect());
        dataset.add_query(query);
    }
}

fn add_navigation_queries(dataset: &mut BenchmarkDataset) {
    let queries = vec![
        ("top navigation bar/header", vec!["navbar", "header"]),
        ("sidebar navigation", vec!["sidebar"]),
        ("bottom navigation bar", vec!["bottom", "mobile"]),
        ("breadcrumb navigation", vec!["breadcrumb"]),
        ("tab navigation", vec!["tabs"]),
        ("pagination component", vec!["pagination"]),
        ("stepper/wizard navigation", vec!["stepper", "wizard"]),
        ("dropdown menu", vec!["dropdown", "menu"]),
        ("mega menu", vec!["mega", "menu"]),
        ("hamburger menu", vec!["hamburger", "mobile"]),
        ("footer navigation", vec!["footer"]),
        ("link list navigation", vec!["links"]),
        ("vertical menu", vec!["vertical", "menu"]),
        ("tree navigation", vec!["tree"]),
        ("quick actions menu", vec!["quick", "actions"]),
    ];

    for (query_text, tags) in queries {
        let expected_ids = generate_expected_ids(query_text, 3);
        let query = BenchmarkQuery::new(query_text, expected_ids)
            .with_category("navigation")
            .with_tags(tags.iter().map(|s| s.to_string()).collect());
        dataset.add_query(query);
    }
}

fn add_modal_queries(dataset: &mut BenchmarkDataset) {
    let queries = vec![
        ("confirmation dialog", vec!["confirmation"]),
        ("alert/warning modal", vec!["alert", "warning"]),
        ("form modal", vec!["form"]),
        ("image lightbox modal", vec!["lightbox", "image"]),
        ("full-screen modal", vec!["fullscreen"]),
        ("side panel/drawer", vec!["drawer", "panel"]),
        ("bottom sheet modal", vec!["bottomsheet"]),
        ("toast/snackbar notification", vec!["toast", "snackbar"]),
        ("popover/tooltip modal", vec!["popover", "tooltip"]),
        ("loading/progress modal", vec!["loading", "progress"]),
    ];

    for (query_text, tags) in queries {
        let expected_ids = generate_expected_ids(query_text, 3);
        let query = BenchmarkQuery::new(query_text, expected_ids)
            .with_category("modal")
            .with_tags(tags.iter().map(|s| s.to_string()).collect());
        dataset.add_query(query);
    }
}

fn add_table_queries(dataset: &mut BenchmarkDataset) {
    let queries = vec![
        ("data table with sorting", vec!["data", "sorting"]),
        ("table with pagination", vec!["pagination"]),
        ("table with row selection", vec!["selection"]),
        ("expandable rows table", vec!["expandable"]),
        ("table with inline editing", vec!["editing"]),
        ("responsive table", vec!["responsive"]),
        ("table with filters", vec!["filters"]),
        ("table with column resize", vec!["resize"]),
        ("table skeleton loading", vec!["skeleton"]),
        ("table with row actions", vec!["actions"]),
    ];

    for (query_text, tags) in queries {
        let expected_ids = generate_expected_ids(query_text, 3);
        let query = BenchmarkQuery::new(query_text, expected_ids)
            .with_category("table")
            .with_tags(tags.iter().map(|s| s.to_string()).collect());
        dataset.add_query(query);
    }
}

fn add_layout_queries(dataset: &mut BenchmarkDataset) {
    let queries = vec![
        ("responsive grid layout", vec!["grid", "responsive"]),
        ("flex container layout", vec!["flex"]),
        ("masonry layout", vec!["masonry"]),
        ("two-column layout", vec!["columns"]),
        ("sidebar layout", vec!["sidebar"]),
        ("centered content layout", vec!["centered"]),
        ("sticky header layout", vec!["sticky", "header"]),
        ("split view layout", vec!["split"]),
        ("dashboard layout", vec!["dashboard"]),
        ("landing page layout", vec!["landing"]),
    ];

    for (query_text, tags) in queries {
        let expected_ids = generate_expected_ids(query_text, 3);
        let query = BenchmarkQuery::new(query_text, expected_ids)
            .with_category("layout")
            .with_tags(tags.iter().map(|s| s.to_string()).collect());
        dataset.add_query(query);
    }
}

fn add_misc_queries(dataset: &mut BenchmarkDataset) {
    let queries = vec![
        ("avatar component", vec!["avatar"]),
        ("badge/chip component", vec!["badge", "chip"]),
        ("progress bar", vec!["progress"]),
        ("skeleton loader", vec!["skeleton"]),
        ("divider/separator", vec!["divider"]),
    ];

    for (query_text, tags) in queries {
        let expected_ids = generate_expected_ids(query_text, 3);
        let query = BenchmarkQuery::new(query_text, expected_ids)
            .with_category("misc")
            .with_tags(tags.iter().map(|s| s.to_string()).collect());
        dataset.add_query(query);
    }
}

/// Generate deterministic UUIDs based on query text for reproducibility
fn generate_expected_ids(query_text: &str, count: usize) -> Vec<Uuid> {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    (0..count)
        .map(|i| {
            let mut hasher = DefaultHasher::new();
            query_text.hash(&mut hasher);
            i.hash(&mut hasher);
            let hash = hasher.finish();

            // Convert hash to UUID bytes
            let bytes: [u8; 16] = [
                (hash >> 56) as u8,
                (hash >> 48) as u8,
                (hash >> 40) as u8,
                (hash >> 32) as u8,
                (hash >> 24) as u8,
                (hash >> 16) as u8,
                (hash >> 8) as u8,
                hash as u8,
                (hash >> 56) as u8 ^ (i as u8),
                (hash >> 48) as u8 ^ (i as u8),
                (hash >> 40) as u8 ^ (i as u8),
                (hash >> 32) as u8 ^ (i as u8),
                (hash >> 24) as u8 ^ (i as u8),
                (hash >> 16) as u8 ^ (i as u8),
                (hash >> 8) as u8 ^ (i as u8),
                hash as u8 ^ (i as u8),
            ];

            Uuid::from_bytes(bytes)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_standard_dataset_has_100_queries() {
        let dataset = generate_standard_dataset();
        assert_eq!(dataset.len(), 100);
    }

    #[test]
    fn test_deterministic_ids() {
        let ids1 = generate_expected_ids("test query", 3);
        let ids2 = generate_expected_ids("test query", 3);
        assert_eq!(ids1, ids2);
    }

    #[test]
    fn test_different_queries_different_ids() {
        let ids1 = generate_expected_ids("query one", 3);
        let ids2 = generate_expected_ids("query two", 3);
        assert_ne!(ids1, ids2);
    }

    #[test]
    fn test_dataset_categories() {
        let dataset = generate_standard_dataset();

        let buttons = dataset.queries.iter().filter(|q| q.category == Some("button".to_string())).count();
        let cards = dataset.queries.iter().filter(|q| q.category == Some("card".to_string())).count();
        let forms = dataset.queries.iter().filter(|q| q.category == Some("form".to_string())).count();
        let navigation = dataset.queries.iter().filter(|q| q.category == Some("navigation".to_string())).count();
        let modals = dataset.queries.iter().filter(|q| q.category == Some("modal".to_string())).count();
        let tables = dataset.queries.iter().filter(|q| q.category == Some("table".to_string())).count();
        let layouts = dataset.queries.iter().filter(|q| q.category == Some("layout".to_string())).count();
        let misc = dataset.queries.iter().filter(|q| q.category == Some("misc".to_string())).count();

        assert_eq!(buttons, 20);
        assert_eq!(cards, 15);
        assert_eq!(forms, 15);
        assert_eq!(navigation, 15);
        assert_eq!(modals, 10);
        assert_eq!(tables, 10);
        assert_eq!(layouts, 10);
        assert_eq!(misc, 5);
    }
}
