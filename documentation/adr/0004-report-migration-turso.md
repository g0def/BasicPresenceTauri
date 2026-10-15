# ADR 0004 — Rester sur libsql 0.9 ; migration vers `turso`/AEGIS reportée

- **Statut** : acceptée, octobre 2026 — **à réévaluer** à chaque version du crate `turso`

## Contexte

Le coffre est chiffré par libsql 0.9 en AES-256-CBC, sans authentification ;
l'intégrité est compensée par un HMAC (ADR 0002). En 2026, Turso recommande son
nouveau crate `turso` pour le chiffrement local, avec **AEGIS-256** (chiffrement
authentifié, par page). Mais cette fonction y est encore marquée
**expérimentale**, et un fichier chiffré par l'un n'est pas lisible par l'autre
(migration de données obligatoire).

## Décision

- Rester sur libsql 0.9 pour la v1, durci : HMAC dont l'état est scellé dans le
  keystore, `default-features = false` (aucune pile réseau embarquée — c'était la
  source de la plupart des alertes `cargo audit` ignorées).
- Ne pas migrer tant que le chiffrement de `turso` est expérimental.

## Conditions de réévaluation

Migrer quand le chiffrement de `turso` est déclaré stable. La migration
demandera : lecture du coffre libsql (DEK actuel) → écriture dans une base
`turso` AEGIS-256 (même DEK ou nouveau DEK ré-enveloppé) → bascule récupérable
après crash (même schéma que la migration du keystore) → suppression du HMAC,
devenu inutile. La synchro cloud (Phase 2) devra être arbitrée en même temps
(voir [turso.md](../turso.md)).
