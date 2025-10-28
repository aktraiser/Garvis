// Module pour le nettoyage et logging des ligatures avec sampling
use std::collections::HashMap;
use std::sync::atomic::{AtomicUsize, Ordering};
use tracing::{trace, info};

/// Gestionnaire de ligatures avec comptage et sampling
#[derive(Debug)]
pub struct LigatureCleaner {
    counters: HashMap<String, AtomicUsize>,
    sample_rate: usize, // 1 sur N ligatures sera loggée en détail
}

impl Default for LigatureCleaner {
    fn default() -> Self {
        Self::new(200) // Sample 1/200 ligatures
    }
}

impl LigatureCleaner {
    pub fn new(sample_rate: usize) -> Self {
        Self {
            counters: HashMap::new(),
            sample_rate,
        }
    }
    
    /// Nettoie les ligatures et génère des logs avec sampling
    pub fn clean_and_log(&mut self, text: &str, context: &str) -> String {
        let ligature_patterns = [
            ("ﬁ", "fi"),
            ("ﬂ", "fl"), 
            ("ﬀ", "ff"),
            ("ﬃ", "ffi"),
            ("ﬄ", "ffl"),
            ("ﬆ", "st"),
            ("Ａ", "A"), // Fullwidth A
            ("ａ", "a"), // Fullwidth a
        ];
        
        let mut cleaned_text = text.to_string();
        let mut _total_replacements = 0;
        
        for (ligature, replacement) in &ligature_patterns {
            let count = cleaned_text.matches(ligature).count();
            if count > 0 {
                // Mise à jour du compteur global
                let key = ligature.to_string();
                let counter = self.counters.entry(key.clone()).or_insert_with(|| AtomicUsize::new(0));
                let previous_count = counter.fetch_add(count, Ordering::Relaxed);
                
                // Sampling pour logging détaillé en TRACE
                if (previous_count + count) % self.sample_rate == 0 {
                    trace!("Ligature '{}' → '{}' in {}: {} occurrences (sampled)", 
                           ligature, replacement, context, count);
                }
                
                cleaned_text = cleaned_text.replace(ligature, replacement);
                _total_replacements += count;
            }
        }
        
        // Ne log plus les détails individuels, seul le résumé
        
        cleaned_text
    }
    
    /// Génère un résumé des ligatures trouvées
    pub fn log_summary(&self) {
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
            info!("Ligatures summary: {} total [{}] sampled=1/{}", 
                  total_ligatures, 
                  summary_parts.join(" "),
                  self.sample_rate);
        }
    }
    
    /// Reset des compteurs
    pub fn reset(&mut self) {
        self.counters.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ligature_cleaning() {
        let mut cleaner = LigatureCleaner::new(1); // Sample tout pour les tests
        
        let input = "Test with ﬁrst and ﬂow ligatures";
        let result = cleaner.clean_and_log(input, "test_context");
        
        assert_eq!(result, "Test with first and flow ligatures");
        
        // Vérifier les compteurs
        assert_eq!(cleaner.counters.get("ﬁ").unwrap().load(Ordering::Relaxed), 1);
        assert_eq!(cleaner.counters.get("ﬂ").unwrap().load(Ordering::Relaxed), 1);
    }
    
    #[test]
    fn test_summary_logging() {
        let mut cleaner = LigatureCleaner::new(10);
        
        // Simuler plusieurs nettoyages
        cleaner.clean_and_log("ﬁﬁﬁ", "doc1");
        cleaner.clean_and_log("ﬂﬂ", "doc2");
        
        // Le summary devrait contenir fi=3 fl=2
        cleaner.log_summary();
    }
}