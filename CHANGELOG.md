# Journal des modifications (Changelog)

Toutes les modifications notables de ce projet seront répertoriées dans ce fichier.

## [0.1.0] - 2026-02-25

### Ajouté
- Implémentation initiale du moteur de copie multi-threadé en Rust.
- Support du hachage double (Standard et Forensic Binding).
- Support des algorithmes SHA-256 et SHA-512.
- Intégration du client NTP pour horodatage certifié via Google.
- Support de l'I/O direct (O_DIRECT) pour l'optimisation des performances.
- Gestion du padding par zéros en cas d'erreurs de lecture secteurs.
- Script de build statique pour Linux (musl).
- Tests unitaires pour la logique de hachage et de manipulation de chemins.
