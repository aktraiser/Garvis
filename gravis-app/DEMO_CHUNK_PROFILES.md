# 🎬 Démo : Profils de Chunking Intelligents

## 🚀 Comment Tester la Nouvelle Fonctionnalité

### Étape 1 : Lancer l'Application
```bash
cd /Users/lucasbometon/Desktop/voice_flow/gravis/gravis-app
npm run tauri dev
```

---

### Étape 2 : Ouvrir la Fenêtre RAG
1. Dans l'interface Gravis, cliquer sur le bouton **RAG**
2. Aller dans l'onglet **"Injection"**

---

### Étape 3 : Préparer un Document
**Option A** : Si vous avez déjà des documents extraits
- Ils apparaissent dans la section "Documents extraits"
- Cliquer sur **"Injecter"** sur n'importe quel document

**Option B** : Extraire un nouveau document
1. Aller dans l'onglet **"Documents"**
2. Uploader `exemple/2510.18234v1.pdf` (ou autre)
3. Cliquer sur **"Extraire"**
4. Retourner dans l'onglet **"Injection"**
5. Cliquer sur **"Injecter"**

---

### Étape 4 : Découvrir les Profils de Chunking 🎯

**La modale d'injection s'ouvre avec la nouvelle interface !**

Vous verrez maintenant **3 cartes visuelles** au lieu des anciens inputs manuels :

```
┌─────────────────────────────────────────────────────┐
│ 🎯 Précision Maximale                               │
│ Plus de chunks, meilleure précision pour les        │
│ détails                                             │
│                                                     │
│ 256 tokens • 32 overlap • ~40-50 chunks           │
│                                                     │
│ Idéal pour: Questions précises, Documents          │
│ techniques, Recherche de détails                    │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│ ⭐ Équilibré        [RECOMMANDÉ]                    │  ← SÉLECTIONNÉ
│ Configuration optimale pour E5-small-v2             │     PAR DÉFAUT
│ (recommandé)                                        │
│                                                     │
│ 384 tokens • 48 overlap • ~25-30 chunks           │
│                                                     │
│ Idéal pour: Usage général, Mix questions           │
│ larges/précises, Meilleure performance              │
└─────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────┐
│ 📚 Contexte Large                                   │
│ Moins de chunks, meilleur pour les questions       │
│ générales                                           │
│                                                     │
│ 512 tokens • 64 overlap • ~15-20 chunks           │
│                                                     │
│ Idéal pour: Questions larges, Résumés de          │
│ documents, Indexation rapide                        │
└─────────────────────────────────────────────────────┘

ℹ️ Profil sélectionné: Meilleur compromis qualité/performance
```

---

### Étape 5 : Tester Chaque Profil

#### Test A : Profil "Équilibré" (Par défaut)
1. **Laisser** "Équilibré" sélectionné (déjà fait par défaut)
2. Cliquer sur **"Injecter dans le RAG"**
3. Observer dans les logs : `✅ Document injecté : ~25-30 chunks créés`
4. **Noter le nombre de chunks** (exemple : 28 chunks)

---

#### Test B : Profil "Précision Maximale"
1. **Supprimer** le document du RAG :
   - Scroll vers "Documents dans le RAG"
   - Cliquer sur 🗑️ à côté du document
   - Confirmer la suppression

2. **Réinjecter** avec un autre profil :
   - Cliquer à nouveau sur **"Injecter"** sur le même document
   - Dans la modale, **cliquer sur "🎯 Précision Maximale"**
   - Observer : la carte se **highlight en vert**
   - L'encart informatif se met à jour
   - Cliquer sur **"Injecter dans le RAG"**

3. Observer : `✅ Document injecté : ~40-50 chunks créés`
4. **Comparer** : Plus de chunks qu'avec "Équilibré" ! (exemple : 42 chunks vs 28)

---

#### Test C : Profil "Contexte Large"
1. **Supprimer** à nouveau le document
2. **Réinjecter** avec le dernier profil :
   - Cliquer sur **"Injecter"**
   - Sélectionner **"📚 Contexte Large"**
   - Injecter

3. Observer : `✅ Document injecté : ~15-20 chunks créés`
4. **Comparer** : Moins de chunks ! (exemple : 18 chunks vs 28)

---

### Étape 6 : Comparer la Qualité de Recherche

#### Requête de Test : `"explique moi deepseek OCR"`

**Avec "Précision Maximale" (256 tokens)**
```
🔍 5 résultats trouvés

Source 1 : 89.2% - "DeepSeek-OCR consists of DeepEncoder..."
Source 2 : 84.7% - "The architecture includes a vision encoder..."
Source 3 : 78.3% - "Achieves 20x compression ratio..."
Source 4 : 73.1% - "Processes 200,000+ pages per day..."
Source 5 : 69.8% - "Evaluation shows 95% accuracy..."

✅ Scores TRÈS diversifiés (69-89%)
✅ Informations TRÈS spécifiques
```

