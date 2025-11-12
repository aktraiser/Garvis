#!/bin/bash

# Script de nettoyage Qdrant pour Phase 1 RAG
# Supprime la collection default_group pour permettre la réindexation avec le nouveau chunking

echo "🗑️  Nettoyage Qdrant pour Phase 1 RAG"
echo "======================================"
echo ""

# Vérifier que Qdrant est accessible
echo "1️⃣  Vérification de Qdrant..."
if ! curl -s http://localhost:6333/health > /dev/null 2>&1; then
    echo "❌ ERREUR : Qdrant n'est pas accessible sur http://localhost:6333"
    echo "   Veuillez démarrer Qdrant et réessayer"
    exit 1
fi
echo "✅ Qdrant est accessible"
echo ""

# Lister les collections existantes
echo "2️⃣  Collections existantes :"
COLLECTIONS=$(curl -s http://localhost:6333/collections | jq -r '.result.collections[].name' 2>/dev/null)
if [ -z "$COLLECTIONS" ]; then
    echo "   Aucune collection trouvée"
else
    echo "$COLLECTIONS" | while read -r collection; do
        echo "   - $collection"
    done
fi
echo ""

# Supprimer la collection default_group
COLLECTION_NAME="collection_default_group"
echo "3️⃣  Suppression de la collection : $COLLECTION_NAME"

# Vérifier si la collection existe
if curl -s http://localhost:6333/collections/$COLLECTION_NAME > /dev/null 2>&1; then
    # Supprimer la collection
    RESPONSE=$(curl -s -X DELETE http://localhost:6333/collections/$COLLECTION_NAME)

    if echo "$RESPONSE" | jq -e '.result == true' > /dev/null 2>&1; then
        echo "✅ Collection $COLLECTION_NAME supprimée avec succès"
    else
        echo "⚠️  Impossible de supprimer la collection (peut-être déjà supprimée)"
    fi
else
    echo "ℹ️  Collection $COLLECTION_NAME n'existe pas (déjà propre)"
fi
echo ""

# Vérifier que la suppression a fonctionné
echo "4️⃣  Vérification finale..."
COLLECTIONS_AFTER=$(curl -s http://localhost:6333/collections | jq -r '.result.collections[].name' 2>/dev/null)

if echo "$COLLECTIONS_AFTER" | grep -q "$COLLECTION_NAME"; then
    echo "⚠️  ATTENTION : La collection existe toujours !"
    exit 1
else
    echo "✅ Collection supprimée confirmée"
fi
echo ""

echo "======================================"
echo "🎉 Nettoyage terminé !"
echo ""
echo "📋 Prochaines étapes :"
echo "   1. Redémarrer l'application Gravis"
echo "   2. Injecter un document de test"
echo "   3. Observer : ~3x plus de chunks créés"
echo "   4. Tester une recherche"
echo ""
echo "📖 Voir GUIDE_TEST_PHASE1.md pour plus de détails"
