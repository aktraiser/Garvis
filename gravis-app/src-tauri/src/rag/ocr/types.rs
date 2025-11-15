// OCR Types - Compatibility layer between OCR system and Direct Chat
// Re-exports types from direct_chat.rs for use in OCR modules

pub use crate::rag::core::direct_chat::{BoundingBox, OCRBlock, BlockType};

// Extension trait for BoundingBox to add area calculation and convenience methods
pub trait BoundingBoxExt {
    fn area(&self) -> f64;
    fn left(&self) -> f64;
    fn top(&self) -> f64;
}

impl BoundingBoxExt for BoundingBox {
    fn area(&self) -> f64 {
        self.width * self.height
    }

    fn left(&self) -> f64 {
        self.x
    }

    fn top(&self) -> f64 {
        self.y
    }
}
