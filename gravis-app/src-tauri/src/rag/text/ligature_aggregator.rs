// Agrégateur global pour les ligatures avec sampling
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::{trace, info};
use once_cell::sync::Lazy;

/// Instance globale de l'agrégateur de ligatures
static LIGATURE_AGGREGATOR: Lazy<Arc<Mutex<LigatureAggregator>>> = 
    Lazy::new(|| Arc::new(Mutex::new(LigatureAggregator::new(500))));

/// Agrégateur global pour collecter et sampler les ligatures
#[derive(Debug)]
pub struct LigatureAggregator {
    counters: HashMap<String, AtomicUsize>,
    sample_rate: usize,
    total_processed: AtomicUsize,
}

impl LigatureAggregator {
    pub fn new(sample_rate: usize) -> Self {
        Self {
            counters: HashMap::new(),
            sample_rate,
            total_processed: AtomicUsize::new(0),
        }
    }
    
    /// Enregistre une ligature détectée avec sampling
    pub fn record_ligature(&mut self, ligature: &str, count: usize, context: &str) {
        let counter = self.counters
            .entry(ligature.to_string())
            .or_insert_with(|| AtomicUsize::new(0));
        
        let previous_count = counter.fetch_add(count, Ordering::Relaxed);
        let total = self.total_processed.fetch_add(count, Ordering::Relaxed);
        
        // Sampling pour logging détaillé en TRACE
        if (previous_count + count) % self.sample_rate == 0 {
            trace!("Ligature '{}' in {}: {} occurrences (sampled, total: {})", 
                   ligature, context, count, previous_count + count);
        }
        
        // Log périodique pour stats globales
        if total > 0 && total % (self.sample_rate * 10) == 0 {
            self.log_intermediate_summary();
        }
    }
    
    /// Log du résumé intermédiaire
    fn log_intermediate_summary(&self) {
        let total = self.total_processed.load(Ordering::Relaxed);
        if total > 0 {
            info!("Ligatures processing: {} total characters processed", total);
        }
    }
    
    /// Génère le résumé final
    pub fn log_final_summary(&self) {
        if self.counters.is_empty() {
            return;
        }
        
        let mut summary_parts = Vec::new();
        let mut total_ligatures = 0;
        
        for (ligature, counter) in &self.counters {
            let count = counter.load(Ordering::Relaxed);
            if count > 0 {
                summary_parts.push(format!("{}={}", ligature, count));
                total_ligatures += count;
            }
        }
        
        if total_ligatures > 0 {
            info!("Ligatures summary: {} total [{}] (sampled=1/{})", 
                  total_ligatures, 
                  summary_parts.join(" "),
                  self.sample_rate);
        }
    }
    
    /// Reset des compteurs
    pub fn reset(&mut self) {
        self.counters.clear();
        self.total_processed.store(0, Ordering::Relaxed);
    }
}

/// Interface globale pour enregistrer des ligatures
pub fn record_ligature_global(ligature: &str, count: usize, context: &str) {
    if let Ok(mut aggregator) = LIGATURE_AGGREGATOR.lock() {
        aggregator.record_ligature(ligature, count, context);
    }
}

/// Interface globale pour log du résumé final
pub fn log_ligature_summary_global() {
    if let Ok(aggregator) = LIGATURE_AGGREGATOR.lock() {
        aggregator.log_final_summary();
    }
}

/// Interface globale pour reset
pub fn reset_ligature_counters_global() {
    if let Ok(mut aggregator) = LIGATURE_AGGREGATOR.lock() {
        aggregator.reset();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ligature_aggregation() {
        let mut aggregator = LigatureAggregator::new(10);
        
        // Simuler détection de ligatures
        aggregator.record_ligature("ﬁ", 5, "doc1");
        aggregator.record_ligature("ﬂ", 3, "doc1");
        aggregator.record_ligature("ﬁ", 7, "doc2");
        
        // Vérifier compteurs
        assert_eq!(aggregator.counters.get("ﬁ").unwrap().load(Ordering::Relaxed), 12);
        assert_eq!(aggregator.counters.get("ﬂ").unwrap().load(Ordering::Relaxed), 3);
        assert_eq!(aggregator.total_processed.load(Ordering::Relaxed), 15);
    }
}