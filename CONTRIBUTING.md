# Contribuer à oxiddd

Tout d'abord, merci de considérer une contribution à `oxiddd`. En tant qu'outil conçu pour l'investigation numérique et la réponse aux incidents (DFIR), le maintien d'une fiabilité extrême et de l'intégrité des données est notre priorité absolue.

## Principes Directeurs

- **L'intégrité avant tout** : Chaque modification doit garantir que les données du disque ne sont jamais modifiées et que les hachages sont calculés avec précision.
- **La sécurité avant la vitesse** : Bien que la performance soit un objectif, la stabilité du processus d'acquisition est primordiale.
- **Zéro dépendance inutile** : Minimisez l'ajout de nouvelles dépendances. Chaque nouvelle crate doit être auditée pour son impact sur la sécurité et les performances.

## Comment contribuer

### Signaler des bogues

- Utilisez le formulaire de **Rapport de Bug**.
- Incluez la commande exacte utilisée.
- Précisez le contexte matériel (type de disque, méthode de connexion) si nécessaire.

### Proposer des fonctionnalités

- Utilisez le formulaire de **Demande de Fonctionnalité**.
- Expliquez la valeur ajoutée pour le forensic.

### Flux de développement

1.  **Forkez** le dépôt et créez votre branche à partir de `main` (ou `dev` pour les nouvelles fonctionnalités).
2.  **Installez les dépendances** : Assurez-vous d'avoir Rust et `musl-tools` installés.
3.  **Exécutez les tests** : `cargo test` doit passer.
4.  **Linting** : Exécutez `cargo clippy` et `cargo fmt`. Nous n'acceptons pas de code avec des avertissements clippy.
5.  **Pas de `unwrap()`** : Évitez `unwrap()` ou `expect()` sur des opérations potentiellement défaillantes. Gérez les erreurs avec élégance avec une journalisation forensic appropriée.

## Code de conduite

Ce projet adhère à un environnement professionnel et respectueux. En participant, vous êtes tenu de respecter cette norme (voir `CODE_OF_CONDUCT.md`).

## Licence

En contribuant à `oxiddd`, vous acceptez que vos contributions soient sous licence **GPL-3.0**.