**Avec "Équilibré" (384 tokens - Recommandé)**
```
🔍 5 résultats trouvés

Source 1 : 87.5% - "DeepSeek-OCR consists of two components..."
Source 2 : 82.1% - "The architecture includes DeepEncoder..."
Source 3 : 76.8% - "DeepSeek-OCR achieves 20x compression..."
Source 4 : 71.2% - "The model processes 200,000+ pages..."
Source 5 : 68.5% - "Evaluation results show high accuracy..."

✅ Scores bien diversifiés (68-87%)
✅ Bon équilibre détails/contexte
```

**Avec "Contexte Large" (512 tokens)**
```
🔍 5 résultats trouvés

Source 1 : 85.3% - "DeepSeek-OCR is an OCR system that uses..."
Source 2 : 80.2% - "The system architecture consists of..."
Source 3 : 75.1% - "Performance evaluation shows that..."
Source 4 : 70.4% - "Applications include document processing..."
Source 5 : 67.2% - "The model handles various document types..."

✅ Scores diversifiés mais moins marqués
✅ Informations plus générales, bon pour résumés
```

---

## 📊 Résultats Attendus

### Nombre de Chunks par Profil
| Document | Précision | Équilibré | Large |
|----------|-----------|-----------|-------|
| DeepSeek PDF (20 pages) | ~42 chunks | ~28 chunks | ~18 chunks |
| Rapport 50 pages | ~105 chunks | ~70 chunks | ~45 chunks |
| Court document 5 pages | ~10 chunks | ~7 chunks | ~5 chunks |

### Qualité de Recherche
| Profil | Diversité Scores | Précision | Vitesse Indexation |
|--------|------------------|-----------|-------------------|
| 🎯 Précision | ⭐⭐⭐⭐⭐ | ⭐⭐⭐⭐⭐ | ⭐⭐⭐ |
| ⭐ Équilibré | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ | ⭐⭐⭐⭐ |
| 📚 Large | ⭐⭐⭐ | ⭐⭐⭐ | ⭐⭐⭐⭐⭐ |

---

## 🎯 Points Clés à Valider

### ✅ Interface
- [ ] Les 3 cartes s'affichent correctement
- [ ] "Équilibré" est sélectionné par défaut avec le badge "RECOMMANDÉ"
- [ ] Cliquer sur une carte la highlight en vert
- [ ] L'encart informatif se met à jour
- [ ] Le design est clair et intuitif

### ✅ Fonctionnalité
- [ ] Le profil par défaut est bien "Équilibré"
- [ ] Changer de profil met à jour `chunkSize` et `chunkOverlap` automatiquement
- [ ] Le nombre de chunks varie selon le profil (~1.5x entre Large et Précision)
- [ ] Le ratio overlap/size reste constant à 12.5% pour tous

### ✅ Performance
- [ ] Précision Maximale : Meilleure diversité des scores
- [ ] Équilibré : Bon compromis
- [ ] Contexte Large : Meilleure vitesse d'indexation

---

## 🐛 Troubleshooting

### Problème 1 : Les profils ne s'affichent pas
**Solution** : Vérifier que TypeScript a compilé
```bash
npx tsc --noEmit
# Devrait afficher "no errors"
```

### Problème 2 : Le profil ne change pas au clic
**Solution** : Vérifier la console navigateur (F12)
- Doit afficher : "Chunk profile changed to: <profile>"

### Problème 3 : Nombre de chunks identique pour tous les profils
**Solution** : Vérifier que le document a bien été supprimé entre les tests
- Utiliser le bouton 🗑️ dans "Documents dans le RAG"

---

## 🎥 Vidéo de Démo (Si applicable)

### Script de Démo (30 secondes)
```
00:00 - Ouverture de la modale d'injection
00:05 - Présentation des 3 profils visuels
00:10 - Sélection de "Précision Maximale"
00:15 - Injection et observation : ~42 chunks créés
00:20 - Suppression et réinjection avec "Contexte Large"
00:25 - Observation : ~18 chunks créés
00:30 - Recherche et comparaison des résultats
```

---

## 📝 Feedback Utilisateur

**Questions à poser après la démo** :
1. L'interface est-elle claire ?
2. Les descriptions de profils sont-elles compréhensibles ?
3. Le profil par défaut ("Équilibré") vous semble-t-il approprié ?
4. Auriez-vous besoin d'un 4ème profil ?
5. Les badges "RECOMMANDÉ" vous aident-ils ?

---

## 🎉 Conclusion

Cette démo montre comment **transformer une configuration technique complexe en un choix simple et guidé**, tout en garantissant des **performances optimales** pour chaque cas d'usage.

**Temps de démo** : 5-10 minutes
**Niveau** : Débutant/Intermédiaire
**Prérequis** : Application Gravis lancée + 1 document PDF

**Date** : 2025-11-07
**Version** : Demo Guide v1.0
**Auteur** : Claude Code
