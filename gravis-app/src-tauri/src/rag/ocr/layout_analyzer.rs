// Layout Analyzer - Détecte la structure sémantique des documents OCR
// Analyse spatiale des BoundingBox pour identifier figures, tables, headers, etc.

use crate::rag::ocr::types::{BoundingBox, OCRBlock, BlockType, BoundingBoxExt};
use regex::Regex;
use std::collections::HashMap;

/// Configuration pour l'analyseur de layout
#[derive(Debug, Clone)]
pub struct LayoutAnalyzerConfig {
    pub min_figure_area: f64,
    pub max_text_density_for_figure: f64,
    pub caption_patterns: Vec<Regex>,
    pub min_table_width: f64,
    pub min_table_columns: usize,
    pub grid_cell_size: f64,
    pub header_aspect_ratio_max: f64,
    pub header_max_y_percent: f64,
}

impl Default for LayoutAnalyzerConfig {
    fn default() -> Self {
        Self {
            min_figure_area: 50000.0,
            max_text_density_for_figure: 0.003,
            caption_patterns: vec![
                Regex::new(r"(?i)^(figure|fig\.?|chart|graph|diagram)\s*\d+").unwrap(),
                Regex::new(r"(?i)^(tableau|table)\s*\d+").unwrap(),
            ],
            min_table_width: 200.0,
            min_table_columns: 2,
            grid_cell_size: 50.0,
            header_aspect_ratio_max: 0.3,
            header_max_y_percent: 0.15,
        }
    }
}

/// Analyseur de layout pour documents OCR
pub struct LayoutAnalyzer {
    config: LayoutAnalyzerConfig,
}

impl LayoutAnalyzer {
    pub fn new(config: LayoutAnalyzerConfig) -> Self {
        Self { config }
    }

    pub fn with_default_config() -> Self {
        Self::new(LayoutAnalyzerConfig::default())
    }

    /// Analyse le layout et retourne les blocs sémantiques détectés
    /// BoundingBox doit avoir un champ text pour le contenu
    pub fn analyze_layout_with_text(
        &self,
        boxes_with_text: &[(BoundingBox, String)],
        image_dimensions: (f64, f64),
        page_number: u32, // 🆕 Numéro de page pour les blocs
    ) -> Vec<OCRBlock> {
        if boxes_with_text.is_empty() {
            return vec![];
        }

        let mut blocks = Vec::new();
        let (page_width, page_height) = image_dimensions;

        // 1. Identifier les régions candidates
        let regions = self.identify_regions(boxes_with_text);

        // 2. Classifier chaque région
        for region in regions {
            if let Some(block) = self.classify_region(&region, page_height, page_number) {
                blocks.push(block);
            }
        }

        // 3. Trier les blocs par position (top-to-bottom, left-to-right)
        blocks.sort_by(|a, b| {
            let y_diff = (a.bounding_box.y - b.bounding_box.y).abs();
            if y_diff < 10.0 {
                a.bounding_box.x.partial_cmp(&b.bounding_box.x).unwrap()
            } else {
                a.bounding_box.y.partial_cmp(&b.bounding_box.y).unwrap()
            }
        });

        blocks
    }

    /// Identifie les régions cohérentes
    fn identify_regions(&self, boxes_with_text: &[(BoundingBox, String)]) -> Vec<Region> {
        let mut regions = Vec::new();
        let mut visited = vec![false; boxes_with_text.len()];

        for (idx, _) in boxes_with_text.iter().enumerate() {
            if visited[idx] {
                continue;
            }

            let cluster = self.grow_region(idx, boxes_with_text, &mut visited);

            if !cluster.is_empty() {
                regions.push(Region::from_boxes_with_text(cluster));
            }
        }

        regions
    }

    /// Croissance de région par connectivité spatiale
    fn grow_region(
        &self,
        start_idx: usize,
        boxes_with_text: &[(BoundingBox, String)],
        visited: &mut [bool],
    ) -> Vec<(BoundingBox, String)> {
        let mut cluster = Vec::new();
        let mut stack = vec![start_idx];

        while let Some(idx) = stack.pop() {
            if visited[idx] {
                continue;
            }

            visited[idx] = true;
            let current = &boxes_with_text[idx];
            cluster.push(current.clone());

            // Trouver les voisins
            for (other_idx, other) in boxes_with_text.iter().enumerate() {
                if visited[other_idx] {
                    continue;
                }

                if self.are_neighbors(&current.0, &other.0) {
                    stack.push(other_idx);
                }
            }
        }

        cluster
    }

