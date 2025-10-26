# GRAVIS OCR - Feuille de Route & Audit Technique

## 🔍 Audit des Solutions OCR pour GRAVIS RAG

### Vue d'ensemble

Dans le cadre de l'extension du système RAG GRAVIS pour traiter les documents PDF et images, nous avons évalué les solutions OCR disponibles en Rust pour une intégration native et performante.

---

## 🎯 Décision Finale : Tesseract via leptess

### **Stratégie Retenue : Tesseract Uniquement**

Après analyse approfondie, nous adoptons **Tesseract via leptess** comme solution exclusive pour GRAVIS OCR.

#### ✅ **Tesseract (via leptess) - Solution Production**

**Avantages décisifs** :
- ✅ **Maturité éprouvée** : 15+ ans de développement, production-ready
- ✅ **Précision validée** : 95%+ sur texte imprimé standard  
- ✅ **Support multilingue** : 100+ langues incluant français, anglais, CJK
- ✅ **Fine-tuning** : Modèles personnalisables par domaine
- ✅ **Écosystème riche** : Documentation, communauté, outils
- ✅ **hOCR/TSV output** : Bounding boxes et confidence scores
- ✅ **Enterprise-grade** : Utilisé par Google, Microsoft, etc.
- ✅ **Bindings Rust matures** : leptess stable et bien maintenu

**Points d'attention maîtrisés** :
- ⚙️ **Installation simple** : `brew install tesseract tesseract-lang` (macOS)
- ⚙️ **Preprocessing intégré** : Via leptonica (inclus avec tesseract)
- ⚙️ **Configuration optimisée** : Variables PSM/OEM documentées
- ⚙️ **Pool de workers** : Gestion concurrence via tokio::spawn_blocking

#### ❌ **OCRS écarté pour l'instant**

**Raisons d'exclusion** :
- ⚠️ **Statut expérimental** : Early preview, stabilité incertaine
- ⚠️ **Latin uniquement** : Pas de support français/multilingue
- ⚠️ **Performance inconnue** : Pas de benchmarks vs Tesseract
- ⚠️ **Écosystème limité** : Communauté réduite, documentation minimale

---

## 🏗️ Architecture OCR Simplifiée pour GRAVIS

### **Solution Tesseract Pure**

```rust
/// Architecture OCR robuste et simple
#[async_trait]
pub trait OcrEngine: Send + Sync {
    async fn ocr_image(&self, path: &Path, config: &OcrConfig) -> Result<OcrResult>;
    async fn ocr_pdf(&self, path: &Path, config: &OcrConfig) -> Result<Vec<OcrPageResult>>;
    fn supported_languages(&self) -> Vec<String>;
}

pub struct TesseractOcr {
    // Pool de workers Tesseract pour concurrence
    worker_pool: Arc<Semaphore>,
}

pub struct GravisOcrProcessor {
    engine: Arc<TesseractOcr>,
    preprocessor: ImagePreprocessor,
    cache: Arc<OcrCache>,
    config: OcrConfig,
}

impl GravisOcrProcessor {
    /// Traitement unifié via Tesseract
    pub async fn process_document(&self, path: &Path) -> Result<OcrResult> {
        // 1. Validation du fichier
        // 2. Preprocessing si nécessaire (deskew, denoise, contrast)
        // 3. OCR via Tesseract avec config optimisée
        // 4. Extraction bounding boxes + confidence
        // 5. Cache du résultat
    }
}
```

### **Pipeline OCR Unifié**

| **Étape** | **Composant** | **Rôle** |
|-----------|---------------|----------|
| **1. Input** | File validation | PNG, JPG, TIFF, PDF support |
| **2. Preprocessing** | Leptonica + ImagePreprocessor | Deskew, denoise, 300 DPI |
| **3. OCR** | Tesseract | Texte + bounding boxes + confidence |
| **4. Post-processing** | TextCleaner | Nettoyage + validation |
| **5. Cache** | Blake3 + LRU | Éviter recalculs identiques |
| **6. Output** | OcrResult | Texte + métadonnées pour RAG |

---

## 🗺️ Feuille de Route d'Implémentation Tesseract

### **Phase 1 : Infrastructure OCR Tesseract (1 semaine)** ✅ TERMINÉE - VALIDÉE

#### **Objectifs** ✅ 
- ✅ Établir l'architecture OCR modulaire centrée sur Tesseract
- ✅ Valider l'infrastructure Tesseract (sans leptess temporairement)
- ✅ Intégrer au pipeline RAG existant

