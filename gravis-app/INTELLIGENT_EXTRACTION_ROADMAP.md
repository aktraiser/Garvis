# GRAVIS - Intelligent Content Extraction Roadmap

## 🎯 Objectif
Transformer l'extraction basique actuelle en pipeline intelligent capable de structurer automatiquement tout type de contenu web (articles, e-commerce, tableaux, emails, etc.) pour optimiser l'utilisation par le LLM.

## 📊 État Actuel vs Vision
```
ACTUEL:  Page → DOM/OCR → Texte brut → LLM generique
VISION:  Page → Extraction intelligente → JSON structuré → Template adaptatif → LLM optimisé
```

**Problème identifié:** Le LLM reçoit du texte brut et ne tire pas parti des données structurées disponibles (ex: prix Disneyland Paris extraits mais non utilisés).

---

## 🗂️ PHASE 1: Foundation & Core Intelligence

### 1.1 Extension - Extraction Riche (0.2-0.5s vs 4.5s OCR)

#### ✅ Déjà implémenté
- [x] Service Worker MV3 + popup interface
- [x] Extraction DOM basique + sélection utilisateur
- [x] Communication sécurisée HMAC avec GRAVIS
- [x] Rate limiting et validation nonce

#### 🚧 À implémenter
- [ ] **Mozilla Readability integration** 
  - Contenu principal propre pour articles/blogs/docs
  - Suppression navigation/footer/ads automatique
  - Lib: `@mozilla/readability`

- [ ] **JSON-LD/Microdata parsing**
  - Extraction données structurées natives (Product, Article, Recipe, Event)
  - Lib: `jsonld` (Digital Bazaar) + `microdata-node`
  - Support Schema.org automatique

- [ ] **Table-to-JSON conversion**
  - Catalogues, pricing, specs → JSON structuré
  - Lib: `table-to-json` 
  - Détection qualité (min colonnes, équilibre)

- [ ] **Page type detection heuristique**
  - `commerce` : prix/devise/panier détectés
  - `article` : article/blog/news patterns
  - `table_dataset` : tables dominantes
  - `email_like` : webmail/CRM patterns
  - `generic` : fallback

#### 📤 Payload structuré cible
```json
{
  "page_type": "product|article|table_dataset|email_like|generic",
  "url": "https://...",
  "title": "...",
  "main_text": "...",           // Readability clean
  "structured": {
    "jsonld": [...],            // Schema.org si dispo
    "microdata": {...}          // Microdata si dispo
  },
  "tables": [
    {"headers": [...], "rows": [...]}
  ],
  "meta": {
    "byline": "...",
    "published": "2025-11-02T09:00:00Z",
    "language": "fr"
  },
  "extraction_confidence": 0.85
}
```

### 1.2 GRAVIS Backend - Classification & Templates

#### 🚧 Pipeline de traitement
- [ ] **Content type classifier**
  - Analyse payload extension → classification finale
  - Regex + contexte pour affiner la détection
  - Validation des heuristiques extension

- [ ] **Template système adaptatif**
  ```rust
  match content_type {
      "commerce" => generate_commerce_prompt(structured_data),
      "article" => generate_article_prompt(structured_data), 
      "table_dataset" => generate_table_prompt(structured_data),
      "email_like" => generate_email_prompt(structured_data),
      _ => generate_generic_prompt(structured_data)
  }
  ```

- [ ] **Crates Rust à intégrer**
  - `scraper` : sélecteurs CSS fallback
  - `json-ld` : normalisation côté backend (optionnel)
  - `regex` : patterns detection avancée

---

## 🗂️ PHASE 2: Templates Intelligents Spécialisés

### 2.1 Commerce Pipeline
```
JSON-LD Product → {title, price, currency, specs[], availability}
→ Prompt: "Analyse ce produit. Prix compétitif ? Spécifications manquantes ? Alternatives ?"
```

### 2.2 Article Pipeline  
```
Readability + JSON-LD Article → {headline, byline, published, key_points[]}
→ Prompt: "Résume en 5 points + 3 citations + 3 questions critiques"
```

### 2.3 Table Dataset Pipeline
```
table-to-json → {columns[], rows[], insights}
→ Prompt: "3 insights + 2 outliers + export CSV ?"
```

### 2.4 Email-like Pipeline
```
Heuristiques → {from, to, subject, date, body}
→ Prompt: "Résume + action items + deadlines + contacts"
```

---

## 🗂️ PHASE 3: Advanced Features

### 3.1 SPA & Dynamic Content
- [ ] Navigation detection (pushState/replaceState)
- [ ] Auto-extraction on route change
- [ ] Shadow DOM traversal
- [ ] iframe content access

### 3.2 Multi-format Support
- [ ] PDF text extraction (pdf.js)
- [ ] Image OCR fallback (Tesseract.js)
- [ ] Video transcript extraction
- [ ] Audio transcript (Web Speech API)

### 3.3 Smart Caching
- [ ] Content similarity detection
- [ ] Incremental updates
- [ ] Offline extraction queue

---

## 🗂️ PHASE 4: AI-Powered Enhancements

### 4.1 Semantic Extraction
- [ ] Named Entity Recognition (NER)
- [ ] Sentiment analysis
- [ ] Topic classification
- [ ] Intent detection

### 4.2 Cross-page Intelligence
- [ ] Site-wide pattern learning
- [ ] Multi-page data aggregation
- [ ] Relationship mapping

---

## 🛡️ Guardrails & Security

### Privacy & Compliance
- [x] Local-only processing (no cloud)
- [x] HMAC signed payloads
- [x] Per-site consent management
- [ ] GDPR compliance mode
- [ ] Paywall respect (no circumvention)

### Performance
- [ ] 50KB content limit
- [ ] Streaming pour gros datasets
- [ ] Background processing
- [ ] Memory optimization

---

## 📈 Success Metrics

### Quantitatifs
- **Vitesse**: DOM (0.2-0.5s) vs OCR (4.5s) ✅
- **Précision**: 90%+ structured data extraction
- **Couverture**: Support 95% sites populaires
- **Coût LLM**: -50% tokens via prompts structurés

### Qualitatifs  
- **UX**: LLM utilise activement les données extraites
- **Fiabilité**: Moins d'hallucinations
- **Polyvalence**: Fonctionne sur tout type de contenu

---

## 🚀 Quick Wins Immédiats

1. **Mozilla Readability** → Qualité texte +80%
2. **JSON-LD parsing** → Données structurées gratuites 
3. **Table extraction** → Catalogues/prix actionables
4. **Type detection** → Templates adaptatifs

---

## 📋 Prochaines Étapes

### Spike Kit E2E (1-2 jours)
1. Readability + JSON-LD dans extension
2. Pipeline classification Rust
3. Templates adaptatifs de base
4. Test complet: Disneyland → structured pricing → LLM utilisation

### Validation (3-5 jours)
1. Test sur 10 sites différents par catégorie
2. Mesure amélioration qualité réponses LLM
3. Benchmark performance vs solution actuelle
4. Feedback utilisateur

### Production (1 semaine)
1. Error handling robuste
2. Fallbacks gracieux
3. Monitoring & télémétrie
4. Documentation utilisateur

---

*Dernière mise à jour: 2 novembre 2025*
*Status: Phase 0 Complete ✅ | Phase 1 Planning 🚧*