    /// Détermine si deux boxes sont voisines
    fn are_neighbors(&self, a: &BoundingBox, b: &BoundingBox) -> bool {
        let horizontal_gap = if a.x > b.x + b.width {
            a.x - (b.x + b.width)
        } else if b.x > a.x + a.width {
            b.x - (a.x + a.width)
        } else {
            0.0
        };

        let vertical_gap = if a.y > b.y + b.height {
            a.y - (b.y + b.height)
        } else if b.y > a.y + a.height {
            b.y - (a.y + a.height)
        } else {
            0.0
        };

        horizontal_gap < 30.0 && vertical_gap < 30.0
    }

    /// Classifie une région en type de bloc
    fn classify_region(&self, region: &Region, page_height: f64, page_number: u32) -> Option<OCRBlock> {
        let bbox = &region.bounding_box;
        let content = region.get_text_content();

        // 1. Détecter les figures
        if self.is_figure_region(region) {
            return Some(OCRBlock {
                page_number,
                block_type: BlockType::Figure,
                content,
                bounding_box: bbox.clone(),
                confidence: 0.8,
                spans: vec![],
            });
        }

        // 2. Détecter les tables
        if self.is_table_region(region) {
            return Some(OCRBlock {
                page_number,
                block_type: BlockType::Table,
                content,
                bounding_box: bbox.clone(),
                confidence: 0.85,
                spans: vec![],
            });
        }

        // 3. Détecter les headers
        if self.is_header_region(region, page_height) {
            return Some(OCRBlock {
                page_number,
                block_type: BlockType::Header,
                content,
                bounding_box: bbox.clone(),
                confidence: 0.9,
                spans: vec![],
            });
        }

        // 4. Détecter les listes
        if self.is_list_region(region) {
            return Some(OCRBlock {
                page_number,
                block_type: BlockType::List,
                content,
                bounding_box: bbox.clone(),
                confidence: 0.75,
                spans: vec![],
            });
        }

        // 5. Détecter key-value pairs
        if self.is_keyvalue_region(region) {
            return Some(OCRBlock {
                page_number,
                block_type: BlockType::KeyValue,
                content,
                bounding_box: bbox.clone(),
                confidence: 0.7,
                spans: vec![],
            });
        }

        // 6. Par défaut: bloc de texte
        Some(OCRBlock {
            page_number,
            block_type: BlockType::Text,
            content,
            bounding_box: bbox.clone(),
            confidence: 0.9,
            spans: vec![],
        })
    }

    fn is_figure_region(&self, region: &Region) -> bool {
        let area = region.bounding_box.area();
        if area < self.config.min_figure_area {
            return false;
        }

        let text_density = region.text_density();
        if text_density > self.config.max_text_density_for_figure {
            return false;
        }

        let content = region.get_text_content();
        for pattern in &self.config.caption_patterns {
            if pattern.is_match(&content) {
                return true;
            }
        }

        text_density < 0.001 && area > 100000.0
    }

    fn is_table_region(&self, region: &Region) -> bool {
        if region.bounding_box.width < self.config.min_table_width {
            return false;
        }

        let columns = self.detect_columns(&region.boxes);
        columns.len() >= self.config.min_table_columns
    }

    fn is_header_region(&self, region: &Region, page_height: f64) -> bool {
        let bbox = &region.bounding_box;

        let y_percent = bbox.y / page_height;
        if y_percent > self.config.header_max_y_percent {
            return false;
        }

        let aspect_ratio = bbox.height / bbox.width;
        if aspect_ratio > self.config.header_aspect_ratio_max {
            return false;
        }

        let content = region.get_text_content();
        content.len() < 100 && content.lines().count() <= 2
    }

    fn is_list_region(&self, region: &Region) -> bool {
        let content = region.get_text_content();
        let lines: Vec<&str> = content.lines().collect();

        if lines.len() < 2 {
            return false;
        }

        let list_patterns = [
            Regex::new(r"^[\s]*[•\-\*]\s+").unwrap(),
            Regex::new(r"^[\s]*\d+[\.\)]\s+").unwrap(),
            Regex::new(r"^[\s]*[a-z][\.\)]\s+").unwrap(),
        ];

        let mut list_items = 0;
        for line in &lines {
            for pattern in &list_patterns {
                if pattern.is_match(line) {
                    list_items += 1;
                    break;
                }
            }
        }

        (list_items as f64 / lines.len() as f64) > 0.5
    }

