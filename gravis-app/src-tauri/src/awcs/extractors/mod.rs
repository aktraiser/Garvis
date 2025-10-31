// GRAVIS AWCS - Extractors Module
// Extracteurs spécialisés pour différentes sources

pub mod window_detector;
pub mod dom_extractor;
pub mod applescript_extractor;
pub mod accessibility_extractor;
pub mod ocr_extractor;

// Re-exports
pub use window_detector::WindowDetector;
pub use dom_extractor::DOMExtractor;
pub use applescript_extractor::AppleScriptExtractor;
pub use accessibility_extractor::AccessibilityExtractor;
pub use ocr_extractor::OCRExtractor;