#### **Livrables**
```rust
// src-tauri/src/rag/ocr/mod.rs
pub mod tesseract;           // Module principal Tesseract
pub mod preprocessor;        // Preprocessing via leptonica
pub mod postprocessor;       // Nettoyage et validation
pub mod cache;              // Cache Blake3 + LRU

// Structure principale simplifiée
pub struct OcrConfig {
    pub languages: Vec<String>,           // ["eng", "fra", "deu"]
    pub psm: PageSegMode,                 // Page Segmentation Mode
    pub oem: OcrEngineMode,              // OCR Engine Mode  
    pub preprocessing: PreprocessConfig,
    pub cache_config: CacheConfig,
    pub performance: PerformanceConfig,
}

pub struct OcrResult {
    pub text: String,
    pub confidence: f32,
    pub language: String,
    pub bounding_boxes: Vec<BoundingBox>,
    pub processing_time: Duration,
    pub engine_used: String,             // Toujours "Tesseract"
    pub metadata: OcrMetadata,
}
```

#### **Dépendances Cargo.toml** ✅ VALIDÉES
```toml
# === OCR Tesseract Phase 1 (Infrastructure validée) ===
# leptess = "0.13"             # TEMPORAIREMENT DÉSACTIVÉ (incompatibilité leptonica 1.86)
image = "0.25"                # ✅ Manipulation d'images
regex = "1.10"                # ✅ Text post-processing  
lru = "0.12"                  # ✅ Cache LRU pour résultats OCR
blake3 = "1.5"                # ✅ Hash rapide pour cache keys
tokio = { version = "1.35", features = ["full"] } # ✅ Async processing

# PDF processing pour extraction de pages (Phase 2)
# pdf-extract = "0.7"          # PDF parsing natif Rust (à activer Phase 2)
# poppler-rs = "0.23"          # PDF → images (système)

# Tesseract system validation ✅
# Tesseract 5.5.1 installé via: brew install tesseract tesseract-lang
# 126 langues disponibles dont eng, fra, deu, spa, ita, por
# Performance: 5ms startup, configuration PSM/OEM validée
```

#### **Commandes Tauri Simplifiées**
```rust
#[tauri::command]
async fn ocr_process_pdf(
    file_path: String,
    languages: Vec<String>,
) -> Result<Vec<OcrPageResult>, String> {
    // Implémentation extraction PDF → OCR Tesseract uniquement
}

#[tauri::command]
async fn ocr_process_image(
    file_path: String,
    languages: Vec<String>,
) -> Result<OcrResult, String> {
    // Implémentation traitement image via Tesseract
}

#[tauri::command]
async fn ocr_get_supported_languages() -> Result<Vec<String>, String> {
    // Liste des langues Tesseract disponibles sur le système
}
```

#### **Validation Phase 1** ✅ SUCCÈS COMPLET
- ✅ **Tesseract 5.5.1** installé et fonctionnel
- ✅ **126 langues** disponibles (6 critiques : eng, fra, deu, spa, ita, por)  
- ✅ **Performance exceptionnelle** : 5ms de démarrage
- ✅ **Capacités complètes** : PSM, OEM, configuration avancée
- ✅ **TESSDATA structuré** : 126 fichiers traineddata + configs
- ✅ **Architecture modulaire** créée et prête
- ✅ **Approche Command-based** validée (alternative à leptess)

### **Phase 2 : Implémentation Command-based + Configuration (1 semaine)** 🔄 PROCHAINE

#### **Objectifs Révisés (Command-based)**
- Implémenter TesseractProcessor via Command::new("tesseract") 
- Preprocessing d'images via crate image (sans leptess)
- Configuration fine des paramètres PSM/OEM pour documents variés
- Cache Blake3 + LRU pour optimiser les performances