    fn is_keyvalue_region(&self, region: &Region) -> bool {
        let content = region.get_text_content();
        let lines: Vec<&str> = content.lines().collect();

        if lines.len() < 2 {
            return false;
        }

        let kv_pattern = Regex::new(r"^[^:=]+[:=]\s*.+$").unwrap();

        let mut kv_count = 0;
        for line in &lines {
            if kv_pattern.is_match(line) {
                kv_count += 1;
            }
        }

        (kv_count as f64 / lines.len() as f64) > 0.6
    }

    fn detect_columns(&self, boxes: &[(BoundingBox, String)]) -> Vec<Column> {
        if boxes.is_empty() {
            return vec![];
        }

        let mut x_clusters: HashMap<u32, Vec<&BoundingBox>> = HashMap::new();

        for (bbox, _text) in boxes {
            let cluster_key = (bbox.x / 20.0) as u32 * 20;
            x_clusters.entry(cluster_key).or_insert_with(Vec::new).push(bbox);
        }

        x_clusters
            .into_iter()
            .filter(|(_, boxes)| boxes.len() >= 2)
            .map(|(x, boxes)| Column {
                x: x as f64,
                boxes: boxes.into_iter().cloned().collect()
            })
            .collect()
    }
}

/// Région détectée dans le document
#[derive(Debug, Clone)]
struct Region {
    boxes: Vec<(BoundingBox, String)>,
    bounding_box: BoundingBox,
}

impl Region {
    fn from_boxes_with_text(boxes: Vec<(BoundingBox, String)>) -> Self {
        if boxes.is_empty() {
            return Self {
                boxes: vec![],
                bounding_box: BoundingBox {
                    x: 0.0,
                    y: 0.0,
                    width: 0.0,
                    height: 0.0,
                },
            };
        }

        // Calculer le bounding box englobant
        let min_x = boxes.iter().map(|(b, _)| b.x).fold(f64::INFINITY, f64::min);
        let min_y = boxes.iter().map(|(b, _)| b.y).fold(f64::INFINITY, f64::min);
        let max_x = boxes.iter().map(|(b, _)| b.x + b.width).fold(f64::NEG_INFINITY, f64::max);
        let max_y = boxes.iter().map(|(b, _)| b.y + b.height).fold(f64::NEG_INFINITY, f64::max);

        let bounding_box = BoundingBox {
            x: min_x,
            y: min_y,
            width: max_x - min_x,
            height: max_y - min_y,
        };

        Self {
            boxes,
            bounding_box,
        }
    }

    fn get_text_content(&self) -> String {
        self.boxes
            .iter()
            .map(|(_, text)| text.as_str())
            .collect::<Vec<_>>()
            .join(" ")
    }

    fn text_density(&self) -> f64 {
        let area = self.bounding_box.area();
        if area == 0.0 {
            return 0.0;
        }

        let text_len = self.get_text_content().len() as f64;
        text_len / area
    }
}

/// Colonne détectée dans une table
#[derive(Debug)]
struct Column {
    x: f64,
    boxes: Vec<BoundingBox>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_figure_detection() {
        let analyzer = LayoutAnalyzer::with_default_config();

        let boxes = vec![(
            BoundingBox {
                x: 100.0,
                y: 100.0,
                width: 400.0,
                height: 300.0,
            },
            "Figure 1: Revenue".to_string(),
        )];

        let region = Region::from_boxes_with_text(boxes);
        assert!(analyzer.is_figure_region(&region));
    }

    #[test]
    fn test_header_detection() {
        let analyzer = LayoutAnalyzer::with_default_config();

        let boxes = vec![(
            BoundingBox {
                x: 50.0,
                y: 20.0,
                width: 500.0,
                height: 30.0,
            },
            "Chapter 1: Introduction".to_string(),
        )];

        let region = Region::from_boxes_with_text(boxes);
        assert!(analyzer.is_header_region(&region, 800.0));
    }

    #[test]
    fn test_list_detection() {
        let analyzer = LayoutAnalyzer::with_default_config();

        let boxes = vec![(
            BoundingBox {
                x: 100.0,
                y: 100.0,
                width: 300.0,
                height: 100.0,
            },
            "• First item\n• Second item\n• Third item".to_string(),
        )];

        let region = Region::from_boxes_with_text(boxes);
        assert!(analyzer.is_list_region(&region));
    }
}
