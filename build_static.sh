#!/bin/bash
# Script pour compiler oxiddd en binaire statique

set -e

echo "[*] Compilation statique pour Linux (x86_64-unknown-linux-musl)"
# Ajouter la target musl si elle n'est pas installée
rustup target add x86_64-unknown-linux-musl || true

# Compiler avec musl pour garantir 0 dépendance dynamique (pas de glibc requise)
cargo build --release --target x86_64-unknown-linux-musl

echo "[*] Le binaire statique Linux est généré dans :"
echo "    target/x86_64-unknown-linux-musl/release/oxiddd"
echo "[*] Vous pouvez vérifier avec 'ldd target/x86_64-unknown-linux-musl/release/oxiddd'"
echo "    (Il devrait afficher 'not a dynamic executable')"

echo ""
echo "[*] Pour compiler statiquement sous Windows, la configuration est déjà prête dans .cargo/config.toml."
echo "    Lancez simplement 'cargo build --release' depuis un environnement Windows MSVC."
