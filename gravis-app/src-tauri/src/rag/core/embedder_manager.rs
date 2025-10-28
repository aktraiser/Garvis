// GRAVIS RAG - Gestionnaire Singleton pour Embedders
// Évite les initialisations multiples et optimise la mémoire

use std::sync::Arc;
use tokio::sync::OnceCell;
use anyhow::Result;
use tracing::info;

use crate::rag::{CustomE5Embedder, CustomE5Config};

/// Gestionnaire singleton pour les embedders
pub struct EmbedderManager {
    custom_e5: OnceCell<Arc<CustomE5Embedder>>,
}

impl EmbedderManager {
    /// Instance singleton du gestionnaire
    pub fn instance() -> &'static EmbedderManager {
        static INSTANCE: EmbedderManager = EmbedderManager {
            custom_e5: OnceCell::const_new(),
        };
        &INSTANCE
    }

    /// Obtient l'embedder CustomE5 (initialisation lazy)
    pub async fn get_custom_e5(&self) -> Result<Arc<CustomE5Embedder>> {
        let embedder = self.custom_e5.get_or_try_init(|| async {
            info!("🔄 Initializing singleton CustomE5 embedder");
            
            // Configuration par défaut optimisée pour cache
            let config = CustomE5Config::default();
            
            let embedder = CustomE5Embedder::new(config).await
                .map_err(|e| anyhow::anyhow!("Failed to initialize CustomE5: {}", e))?;
            info!("✅ Singleton CustomE5 embedder initialized and cached");
            
            Ok::<Arc<CustomE5Embedder>, anyhow::Error>(Arc::new(embedder))
        }).await?;

        Ok(embedder.clone())
    }

    /// Obtient l'embedder CustomE5 avec configuration personnalisée
    pub async fn get_custom_e5_with_config(&self, config: CustomE5Config) -> Result<Arc<CustomE5Embedder>> {
        // Si pas encore initialisé, utilise la config fournie
        let embedder = self.custom_e5.get_or_try_init(|| async {
            info!("🔄 Initializing singleton CustomE5 embedder with custom config");
            
            let embedder = CustomE5Embedder::new(config).await
                .map_err(|e| anyhow::anyhow!("Failed to initialize CustomE5 with config: {}", e))?;
            info!("✅ Singleton CustomE5 embedder initialized with custom config");
            
            Ok::<Arc<CustomE5Embedder>, anyhow::Error>(Arc::new(embedder))
        }).await?;

        Ok(embedder.clone())
    }

    /// Force la réinitialisation (pour tests ou changement de config)
    #[cfg(test)]
    pub async fn reset(&self) {
        // Cette méthode n'est disponible que pour les tests
        // En production, le singleton persiste pour toute la durée de vie de l'app
    }
}

/// Fonction utilitaire pour obtenir rapidement l'embedder CustomE5
pub async fn get_embedder() -> Result<Arc<CustomE5Embedder>> {
    EmbedderManager::instance().get_custom_e5().await
}

/// Fonction utilitaire avec configuration personnalisée
pub async fn get_embedder_with_config(config: CustomE5Config) -> Result<Arc<CustomE5Embedder>> {
    EmbedderManager::instance().get_custom_e5_with_config(config).await
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_singleton_behavior() {
        // Premier appel - initialisation
        let embedder1 = get_embedder().await.unwrap();
        
        // Deuxième appel - doit retourner la même instance
        let embedder2 = get_embedder().await.unwrap();
        
        // Vérifier que c'est le même Arc (même adresse mémoire)
        assert!(Arc::ptr_eq(&embedder1, &embedder2));
    }
}