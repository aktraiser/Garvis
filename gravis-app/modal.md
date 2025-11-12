# API LLM avec vLLM et Modal

Ce projet déploie un modèle de langage Qwen3-8B-FP8 sur Modal avec une API compatible OpenAI utilisant vLLM.

## Table des matières

- [Prérequis](#prérequis)
- [Installation](#installation)
- [Configuration](#configuration)
- [Déploiement](#déploiement)
- [Utilisation](#utilisation)
- [Endpoints disponibles](#endpoints-disponibles)
- [Exemples de code](#exemples-de-code)
- [Gestion des coûts](#gestion-des-coûts)
- [Troubleshooting](#troubleshooting)

## Prérequis

- Python 3.9 ou supérieur
- Un compte Modal (https://modal.com)
- Token Modal (Token ID et Token Secret)

## Installation

1. Cloner ou naviguer vers le répertoire du projet :

```bash
cd /Users/lucasbometon/Desktop/api_llm/Modal
```

2. Créer un environnement virtuel :

```bash
python3 -m venv venv
```

3. Activer l'environnement virtuel :

```bash
source venv/bin/activate
```

4. Installer Modal :

```bash
pip install modal
```

## Configuration

### Authentification Modal

Configurer votre token Modal avec vos identifiants :

```bash
modal token set --token-id <VOTRE_TOKEN_ID> --token-secret <VOTRE_TOKEN_SECRET>
```

### Configuration du modèle

Dans le fichier `vllm_inference.py`, vous pouvez modifier :

- **MODEL_NAME** : Le modèle Hugging Face à utiliser (défaut: `Qwen/Qwen3-8B-FP8`)
- **N_GPU** : Nombre de GPUs H100 à utiliser (défaut: 1)
- **FAST_BOOT** : Mode de démarrage rapide (True) ou performance optimale (False)
- **VLLM_PORT** : Port du serveur vLLM (défaut: 8000)

```python
MODEL_NAME = "Qwen/Qwen3-8B-FP8"
N_GPU = 1
FAST_BOOT = True
```

## Déploiement

### Déploiement sur Modal

Pour déployer l'application sur Modal :

```bash
modal deploy vllm_inference.py
```

Cette commande va :
1. Construire l'image Docker avec CUDA et vLLM
2. Télécharger les poids du modèle
3. Déployer l'application
4. Retourner une URL publique

### Test du déploiement

Pour tester localement le déploiement :

```bash
modal run vllm_inference.py
```

## Utilisation

### URL de l'API

Après le déploiement, votre API est accessible à :

```
https://lbometon2--example-vllm-inference-serve.modal.run
```

### Documentation interactive

Swagger UI disponible à :

```
https://lbometon2--example-vllm-inference-serve.modal.run/docs
```

## Endpoints disponibles

### Health Check

```bash
GET /health
```

Vérifie que le serveur répond correctement.

```bash
curl https://lbometon2--example-vllm-inference-serve.modal.run/health
```

### Chat Completions

```bash
POST /v1/chat/completions
```

Endpoint principal pour les conversations avec le modèle.

**Paramètres** :
- `model` : Nom du modèle (utilisez "llm")
- `messages` : Liste des messages au format OpenAI
- `stream` : Boolean pour activer le streaming (optionnel)
- `temperature` : Température de génération (optionnel, 0.0-2.0)
- `max_tokens` : Nombre maximum de tokens à générer (optionnel)

## Exemples de code

### Avec curl

```bash
curl -X POST "https://lbometon2--example-vllm-inference-serve.modal.run/v1/chat/completions" \
  -H "Content-Type: application/json" \
  -d '{
    "model": "llm",
    "messages": [
      {"role": "system", "content": "Tu es un assistant utile."},
      {"role": "user", "content": "Explique-moi la décomposition en valeurs singulières."}
    ],
    "stream": false
  }'
```

### Avec Python et OpenAI SDK

Installer la bibliothèque OpenAI :

```bash
pip install openai
```

Code Python :

```python
from openai import OpenAI

# Initialiser le client
client = OpenAI(
    base_url="https://lbometon2--example-vllm-inference-serve.modal.run/v1",
    api_key="not-needed"  # L'API ne requiert pas de clé
)

# Envoyer une requête
response = client.chat.completions.create(
    model="llm",
    messages=[
        {"role": "system", "content": "Tu es un assistant utile."},
        {"role": "user", "content": "Bonjour! Comment ça va?"}
    ]
)

print(response.choices[0].message.content)
```

### Streaming avec Python

```python
from openai import OpenAI

client = OpenAI(
    base_url="https://lbometon2--example-vllm-inference-serve.modal.run/v1",
    api_key="not-needed"
)

# Streaming activé
stream = client.chat.completions.create(
    model="llm",
    messages=[
        {"role": "user", "content": "Écris-moi un poème sur l'intelligence artificielle."}
    ],
    stream=True
)

# Afficher les chunks au fur et à mesure
for chunk in stream:
    if chunk.choices[0].delta.content:
        print(chunk.choices[0].delta.content, end="")
```

### Avec aiohttp (async)

```python
import aiohttp
import json

async def query_llm():
    url = "https://lbometon2--example-vllm-inference-serve.modal.run/v1/chat/completions"

    payload = {
        "model": "llm",
        "messages": [
            {"role": "user", "content": "Quelle est la capitale de la France?"}
        ],
        "stream": False
    }

    async with aiohttp.ClientSession() as session:
        async with session.post(url, json=payload) as resp:
            result = await resp.json()
            print(result['choices'][0]['message']['content'])

# Exécuter
import asyncio
asyncio.run(query_llm())
```

### Avec JavaScript/Node.js

```javascript
const OpenAI = require('openai');

const client = new OpenAI({
  baseURL: 'https://lbometon2--example-vllm-inference-serve.modal.run/v1',
  apiKey: 'not-needed'
});

async function main() {
  const completion = await client.chat.completions.create({
    model: 'llm',
    messages: [
      { role: 'user', content: 'Bonjour!' }
    ]
  });

  console.log(completion.choices[0].message.content);
}

main();
```

## Gestion des coûts

### Mise en veille automatique

Le serveur se met automatiquement en veille après **15 minutes** d'inactivité pour réduire les coûts.

Configuration dans le code :

```python
scaledown_window=15 * MINUTES
```

### Cold Start

Lors du premier appel après une période d'inactivité, le démarrage peut prendre quelques minutes :
- Avec `FAST_BOOT=True` : ~30-60 secondes
- Avec `FAST_BOOT=False` : ~2-3 minutes (mais meilleures performances ensuite)

### Volumes Modal

Deux volumes sont utilisés pour le cache :
- `huggingface-cache` : Cache des poids du modèle
- `vllm-cache` : Cache des artefacts de compilation vLLM

Ces volumes persistent entre les déploiements pour accélérer les démarrages.

## Troubleshooting

### Erreur : "modal: command not found"

Assurez-vous que l'environnement virtuel est activé :

```bash
source venv/bin/activate
```

Ou utilisez le chemin complet :

```bash
./venv/bin/modal deploy vllm_inference.py
```

### Timeout lors du déploiement

Le premier déploiement peut prendre 5-10 minutes pour télécharger tous les composants. Augmentez le timeout si nécessaire :

```python
timeout=10 * MINUTES
```

### Erreur 503 ou 504

Le serveur est probablement en train de démarrer (cold start). Attendez 1-2 minutes et réessayez.

### Changer de modèle

Pour utiliser un autre modèle, modifiez dans `vllm_inference.py` :

```python
MODEL_NAME = "meta-llama/Llama-3-8b"  # Exemple
MODEL_REVISION = "main"  # ou un commit spécifique
```

Vérifiez que le modèle est compatible avec vLLM et qu'il tient dans la mémoire GPU (8GB pour un H100).

## Monitoring

### Dashboard Modal

Consultez les logs et métriques sur le dashboard Modal :

```
https://modal.com/apps/lbometon2/main/deployed/example-vllm-inference
```

### Logs en temps réel

```bash
modal logs example-vllm-inference
```

## Support

- Documentation Modal : https://modal.com/docs
- Documentation vLLM : https://docs.vllm.ai
- Issues GitHub Modal : https://github.com/modal-labs/modal-examples

## Structure du projet

```
Modal/
├── venv/                  # Environnement virtuel Python
├── vllm_inference.py      # Script principal de déploiement
└── README.md              # Cette documentation
```

## Licence

Ce projet utilise des technologies open-source. Consultez les licences respectives de Modal, vLLM et Qwen pour plus d'informations.


✅ Tests réussis ! Voici les résultats
Test 1 : Sans paramètres anti-répétition
{
  "max_tokens": 50
}
Résultat : Le thinking a démarré mais s'est arrêté trop tôt (limite de tokens)
Pas de répétition visible, mais texte tronqué
Test 2 : Avec paramètres anti-répétition (modérés)
{
  "temperature": 0.7,
  "frequency_penalty": 0.5,
  "top_p": 0.9,
  "max_tokens": 200,
  "stop": ["</think>"]
}
✅ Le stop: ["</think>"] a correctement arrêté après le thinking
✅ Texte fluide, pas de répétition
✅ 139 tokens générés
Test 3 : Avec paramètres anti-répétition élevés
{
  "temperature": 0.7,
  "frequency_penalty": 1.0,  // Pénalité élevée
  "top_p": 0.9,
  "max_tokens": 300,
  "stop": ["</think>"]
}
✅ Aucune répétition même avec 300 tokens de thinking
✅ Vocabulaire très varié : "cardiovascular", "endorphins", "weight management", "bone density", "osteoporosis", "metabolism", "stamina"
✅ Le modèle raisonne de façon cohérente sans boucler
🎯 Conclusion
Votre API vLLM sur Modal fonctionne parfaitement !
Paramètres recommandés pour éviter les répétitions :
{
    "temperature": 0.7,          # Bon équilibre créativité/cohérence
    "frequency_penalty": 0.5,    # Usage général
    "frequency_penalty": 1.0,    # Si répétitions sévères
    "top_p": 0.9,
    "max_tokens": 300-1000,
    "stop": ["</think>", "\n\n\n"]
}
Fichiers de référence :
📖 README.md - Documentation complète avec exemples
📚 ANTI_REPETITION_GUIDE.md - Guide détaillé des paramètres
🧪 client_example.py - Scripts Python pour tester
Les boucles "the the the..." sont maintenant évitées grâce à frequency_penalty ! 🚀