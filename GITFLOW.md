# Flux de Travail Git & Stratégie de Branches (GitFlow)

Pour maintenir les standards de rigueur requis pour un outil d'investigation numérique (forensics), `oxiddd` suit une stratégie de branches structurée basée sur un modèle **GitFlow** léger.

## 1. Branches Permanentes

- **`main`** : La branche stable. Elle contient le code prêt pour la production. Chaque commit sur `main` doit être marqué par un tag de version.
- **`develop`** (Optionnel pour l'instant) : Si l'équipe s'agrandit, nous introduirons une branche `develop` pour l'intégration continue avant le passage sur `main`.

## 2. Branches Temporaires (Éphémères)

### Branches de Fonctionnalités (`feature/`)
- **But** : Nouvelles fonctionnalités ou éléments de la roadmap.
- **Base** : `main`
- **Fusion vers** : `main` via une Pull Request (PR).
- **Nommage** : `feature/description-courte` (ex: `feature/support-blake3`).

### Branches de Correction (`fix/` ou `bugfix/`)
- **But** : Correction de bugs rapportés dans les issues.
- **Base** : `main`
- **Fusion vers** : `main` via une Pull Request.
- **Nommage** : `fix/id-issue-description` (ex: `fix/12-macos-compilation`).

### Branches de Release (`release/`)
- **But** : Préparation d'une nouvelle version (tests finaux, montée de version).
- **Base** : `main`
- **Fusion vers** : `main`.
- **Nommage** : `release/vX.Y.Z` (ex: `release/v0.2.0`).

---

## 3. Cycle de Développement d'une Tâche

Voici la procédure à suivre pour démarrer un élément du **Kanban** :

### Étape 1 : Préparation sur GitHub
1. Allez sur l'onglet **Projects** de votre dépôt.
2. Choisissez une tâche dans la colonne **Todo** et déplacez-la vers **In Progress**.

### Étape 2 : Création de la branche locale
Synchronisez votre dépôt et créez votre branche de travail :
```bash
git checkout main
git pull origin main
git checkout -b feature/nom-de-la-feature
```

### Étape 3 : Implémentation & Validation Locale
Développez votre fonctionnalité et assurez-vous qu'elle passe les contrôles de qualité :
```bash
cargo fmt                # Formatage automatique du code
cargo clippy             # Analyse statique (0 avertissement toléré)
cargo test               # Exécution des tests unitaires et d'intégration
```

### Étape 4 : Commit & Push
Utilisez des messages de commit clairs et descriptifs :
```bash
git add .
git commit -m "feat: ajout du support du hachage BLAKE3"
git push origin feature/nom-de-la-feature
```

### Étape 5 : Pull Request & CI
1. Ouvrez une **Pull Request** sur GitHub.
2. Remplissez la **Checklist Forensic** dans le template de PR (vérification d'intégrité).
3. Attendez que la **CI (GitHub Actions)** passe au vert.

### Étape 6 : Fusion (Merge)
Une fois la PR approuvée et la CI validée :
1. Fusionnez la PR dans `main`.
2. Supprimez la branche distante.
3. Déplacez la tâche du Kanban vers **Done**.

---

## 4. Publication d'une Version (Release)

Quand le code sur `main` est prêt pour une nouvelle version :

1. **Créer le tag de version** :
   ```bash
   git tag -a v0.2.0 -m "Release v0.2.0 : Support multi-plateforme et corrections CI"
   ```
2. **Pousser le tag** :
   ```bash
   git push origin v0.2.0
   ```
3. **Release Automatisée** : Le workflow `release.yml` va automatiquement :
   - Compiler les binaires pour Linux, macOS et Windows.
   - Créer une Release GitHub officielle.
   - Attacher les exécutables compressés (.tar.gz).

---

## 5. Correctifs de Sécurité (Hotfix)
En cas de faille critique d'intégrité, créez une branche `hotfix/` directement à partir du dernier tag stable, appliquez le correctif, et fusionnez immédiatement vers `main`.
