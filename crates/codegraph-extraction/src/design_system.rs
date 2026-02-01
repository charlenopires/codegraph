//! Design System Detector - identifies UI frameworks from code patterns

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Supported design systems
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DesignSystemType {
    MaterialUI,
    Tailwind,
    Chakra,
    Bootstrap,
    AntDesign,
    Shadcn,
    Custom,
    Unknown,
}

impl DesignSystemType {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::MaterialUI => "material-ui",
            Self::Tailwind => "tailwind",
            Self::Chakra => "chakra",
            Self::Bootstrap => "bootstrap",
            Self::AntDesign => "ant-design",
            Self::Shadcn => "shadcn",
            Self::Custom => "custom",
            Self::Unknown => "unknown",
        }
    }
}

/// Detection result with confidence score
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectionResult {
    pub design_system: DesignSystemType,
    pub confidence: f32,
    pub evidence: Vec<String>,
}

/// Design system detection patterns
struct DetectionPattern {
    design_system: DesignSystemType,
    class_patterns: Vec<Regex>,
    import_patterns: Vec<Regex>,
    component_patterns: Vec<Regex>,
}

/// Design system detector
pub struct DesignSystemDetector {
    patterns: Vec<DetectionPattern>,
}

impl Default for DesignSystemDetector {
    fn default() -> Self {
        Self::new()
    }
}

// Pre-compiled regex patterns for common design systems
static TAILWIND_CLASSES: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(flex|grid|p-\d|m-\d|text-\w+|bg-\w+|rounded|shadow|hover:|focus:|sm:|md:|lg:|xl:)").unwrap()
});

static BOOTSTRAP_CLASSES: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?i)\b(btn-|col-|row|container|navbar|card|modal|form-control|d-flex|justify-content|align-items)").unwrap()
});

static MATERIAL_IMPORTS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"@mui/|@material-ui/|material-ui").unwrap()
});

static CHAKRA_IMPORTS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"@chakra-ui/").unwrap()
});

static ANT_IMPORTS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"antd|@ant-design/").unwrap()
});

static SHADCN_PATTERNS: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"@/components/ui/|shadcn").unwrap()
});

