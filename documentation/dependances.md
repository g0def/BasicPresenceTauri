# Dépendances : politique et état

## Politique

- **Âge minimum de 7 jours** pour toute version installée : `minimumReleaseAge`
  (pnpm), `cooldown` (Dependabot) ; côté Rust, vérifié à la main lors des mises à
  jour (crates.io ne propose pas ce filtre). Une version trop récente est
  remplacée par la plus récente qui a 7 jours.
- **Versions npm épinglées exactement**, overrides de sécurité épinglés aussi
  (`pnpm-workspace.yaml`) ; actions GitHub épinglées par SHA, image Docker de
  `arch-repo.yml` par digest.
- **Stockage et crypto** : chaque montée de version est prouvée par des tests
  figés avant la mise à jour ([ADR 0006](adr/0006-migrations-de-stockage-verifiees.md)).
- **Une exception d'audit** (`deny.toml`, `.cargo/audit.toml`) dit pourquoi la
  faille n'est pas atteignable, et quand la retirer.

## État après la mise à jour de novembre 2026

Outillage : Node 24 LTS, pnpm 12, TypeScript 6, ESLint 10 (règles React
Compiler incluses), Vite 8 (Rolldown), Vitest 5. Stockage et crypto : libSQL
0.10, argon2 0.6, chacha20poly1305 0.11, hmac 0.13, sha2 0.11, keyring-core 1.

### Ce qui n'est pas encore à la dernière version

| Paquet | Retenu à | Raison | À revoir |
|---|---|---|---|
| TypeScript | 6.0.3 | typescript-eslint exige `typescript <6.1` (TS 7 n'a pas encore d'API JS) | quand typescript-eslint supporte TS 7 (attendu avec TS 7.1) |
| eslint-plugin-jsx-a11y | 6.10.2 | sa plage de pairs s'arrête à ESLint 9, mais il fonctionne sous ESLint 10 (règles vérifiées) ; seul avertissement de `pnpm peers check` | à la prochaine version (PR #1081 en attente) |
| tauri-plugin-updater | 2.14.0 | la 2.14.1 (zip 9) a moins de 7 jours | dès le 2026-11-04 |
| calamine (dev) | 0.37.0 | la 0.38 (corrige RUSTSEC-2026-0436) a moins de 7 jours | dès le 2026-11-09 |

### Exceptions d'audit restantes

| Alerte | Paquet (via) | Pourquoi ce n'est pas atteignable | Retrait |
|---|---|---|---|
| RUSTSEC-2026-0367/0368/0369/0429 | zip 4 (updater), 6 (spreadsheet-ods), 9 (calamine) | aucune archive non fiable n'est extraite : l'updater n'extrait (Windows) qu'après vérification de la signature minisign, spreadsheet-ods n'écrit que nos .ods, calamine ne sert qu'aux tests | updater 2.14.1 (zip 9) + nouvelle version de spreadsheet-ods |
| RUSTSEC-2026-0436 | calamine | dépendance de test, ne lit que des fichiers générés par les tests | calamine 0.38 |
| RUSTSEC-2026-0370/0371/0372 | zvariant, zbus_names (zbus 5, via tauri-plugin-opener) | crates non maintenues (fusionnées dans zbus 6), pas de faille | quand tauri-plugin-opener passe à zbus 6 |
| RUSTSEC-2026-0194/0195 | quick-xml (spreadsheet-ods, syntect) | DoS au parsing : on n'écrit que nos .ods et on ne charge que des thèmes embarqués | nouvelle version des parents |
| RUSTSEC-2025-0141, 2024-0320, 2024-0370 | bincode, yaml-rust, proc-macro-error | non maintenues ; données de confiance embarquées ou build uniquement | nouvelle version de syntect |
| GHSA-g3hv-75wg-m249 (modérée, npm) | minimatch 3/10 (eslint, jsx-a11y) | DoS sur des motifs glob contrôlés par l'attaquant : ici, seuls les motifs de notre configuration ESLint, outil de dev | quand ESLint passe à minimatch 11 |
