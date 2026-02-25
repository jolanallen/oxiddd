# Liste des tâches (TODO)

## Fonctionnalités prévues (Roadmap)

### Performance
- [ ] Implémenter le support de BLAKE3 pour un hachage ultra-rapide non-judiciaire.
- [ ] Support de l'accélération matérielle (AVX-512 / SHA-NI) via des intrinsics spécifiques.

### Forensic
- [ ] Ajouter une option de vérification post-écriture (re-lecture complète et comparaison).  **Important**
- [ ] Support du format compressé `.e01` (Expert Witness Format).
- [ ] Génération de rapports au format PDF ou JSON détaillé.

### Interface
- [ ] Ajout d'une option `count=` pour limiter la taille de l'acquisition.
- [ ] Option `skip=` pour démarrer l'acquisition à un offset spécifique.
- [ ] Mode interactif pour lister et sélectionner les périphériques blocs disponibles.

### Robustesse
- [ ] Gestion plus fine des tentatives de relecture (retries) sur secteurs défectueux.
- [ ] Support multi-plateforme complet (vérification O_DIRECT sur Windows/macOS).