impl DesignSystemDetector {
    pub fn new() -> Self {
        let patterns = vec![
            // Tailwind CSS
            DetectionPattern {
                design_system: DesignSystemType::Tailwind,
                class_patterns: vec![
                    Regex::new(r"(?i)\bflex\b").unwrap(),
                    Regex::new(r"(?i)\bgrid\b").unwrap(),
                    Regex::new(r"(?i)\bp-\d").unwrap(),
                    Regex::new(r"(?i)\bm-\d").unwrap(),
                    Regex::new(r"(?i)\btext-(xs|sm|base|lg|xl)").unwrap(),
                    Regex::new(r"(?i)\bbg-(white|black|gray|red|blue|green)").unwrap(),
                    Regex::new(r"(?i)\brounded(-\w+)?").unwrap(),
                    Regex::new(r"(?i)\bshadow(-\w+)?").unwrap(),
                    Regex::new(r"(?i)\bhover:").unwrap(),
                    Regex::new(r"(?i)\bfocus:").unwrap(),
                    Regex::new(r"(?i)\b(sm|md|lg|xl):").unwrap(),
                    Regex::new(r"(?i)\bw-\d").unwrap(),
                    Regex::new(r"(?i)\bh-\d").unwrap(),
                ],
                import_patterns: vec![
                    Regex::new(r"tailwindcss").unwrap(),
                    Regex::new(r"tailwind\.config").unwrap(),
                ],
                component_patterns: vec![],
            },
            // Bootstrap
            DetectionPattern {
                design_system: DesignSystemType::Bootstrap,
                class_patterns: vec![
                    Regex::new(r"(?i)\bbtn-").unwrap(),
                    Regex::new(r"(?i)\bcol-").unwrap(),
                    Regex::new(r"(?i)\brow\b").unwrap(),
                    Regex::new(r"(?i)\bcontainer(-fluid)?\b").unwrap(),
                    Regex::new(r"(?i)\bnavbar").unwrap(),
                    Regex::new(r"(?i)\bcard(-\w+)?").unwrap(),
                    Regex::new(r"(?i)\bmodal").unwrap(),
                    Regex::new(r"(?i)\bform-control").unwrap(),
                    Regex::new(r"(?i)\bd-flex").unwrap(),
                    Regex::new(r"(?i)\bjustify-content-").unwrap(),
                    Regex::new(r"(?i)\balign-items-").unwrap(),
                    Regex::new(r"(?i)\btext-center").unwrap(),
                ],
                import_patterns: vec![
                    Regex::new(r"bootstrap").unwrap(),
                    Regex::new(r"react-bootstrap").unwrap(),
                ],
                component_patterns: vec![],
            },
            // Material UI
            DetectionPattern {
                design_system: DesignSystemType::MaterialUI,
                class_patterns: vec![
                    Regex::new(r"(?i)\bMui").unwrap(),
                    Regex::new(r"(?i)\bmakeStyles").unwrap(),
                ],
                import_patterns: vec![
                    Regex::new(r"@mui/").unwrap(),
                    Regex::new(r"@material-ui/").unwrap(),
                    Regex::new(r"material-ui").unwrap(),
                ],
                component_patterns: vec![
                    Regex::new(r"<(Button|TextField|Paper|Grid|Box|Typography|AppBar|Toolbar|IconButton|Menu|MenuItem|Dialog|Snackbar|Drawer|List|ListItem|Card|CardContent|CardActions|Avatar|Chip|Badge|Tooltip|Tabs|Tab)").unwrap(),
                ],
            },
            // Chakra UI
            DetectionPattern {
                design_system: DesignSystemType::Chakra,
                class_patterns: vec![],
                import_patterns: vec![
                    Regex::new(r"@chakra-ui/").unwrap(),
                ],
                component_patterns: vec![
                    Regex::new(r"<(ChakraProvider|Box|Flex|Stack|HStack|VStack|Grid|Button|Input|Heading|Text|Image|Avatar|Badge|Alert|Toast|Modal|Drawer|Menu|Tabs|Accordion|Card)").unwrap(),
                ],
            },
            // Ant Design
            DetectionPattern {
                design_system: DesignSystemType::AntDesign,
                class_patterns: vec![
                    Regex::new(r"(?i)\bant-").unwrap(),
                ],
                import_patterns: vec![
                    Regex::new(r"antd").unwrap(),
                    Regex::new(r"@ant-design/").unwrap(),
                ],
                component_patterns: vec![
                    Regex::new(r"<(ConfigProvider|Layout|Menu|Breadcrumb|Button|Input|Form|Table|Modal|Drawer|Notification|Message|Card|Tabs|Collapse|Avatar|Badge|Tag|Progress|Spin)").unwrap(),
                ],
            },
            // shadcn/ui
            DetectionPattern {
                design_system: DesignSystemType::Shadcn,
                class_patterns: vec![],
                import_patterns: vec![
                    Regex::new(r"@/components/ui/").unwrap(),
                    Regex::new(r"shadcn").unwrap(),
                ],
                component_patterns: vec![
                    Regex::new(r"<(Accordion|Alert|AlertDialog|AspectRatio|Avatar|Badge|Button|Calendar|Card|Carousel|Checkbox|Collapsible|Command|ContextMenu|Dialog|Drawer|DropdownMenu|Form|HoverCard|Input|Label|Menubar|NavigationMenu|Popover|Progress|RadioGroup|ScrollArea|Select|Separator|Sheet|Skeleton|Slider|Switch|Table|Tabs|Textarea|Toast|Toggle|Tooltip)").unwrap(),
                ],
            },
        ];

        Self { patterns }
    }

    /// Detect design system from HTML classes
    pub fn detect_from_classes(&self, classes: &[String]) -> DetectionResult {
        let class_text = classes.join(" ");
        self.detect_from_content(&class_text, "", "")
    }

