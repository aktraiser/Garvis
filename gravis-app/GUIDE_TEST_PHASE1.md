# Guide de Test - Phase 1 RAG Improvements 🧪

## 🚀 Comment tester les améliorations

### Étape 1 : Rebuild l'application

```bash
cd /Users/lucasbometon/Desktop/voice_flow/gravis/gravis-app

# Rebuild Tauri
npm run tauri build

# OU en mode dev
npm run tauri dev
```

---

### Étape 2 : Nettoyer la base RAG existante

⚠️ **IMPORTANT** : Les anciens documents utilisent le chunking 1024 tokens (obsolète).

**Dans l'interface Gravis** :
1. Aller dans l'onglet RAG
2. Lister tous les documents : `list_rag_documents`
3. Supprimer TOUS les documents un par un
4. Vérifier que Qdrant est vide

**Alternative via CLI** :
```bash
# Si vous avez accès direct à Qdrant
curl -X DELETE http://localhost:6333/collections/collection_default_group
```

---

### Étape 3 : Réinjecter un document de test

**Document recommandé** : `2510.18234v1.pdf` (DeepSeek OCR)

**Procédure** :
1. Extraire le PDF via OCR (comme d'habitude)
2. Injecter le document dans le RAG
3. **Observer** : Nombre de chunks créés devrait être ~3x plus élevé qu'avant

**Avant Phase 1** :
```
✅ Document injecté : 8 chunks créés
```

**Après Phase 1** (attendu) :
```
✅ Document injecté : 24-30 chunks créés
```

---

### Étape 4 : Tester une requête

**Requête de test** :
```
"explique moi deepseek OCR"
```

#### Résultats AVANT Phase 1 :
```
📊 5 résultats trouvés

[Source 1] Score: 70.3%
Fichier: 2510.18234v1.pdf
Catégorie: Mixed
Contenu: Abstract We present DeepSeek-OCR as an initial...

[Source 2] Score: 70.3%
Fichier: 2510.18234v1.pdf
Catégorie: Mixed
Contenu: Abstract We present DeepSeek-OCR as an initial...

[Source 3] Score: 70.2%
Fichier: 2510.18234v1.pdf
Catégorie: Mixed
Contenu: Abstract We present DeepSeek-OCR as an initial...

❌ PROBLÈME : Toutes les sources sont identiques !
```

#### Résultats APRÈS Phase 1 (attendu) :
```
📊 5 résultats trouvés

[Source 1] Score: 87.5%
Fichier: 2510.18234v1.pdf
Catégorie: Mixed
Contenu: DeepSeek-OCR consists of two components: DeepEncoder...

[Source 2] Score: 82.1%
Fichier: 2510.18234v1.pdf
Catégorie: Mixed
Contenu: The architecture includes a DeepEncoder for vision...

[Source 3] Score: 76.8%
Fichier: 2510.18234v1.pdf
Catégorie: Mixed
Contenu: DeepSeek-OCR achieves 20x compression ratio while...

[Source 4] Score: 71.2%
Fichier: 2510.18234v1.pdf
Catégorie: Mixed
Contenu: The model processes 200,000+ pages per day...

[Source 5] Score: 68.5%
Fichier: 2510.18234v1.pdf
Catégorie: Mixed
Contenu: Evaluation shows 95% accuracy on text recognition...

✅ RÉSULTAT : Sources DIVERSIFIÉES et COMPLÉMENTAIRES !
```

---

### Étape 5 : Vérifier la réponse du LLM

**AVANT Phase 1** :
```
🧠 Réflexion du modèle

DeepSeek OCR est un système qui... [répète 5x la même info de l'abstract]

📚 Sources RAG (5 chunks en 196ms)
Source 1: 70.3% - Abstract We present...
Source 2: 70.3% - Abstract We present...
Source 3: 70.2% - Abstract We present...
...
```

**APRÈS Phase 1 (attendu)** :
```
🧠 Réflexion du modèle

DeepSeek OCR est un système de reconnaissance optique de caractères innovant [Source 1].

## Architecture
Le système utilise deux composants principaux [Source 2]:
- **DeepEncoder** : Encoder vision qui compresse les images en tokens
- **DeepSeek-3B-MoE** : Décodeur de langage avec architecture MoE

## Performance
DeepSeek-OCR atteint un ratio de compression de 20:1 tout en maintenant
une précision de 95% [Source 3]. Le système peut traiter plus de 200 000
pages par jour sur un seul GPU H100 [Source 4].

## Cas d'usage
Applications incluent la numérisation de documents, l'extraction de données
structurées, et l'indexation de corpus massifs [Source 5].

📚 Sources RAG (5 chunks en 196ms)
Source 1: 87.5% - DeepSeek-OCR consists of...
Source 2: 82.1% - The architecture includes...
Source 3: 76.8% - achieves 20x compression...
...

✅ RÉSULTAT : Réponse SYNTHÉTIQUE avec infos COMPLÉMENTAIRES !
```

---

## 📊 Métriques à valider

### 1. Chunking
| Métrique | Avant | Après | Status |
|----------|-------|-------|--------|
| Chunks/doc | ~8-12 | ~24-36 | ⬜ À vérifier |
| Taille chunk | ~1024 tokens | ~384 tokens | ✅ Confirmé |
| Overlap | 128 tokens | 48 tokens | ✅ Confirmé |

### 2. Diversité des scores
| Métrique | Avant | Après | Status |
|----------|-------|-------|--------|
| Score source 1 | 70.3% | 85-90% | ⬜ À vérifier |
| Score source 5 | 70.0% | 65-75% | ⬜ À vérifier |
| Écart min/max | <1% | 15-20% | ⬜ À vérifier |

### 3. Qualité des réponses
| Critère | Avant | Après | Status |
|---------|-------|-------|--------|
| Redondance | Très haute | Basse | ⬜ À vérifier |
| Complétude | Moyenne | Élevée | ⬜ À vérifier |
| Citations uniques | 1 source | 3-5 sources | ⬜ À vérifier |

---

## 🧪 Tests avancés

### Test 1 : Vérifier les préfixes E5

**Backend logs** à surveiller :
```rust
// Lors de l'indexation
🧮 Generating embeddings for 24 chunks
[INFO] Encoding with prefix: "passage: <content>"

// Lors de la recherche
🔍 Searching RAG for: "explique moi deepseek"
[INFO] Encoding with prefix: "query: explique moi deepseek"
```

### Test 2 : Vérifier la normalisation L2

**Code de test** :
```rust
#[tokio::test]
async fn test_embedding_normalized() {
    let embedder = CustomE5Embedder::new(CustomE5Config::default()).await.unwrap();
    let emb = embedder.encode("test").await.unwrap();

    let norm: f32 = emb.iter().map(|x| x*x).sum::<f32>().sqrt();

    println!("Norme L2: {}", norm);
    assert!((norm - 1.0).abs() < 1e-4, "Embedding non normalisé !");
}
```

### Test 3 : Benchmark avant/après

```bash
# Créer un script de benchmark
cat > benchmark_rag.sh << 'EOF'
#!/bin/bash

echo "=== BENCHMARK RAG PHASE 1 ==="
echo ""

# 1. Injection
echo "1. Injection document..."
START=$(date +%s%3N)
# Votre commande d'injection ici
END=$(date +%s%3N)
INJECTION_TIME=$((END - START))
echo "   Temps: ${INJECTION_TIME}ms"

# 2. Recherche
echo "2. Recherche RAG..."
START=$(date +%s%3N)
# Votre commande de recherche ici
END=$(date +%s%3N)
SEARCH_TIME=$((END - START))
echo "   Temps: ${SEARCH_TIME}ms"

# 3. Résumé
echo ""
echo "=== RÉSUMÉ ==="
echo "Injection: ${INJECTION_TIME}ms"
echo "Recherche: ${SEARCH_TIME}ms"
EOF

chmod +x benchmark_rag.sh
./benchmark_rag.sh
```

---

## ❓ Troubleshooting

### Problème 1 : Toujours des résultats redondants

**Cause probable** : Documents anciens pas supprimés

**Solution** :
```bash
# Supprimer complètement la collection Qdrant
curl -X DELETE http://localhost:6333/collections/collection_default_group

# Vérifier qu'elle est supprimée
curl http://localhost:6333/collections

# Réinjecter les documents
```

### Problème 2 : Moins de chunks que prévu

**Cause probable** : Document trop court ou chunking pas appliqué

**Vérification** :
```rust
// Dans les logs, chercher :
📊 Smart chunking created X chunks (avg: Y chars, detected Z sections)

// X devrait être ~3x plus élevé qu'avant
```

### Problème 3 : Scores tous identiques

**Cause probable** : Embeddings pas re-générés

**Solution** :
1. Vérifier que `encode_document` est bien appelé (logs)
2. Vérifier le cache embeddings (peut-être vider)
3. Restart l'application Tauri

### Problème 4 : LLM répète toujours les infos

**Cause probable** : Prompt système pas appliqué

**Vérification** :
```bash
# Dans les logs Tauri, chercher :
✅ RAG context prepared: X chunks, Y sources, Zms

# Le contexte doit contenir le nouveau prompt avec:
"**INSTRUCTIONS POUR RÉPONDRE**:"
"1. **Analyse et synthèse**: ..."
```

---

## 📝 Checklist finale

Avant de valider la Phase 1 :

- [ ] Code compile sans erreurs (`cargo check`)
- [ ] Tests unitaires passent (`cargo test`)
- [ ] Application se lance (`npm run tauri dev`)
- [ ] Documents anciens supprimés
- [ ] Nouveau document injecté avec ~3x plus de chunks
- [ ] Recherche retourne des scores diversifiés (65-90%)
- [ ] Réponse LLM synthétique et non redondante
- [ ] Logs montrent les préfixes E5 (`query:` et `passage:`)
- [ ] Normalisation L2 active (norme ≈ 1.0)

---

## 🎯 Résultat attendu

**Si tout fonctionne correctement** :

✅ Chunks plus petits et cohérents
✅ Scores diversifiés (pas tous à 70%)
✅ Réponses synthétiques sans redondance
✅ Citations de sources variées
✅ Performance globale améliorée de 2-3x

**Temps de validation estimé** : 15-30 minutes

---

## 🔜 Prochaines étapes

Si Phase 1 validée avec succès :

1. **Phase 2A** : Implémenter MMR re-ranking (3h)
2. **Phase 2B** : Ajouter cross-encoder (1 jour)
3. **Phase 3** : Hybrid search BM25 + Vector (2 jours)

---

**Date** : 2025-11-07
**Version** : Phase 1 - Guide de Test
**Auteur** : Claude Code