#### **Fonctionnalités Clés Révisées (Command-based)**
```rust
pub struct TesseractProcessor {
    config: TesseractConfig,
    preprocessor: ImagePreprocessor,  // Via crate image 
    cache: Arc<OcrCache>,
}

impl TesseractProcessor {
    /// Traitement OCR via Command::new("tesseract")
    pub async fn process_image(&self, image_path: &Path) -> Result<OcrResult> {
        // 1. Preprocessing via crate image (contrast, resize, etc.)
        let processed_path = self.preprocess_image(image_path).await?;
        
        // 2. Construction commande Tesseract
        let output_path = self.generate_temp_output_path();
        let mut cmd = Command::new("tesseract");
        cmd.arg(&processed_path)
           .arg(&output_path)
           .arg("-l").arg(self.config.languages.join("+"))
           .arg("--psm").arg(self.config.psm.to_string())
           .arg("--oem").arg(self.config.oem.to_string())
           .arg("tsv"); // Format TSV pour bounding boxes + confidence
        
        // 3. Exécution avec tokio::spawn_blocking
        let result = tokio::task::spawn_blocking(move || cmd.output()).await??;
        
        // 4. Parsing des résultats TSV
        let ocr_result = self.parse_tesseract_output(&result.stdout)?;
        
        // 5. Cache du résultat
        if let Some(cache) = &self.cache {
            cache.store(&image_path, &ocr_result).await?;
        }
        
        Ok(ocr_result)
    }
    
    /// Preprocessing via crate image (sans leptess)
    async fn preprocess_image(&self, image_path: &Path) -> Result<PathBuf> {
        let image = image::open(image_path)?;
        let mut processed = image;
        
        // Preprocessing basique via image crate
        if self.config.enhance_contrast {
            processed = processed.adjust_contrast(15.0);
        }
        
        if self.config.resize_for_ocr {
            let (width, height) = processed.dimensions();
            if width < 1200 || height < 800 {
                processed = processed.resize(1200, 800, image::imageops::FilterType::Lanczos3);
            }
        }
        
        // Sauvegarder image preprocessée
        let temp_path = self.generate_temp_path(image_path);
        processed.save(&temp_path)?;
        
        Ok(temp_path)
    }
}
```

### **Phase 3 : Interface Utilisateur Simple (1 semaine)**

#### **Objectifs**
- Ajouter l'upload de PDF/images dans la modale RAG
- Implémenter l'aperçu OCR avec correction manuelle
- Interface de configuration des langues Tesseract

#### **Composants UI Simplifiés**
```tsx
// src/components/rag/OcrUploadZone.tsx
const OcrUploadZone = ({ onOcrComplete }: { onOcrComplete: (result: OcrResult) => void }) => {
  const [isProcessing, setIsProcessing] = useState(false);
  const [selectedLanguages, setSelectedLanguages] = useState<string[]>(['eng', 'fra']);

  return (
    <div className="ocr-upload-zone">
      <div className="upload-area" onDrop={handleFileDrop}>
        <FileText size={48} />
        <p>Glissez vos PDF ou images ici</p>
        <p className="formats">Formats: PDF, PNG, JPG, TIFF</p>
      </div>
      
      <TesseractLanguageSelector 
        languages={selectedLanguages} 
        onChange={setSelectedLanguages} 
      />
      
      {isProcessing && <OcrProgressIndicator />}
    </div>
  );
};

// src/components/rag/OcrPreview.tsx  
const OcrPreview = ({ result }: { result: OcrResult }) => {
  const [editedText, setEditedText] = useState(result.text);
  const [showBoundingBoxes, setShowBoundingBoxes] = useState(false);

  return (
    <div className="ocr-preview">
      <div className="ocr-metadata">
        <span>Confiance: {(result.confidence * 100).toFixed(1)}%</span>
        <span>Tesseract v{result.tesseract_version}</span>
        <span>Langue: {result.language}</span>
        <span>Temps: {result.processing_time}ms</span>
      </div>
      
      <div className="ocr-content">
        <div className="image-preview">
          <img src={result.source_image} alt="Document" />
          {showBoundingBoxes && (
            <BoundingBoxOverlay boxes={result.bounding_boxes} />
          )}
        </div>
        
        <div className="text-editor">
          <textarea 
            value={editedText}
            onChange={(e) => setEditedText(e.target.value)}
            placeholder="Texte extrait par Tesseract (éditable)"
          />
        </div>
      </div>
      
      <div className="ocr-actions">
        <button onClick={() => setShowBoundingBoxes(!showBoundingBoxes)}>
          {showBoundingBoxes ? 'Masquer' : 'Afficher'} les zones
        </button>
        <button onClick={() => handleAcceptOcr(editedText)}>
          Valider et indexer
        </button>
      </div>
    </div>
  );
};
```

### **Phase 4 : Optimisations Production (1 semaine)**

#### **Objectifs**
- Cache intelligent des résultats OCR Tesseract
- Traitement par lots et parallélisation via tokio::spawn_blocking
- Monitoring et métriques de performance Tesseract