    /// Detect design system from full content (HTML + CSS + JS)
    pub fn detect_from_content(&self, html: &str, css: &str, js: &str) -> DetectionResult {
        let mut scores: HashMap<DesignSystemType, (f32, Vec<String>)> = HashMap::new();

        let combined = format!("{} {} {}", html, css, js);

        for pattern in &self.patterns {
            let mut score = 0.0f32;
            let mut evidence = Vec::new();

            // Check class patterns
            for class_re in &pattern.class_patterns {
                let matches: Vec<_> = class_re.find_iter(&combined).collect();
                if !matches.is_empty() {
                    score += matches.len() as f32 * 0.1;
                    evidence.push(format!("class pattern: {} ({} matches)", class_re.as_str(), matches.len()));
                }
            }

            // Check import patterns (higher weight)
            for import_re in &pattern.import_patterns {
                if import_re.is_match(&combined) {
                    score += 2.0;
                    evidence.push(format!("import: {}", import_re.as_str()));
                }
            }

            // Check component patterns (medium weight)
            for comp_re in &pattern.component_patterns {
                let matches: Vec<_> = comp_re.find_iter(&combined).collect();
                if !matches.is_empty() {
                    score += matches.len() as f32 * 0.5;
                    evidence.push(format!("component: {} ({} matches)", comp_re.as_str(), matches.len()));
                }
            }

            if score > 0.0 {
                scores.insert(pattern.design_system, (score, evidence));
            }
        }

        // Find the highest scoring design system
        let mut best = (DesignSystemType::Unknown, 0.0f32, Vec::new());
        for (ds, (score, evidence)) in scores {
            if score > best.1 {
                best = (ds, score, evidence);
            }
        }

        // Calculate confidence (0.0 - 1.0)
        let confidence = (best.1 / 10.0).min(1.0);

        DetectionResult {
            design_system: best.0,
            confidence,
            evidence: best.2,
        }
    }

    /// Quick check for Tailwind classes
    pub fn is_tailwind(&self, classes: &str) -> bool {
        TAILWIND_CLASSES.is_match(classes)
    }

    /// Quick check for Bootstrap classes
    pub fn is_bootstrap(&self, classes: &str) -> bool {
        BOOTSTRAP_CLASSES.is_match(classes)
    }

    /// Quick check for Material UI imports
    pub fn is_material_ui(&self, imports: &str) -> bool {
        MATERIAL_IMPORTS.is_match(imports)
    }

    /// Quick check for Chakra UI imports
    pub fn is_chakra(&self, imports: &str) -> bool {
        CHAKRA_IMPORTS.is_match(imports)
    }

    /// Quick check for Ant Design imports
    pub fn is_ant_design(&self, imports: &str) -> bool {
        ANT_IMPORTS.is_match(imports)
    }

    /// Quick check for shadcn/ui patterns
    pub fn is_shadcn(&self, content: &str) -> bool {
        SHADCN_PATTERNS.is_match(content)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_detect_tailwind() {
        let detector = DesignSystemDetector::new();
        let classes = vec![
            "flex".to_string(),
            "p-4".to_string(),
            "bg-blue-500".to_string(),
            "hover:bg-blue-600".to_string(),
            "rounded-lg".to_string(),
        ];

        let result = detector.detect_from_classes(&classes);
        assert_eq!(result.design_system, DesignSystemType::Tailwind);
        assert!(result.confidence > 0.0);
    }

    #[test]
    fn test_detect_bootstrap() {
        let detector = DesignSystemDetector::new();
        let classes = vec![
            "btn".to_string(),
            "btn-primary".to_string(),
            "col-md-6".to_string(),
            "d-flex".to_string(),
        ];

        let result = detector.detect_from_classes(&classes);
        assert_eq!(result.design_system, DesignSystemType::Bootstrap);
    }

    #[test]
    fn test_detect_material_ui() {
        let detector = DesignSystemDetector::new();
        let js = r#"import { Button, TextField } from '@mui/material';"#;

        let result = detector.detect_from_content("", "", js);
        assert_eq!(result.design_system, DesignSystemType::MaterialUI);
    }
}
