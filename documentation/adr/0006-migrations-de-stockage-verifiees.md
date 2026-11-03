# ADR 0006 — Prouver chaque montée de version qui touche au stockage ou à la crypto

- **Statut** : acceptée, novembre 2026 — porte la version de libSQL de l'[ADR 0004](0004-report-migration-turso.md) à 0.10

## Contexte

Les données des utilisateurs ne vivent que sur leur machine, chiffrées : un
keystore scellé par une clé d'appareil (trousseau OS), un coffre libSQL chiffré
par une DEK enveloppée (Argon2id + XChaCha20-Poly1305), et une empreinte
HMAC-SHA256 d'intégrité. Aucune migration manuelle n'est possible : si une
dépendance change un format sur disque, le coffre devient illisible.

La mise à jour de novembre 2026 touchait toutes ces briques : libSQL 0.9 → 0.10,
la génération « digest 0.11 » de RustCrypto (argon2 0.6, chacha20poly1305 0.11,
hmac 0.13, sha2 0.11) et keyring 3 → keyring-core 1.

## Décision

Une montée de version qui touche au stockage ou à la crypto se fait en trois
temps, chacun dans son propre commit :

1. **Figer l'existant avant** : des tests à valeurs connues écrits et commités
   avec les anciennes crates.
   - `infrastructure/crypto/compat_vectors.rs` : hash PHC Argon2id, dérivation
     de la KEK, enveloppes v1/v2, SHA-256 des jetons, base64.
   - `vault_integrity::tests::mac_formats_are_unchanged` : MAC v1/v2.
   - `integration_tests::vault_written_by_libsql_0_9_still_opens` : une vraie
     paire keystore + coffre écrite par libSQL 0.9.30
     (`src-tauri/tests/fixtures/vault-libsql-0.9/`, scellée par la clé de test),
     ouverte en `IntegrityPolicy::HardFail`, relue, modifiée puis rouverte.
2. **Monter la version** : les mêmes tests doivent passer **sans être modifiés**.
3. Pour ce qui dépend de l'OS (trousseau), vérifier dans le code des deux
   versions que la recherche se fait sur les mêmes coordonnées, et sur une
   machine réelle quand c'est possible (`device_key::tests::reads_existing_device_key`,
   ignoré par défaut, lecture seule).

Pour keyring, l'intégration suit la recommandation des mainteneurs depuis
keyring 4 : `keyring-core` et un store natif par plateforme (les mêmes que la
façade `v1`), construits directement, sans store global.

## Conséquences

- Les fixtures et les vecteurs ne se régénèrent jamais pour faire passer une
  montée de version : un échec signifie que les données existantes ne
  s'ouvriraient plus.
- Une nouvelle génération de format (ex. migration vers `turso`/AEGIS) ajoute ses
  propres fixtures, à côté des anciennes, et un chemin de migration testé.