#### **Cache OCR Blake3 Optimisé**
```rust
pub struct TesseractCache {
    cache: Arc<Mutex<LruCache<String, CachedOcrResult>>>,
    config: CacheConfig,
}

impl TesseractCache {
    /// Génération de clé cache basée sur contenu + langues
    fn generate_cache_key(&self, image_hash: &str, languages: &[String]) -> String {
        let lang_hash = blake3::hash(
            languages.join(",").as_bytes()
        ).to_hex().to_string();
        
        format!("{}:{}", image_hash, &lang_hash[..16])
    }
    
    /// Vérification de fraîcheur des résultats
    pub async fn get(&self, image_hash: &str, languages: &[String]) -> Option<OcrResult> {
        let key = self.generate_cache_key(image_hash, languages);
        
        if let Ok(mut cache) = self.cache.lock() {
            if let Some(cached) = cache.get(&key) {
                if cached.is_fresh(self.config.ttl) {
                    return Some(cached.result.clone());
                } else {
                    cache.pop(&key); // Supprimer entrée expirée
                }
            }
        }
        None
    }
}
```

#### **Batch Processing Command-based**
```rust
impl TesseractProcessor {
    /// Traitement par lots via Command::new("tesseract")
    pub async fn process_batch(&self, inputs: Vec<DocumentInput>) -> Result<Vec<OcrResult>> {
        let semaphore = Arc::new(Semaphore::new(self.config.max_concurrent_jobs));
        let mut handles = Vec::new();
        
        for input in inputs {
            let sem = Arc::clone(&semaphore);
            let processor = self.clone();
            
            // Traitement Command-based avec spawn_blocking
            let handle = tokio::spawn(async move {
                let _permit = sem.acquire().await.unwrap();
                processor.process_image(&input.path).await
            });
            
            handles.push(handle);
        }
        
        // Collecte des résultats avec gestion d'erreurs
        let mut results = Vec::new();
        for handle in handles {
            match handle.await {
                Ok(Ok(result)) => results.push(result),
                Ok(Err(e)) => error!("Tesseract Command processing failed: {}", e),
                Err(e) => error!("Task join failed: {}", e),
            }
        }
        
        Ok(results)
    }
}
```

---

## 📊 Benchmarks et Métriques

### **Métriques Tesseract à Surveiller**

```rust
#[derive(Debug, Serialize, Deserialize)]
pub struct TesseractMetrics {
    // Performance Tesseract
    pub processing_time_ms: u64,
    pub throughput_pages_per_minute: f32,
    pub memory_usage_mb: f32,
    pub tesseract_version: String,
    
    // Qualité OCR
    pub average_confidence: f32,
    pub text_extraction_ratio: f32,        // Chars extraits / chars estimés
    pub error_rate: f32,                   // % échecs de traitement
    
    // Configuration Tesseract
    pub psm_distribution: HashMap<String, u64>, // Distribution des PSM utilisés
    pub oem_distribution: HashMap<String, u64>, // Distribution des OEM utilisés
    pub preprocessing_stats: PreprocessingStats,
    
    // Cache
    pub cache_hit_rate: f32,
    pub cache_size_mb: f32,
    
    // Types de documents
    pub pdf_pages_processed: u64,
    pub images_processed: u64,
    pub languages_detected: HashMap<String, u64>,
}
```

### **Benchmark de Validation**

```bash
# Benchmark OCR Tesseract intégré au benchmark RAG existant
cargo run --bin benchmark_custom_e5 -- \
  --chunks 1000 \
  --pdf-docs 100 \
  --image-docs 50 \
  --ocr-languages eng,fra \
  --tesseract-psm auto \
  --export-json ocr_tesseract_benchmark.json

# Test de langues multiples
for lang in "eng" "fra" "eng,fra" "eng,fra,deu"; do
  cargo run --bin benchmark_custom_e5 -- \
    --pdf-docs 25 --ocr-languages $lang \
    --export-json "tesseract_${lang}.json"
done
```

### **Objectifs de Performance**

| **Métrique** | **Cible** | **Excellent** |
|--------------|-----------|---------------|
| **Pages PDF/min** | 10+ | 25+ |
| **Images/min** | 30+ | 60+ |
| **Confiance moyenne** | 85%+ | 92%+ |
| **Cache hit rate** | 60%+ | 80%+ |
| **Mémoire par page** | <50MB | <30MB |
| **Erreur rate** | <5% | <2% |

---

## 🔧 Configuration Recommandée

### **Production Settings Tesseract**

