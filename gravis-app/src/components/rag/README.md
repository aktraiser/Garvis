n# RAG Window Components

Ce dossier contient les composants refactorisés pour la fenêtre RAG, organisés de manière modulaire et maintenable.

## Structure

```
rag/
├── types.ts                    # Types TypeScript partagés
├── tabs/
│   ├── DocumentsTab.tsx        # Onglet de gestion des documents
│   └── InjectionTab.tsx        # Onglet d'injection RAG
├── index.ts                    # Exports centralisés
└── README.md                   # Cette documentation
```

## Hooks associés

```
hooks/
├── useDocuments.ts             # Logique de gestion des documents
└── useRagLogic.ts              # Logique RAG et injection
```

## Composants principaux

### DocumentsTab
- **Responsabilité** : Affichage et gestion des documents uploadés
- **Fonctionnalités** :
  - Liste des documents avec métadonnées
  - Extraction de contenu
  - Visualisation des extractions
  - Actions (voir, extraire, supprimer)
  - Préparation pour injection RAG

### InjectionTab
- **Responsabilité** : Gestion du RAG et injection de documents
- **Fonctionnalités** :
  - Recherche dans la base RAG
  - Affichage des documents RAG persistés
  - Configuration d'injection avec métadonnées
  - Modal de paramétrage avancé

## Hooks

### useDocuments
- **État** : documents, loading, notifications, extractions
- **Actions** : upload, delete, extract, view, edit

### useRagLogic
- **État** : recherche RAG, documents RAG, métadonnées d'injection
- **Actions** : rechercher, injecter, supprimer du RAG

## Avantages de cette architecture

1. **Maintenabilité** : Code organisé en petits modules focalisés
2. **Réutilisabilité** : Composants et hooks réutilisables
3. **Testabilité** : Logique séparée des composants UI
4. **Lisibilité** : Fichiers plus petits et mieux organisés
5. **Évolutivité** : Facile d'ajouter de nouvelles fonctionnalités

## Réduction de complexité

- **Avant** : 1 fichier de 2237 lignes
- **Après** : 7 fichiers modulaires (318 lignes pour le composant principal)
- **Réduction** : 85% de la taille du fichier principal