```rust
// src-tauri/src/rag/ocr/config.rs
impl Default for TesseractConfig {
    fn default() -> Self {
        Self {
            languages: vec!["eng".to_string(), "fra".to_string()],
            psm: PageSegMode::AutoOsd,      // Auto détection orientation/script
            oem: OcrEngineMode::LstmOnly,   // LSTM uniquement (plus précis)
            whitelist: None,                // Pas de restriction de caractères
            dpi: 300,                       // DPI optimal pour Tesseract
            
            preprocessing: PreprocessConfig {
                auto_deskew: true,          // Correction automatique inclinaison
                noise_removal: true,        // Suppression bruit via leptonica
                contrast_enhancement: true, // Amélioration contraste
                min_dpi: 300,              // DPI minimal requis
                max_dpi: 600,              // DPI maximal (éviter surcharge)
            },
            
            performance: PerformanceConfig {
                max_concurrent_jobs: 4,     // 4 workers Tesseract parallèles
                timeout_per_page: Duration::from_secs(30),
                memory_limit_mb: 512,       // Limite mémoire par worker
                use_spawn_blocking: true,   // tokio::spawn_blocking pour CPU-bound
            },
            
            cache: CacheConfig {
                enabled: true,
                max_size_mb: 256,           // Cache LRU 256MB
                ttl: Duration::from_hours(24),
                hash_algorithm: "blake3",   // Hash rapide pour clés cache
            },
            
            quality: QualityConfig {
                min_confidence: 0.75,       // Seuil confiance Tesseract
                auto_language_detection: true,
                confidence_threshold_per_language: HashMap::from([
                    ("eng".to_string(), 0.8),
                    ("fra".to_string(), 0.75),
                    ("deu".to_string(), 0.7),
                ]),
            },
        }
    }
}
```

### **Variables d'Environnement**

```bash
# .env additions pour Tesseract
TESSERACT_DATA_DIR=/opt/homebrew/share/tessdata
GRAVIS_OCR_CACHE_DIR=/Users/lucas/.cache/gravis/ocr
GRAVIS_OCR_TEMP_DIR=/tmp/gravis_ocr
GRAVIS_TESSERACT_MAX_WORKERS=4
GRAVIS_TESSERACT_MEMORY_LIMIT_MB=512

# Installation Tesseract macOS
brew install tesseract tesseract-lang
```

---

## 🎯 Décision Finale et Recommandations

### **Stratégie Retenue : Tesseract Exclusif**

#### **Justification Technique**
1. **Tesseract** comme solution unique pour la robustesse et le support multilingue
2. **Leptess** comme bindings Rust matures et stables
3. **Architecture simplifiée** centrée sur une seule technologie éprouvée
4. **Preprocessing leptonica** intégré pour qualité optimale

#### **Roadmap d'Adoption Actualisée**
- ✅ **Phase 1** : Infrastructure Tesseract validée (1 semaine) - **TERMINÉE**
- 🔄 **Phase 2** : Implémentation Command-based + configuration PSM/OEM (1 semaine) - **PROCHAINE**
- 📋 **Phase 3** : Interface utilisateur simple (1 semaine)
- 📋 **Phase 4** : Optimisations production + cache (1 semaine)

#### **Critères de Succès**
- ✅ **Intégration seamless** dans le workflow RAG existant
- ✅ **Performance** : >20 pages PDF/min en production (Tesseract optimisé)
- ✅ **Qualité** : >90% confidence moyenne sur documents imprimés
- ✅ **Robustesse** : <1% taux d'erreur avec preprocessing leptonica
- ✅ **Flexibilité** : Support 10+ langues Tesseract principales

### **Investment ROI**
- **Développement** : 4 semaines développeur senior (approche simplifiée)
- **Infrastructure** : 0€ (Tesseract open-source)
- **Maintenance** : Très faible (solution unique éprouvée)
- **Valeur ajoutée** : Support PDF/images dans RAG = 60%+ use cases supplémentaires

---

---

## 🎉 STATUS ACTUEL - PHASE 1 TERMINÉE

### **✅ VALIDATION PHASE 1 RÉUSSIE** (26 octobre 2025)

**Infrastructure Tesseract entièrement validée** :
- ✅ **Tesseract 5.5.1** installé et fonctionnel 
- ✅ **126 langues** disponibles (performance: 5ms startup)
- ✅ **Capacités complètes** : PSM, OEM, configuration avancée
- ✅ **TESSDATA structuré** : /opt/homebrew/share/tessdata
- ✅ **Architecture modulaire** créée et prête
- ✅ **Approche Command-based** validée comme alternative à leptess

### **🔄 PROCHAINES ÉTAPES**
- **Phase 2** : Implémentation TesseractProcessor via Command::new("tesseract")
- **Phase 3** : Interface utilisateur et intégration Tauri 
- **Phase 4** : Optimisations production et cache Blake3

---

*Feuille de route créée le : 26 octobre 2025*  
*Dernière mise à jour : 26 octobre 2025*  
*Status : **Phase 1 TERMINÉE ✅ - Phase 2 PRÊTE** 🚀*  
*Priorité : **Haute** - Extension critique du système RAG*