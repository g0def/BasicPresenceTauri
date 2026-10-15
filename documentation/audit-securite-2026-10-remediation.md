# Audit n°3 — Remédiation architecture & sécurité

| | |
|---|---|
| **Date** | 15 octobre 2026 |
| **Base** | commit `dcf97a2` (après l'[audit n°2](audit-securite-2026-10.md)) |
| **Branche** | `refactor/security-architecture-remediation` |
| **Méthode** | Revue complète (backend, frontend, CI) + recherches sur les bonnes pratiques 2026 (Tauri v2, OWASP/RFC 9106, pnpm 11, GitHub Actions), puis correction lot par lot, tests entre chaque lot |

## 1. Synthèse

Les deux audits précédents laissaient ouverts **E1, M1, M2, M5, F6, I1, I4, I6**. La revue a en outre trouvé des défauts **plus graves**, absents des audits précédents. Les trois principaux :

1. **L'évidence d'altération du coffre se contournait** en supprimant `vault.db.hmac` ou en créant `vault.db.dirty` (acceptés même en `HardFail`).
2. **Une seule connexion libSQL était partagée** par toutes les commandes asynchrones, avec des `BEGIN`/`ROLLBACK` manuels : deux commandes concurrentes pouvaient se retrouver dans la même transaction, et un `ROLLBACK` annuler les écritures d'une autre.
3. **Un crash pendant une migration** du coffre produisait un faux `VAULT_TAMPERED` bloquant (migrations non transactionnelles, marqueur posé trop tard).

Tous les constats ouverts sont **corrigés ou tranchés par une décision documentée**. La seule action restante est un **réglage GitHub** (E1) que seul le propriétaire du dépôt peut faire.

**Vérification finale** : Rust 132 tests, `clippy -D warnings`, `cargo deny` OK ; frontend 126 tests, `tsc`, ESLint (typé + frontières), `pnpm audit` (0 vulnérabilité), build de production OK ; démarrage réel de l'application en dev avec IPC à travers le pattern d'isolation vérifié.

## 2. Constats et corrections

### Sécurité critique (lot 0)

| Constat | Correction |
|---|---|
| Intégrité du coffre contournable par les fichiers annexes | Référence et indicateur « session ouverte » stockés **dans le keystore scellé** ; coffre supprimé = refusé ; MAC v2 avec longueurs (ferme **I1**) ; fichiers hérités migrés une fois puis supprimés. [ADR 0002](adr/0002-integrite-du-coffre.md) |
| Transactions sur connexion partagée | `LibsqlVaultManager::write` : verrou d'écriture, connexion dédiée, `BEGIN IMMEDIATE`, transaction ambiante (task-local) ; `busy_timeout`. Garde CI contre tout `BEGIN` manuel. |
| Migrations non atomiques / faux `VAULT_TAMPERED` | Chaque migration et sa ligne `_migrations` dans une transaction ; « session ouverte » posée **avant** ouverture et migrations, échec = refus d'ouvrir. |
| Erreur de démarrage = panic sans message | Boîte de dialogue native puis sortie propre ; clé de device jamais régénérée si un keystore scellé existe. |

### Sécurité élevée (lot 1)

| Constat | Correction |
|---|---|
| **M1** chemins de fichiers libres depuis la WebView | Dialogues ouverts **côté Rust** ; plus aucun chemin dans l'IPC ; handle opaque à usage unique pour l'import ; permissions `dialog:*` retirées ; écritures atomiques. |
| **M5** pas de changement de mot de passe | `change_password` (ré-enveloppement, coffre non re-chiffré) + écran Réglages → Sécurité + avertissement « aucune récupération » à l'inscription. |
| **M2** session non liée au token | **Décision** : session globale assumée ([ADR 0001](adr/0001-session-globale.md)) ; `logout` exige désormais le token du détenteur. |
| KEK dérivée avec les paramètres du hash d'auth | Profil KDF dédié plus fort (64 MiB, t=3), stocké par compte ; mise à niveau transparente au login. |
| Pas d'AAD dans l'enveloppe des clés | Format v2 : AAD `rôle ‖ compte` ; v1 relu puis ré-enveloppé au login. |
| Verrouillage : TOCTOU, pas de progression | Incrément atomique, tentatives sérialisées, verrouillage doublé à chaque échec (plafond 24 h), entrées surdimensionnées refusées sans calcul, paramètres PHC bornés. |
| Inscriptions concurrentes → deux comptes | Insertion conditionnelle atomique. |
| Inactivité uniquement côté UI | `last_seen` dans la session, 6 min côté Rust ; ping d'activité de l'UI ; retour au premier plan jugé sur le temps écoulé. |
| Entrées non bornées (note, segments, distances…) | Règles centralisées dans `domain/rules.rs`, appliquées aussi à l'import. |
| Message serde exposant le contenu d'un fichier | Seules ligne/colonne remontent. |
| Argon2 / E/S bloquants sur l'exécuteur async | `spawn_blocking`. |
| Coffre non verrouillé à la sortie / au redémarrage | `RunEvent::Exit` + logout avant redémarrage sur mise à jour. |

### Chaîne d'approvisionnement & configuration (lot 2)

| Constat | Correction |
|---|---|
| libsql embarquait toute sa pile réseau | `default-features = false` : 718 → 647 crates, 8 exceptions `cargo audit` supprimées. |
| Pas de profil release | `lto`, `codegen-units = 1`, `strip`, `overflow-checks`. |
| `cargo audit` installé sans version | `cargo-deny` (advisories, licences, sources) via action épinglée. |
| Édition 2021 | Édition 2024. |
| pnpm sans garde-fous | `packageManager`, `minimumReleaseAge` 7 jours (exceptions nominatives pour les correctifs de l'audit n°2), `strictDepBuilds`, dépendances et overrides en version exacte, `cooldown` Dependabot. |
| **E1** clé de signature + release | Job dans un environnement `release` protégé, permissions par job, **pas de cache** dans le job qui signe, revérification complète du commit tagué, attestation de provenance, SBOM CycloneDX, `SHA256SUMS`, garde contre la config de test (**I3**), `zizmor` en CI. **Réglages GitHub à faire** : voir [release.md](release.md#protecting-the-signing-key-audit-finding-e1). |
| PKGBUILD sans vérification | sha256 épinglé ; licence corrigée (n'était pas MIT). |
| Pattern d'isolation absent | Activé : liste blanche des commandes, synchronisée avec `lib.rs` par un test. |
| CSP / config Tauri | `form-action 'none'`, `removeUnusedCommands`, label de fenêtre explicite. `freezePrototype` **non activé** : il casse echarts et Milkdown (`tslib.__extends`), vérifié. `style-src 'unsafe-inline'` conservé (**F6**) : requis par le surligneur syntaxique et Radix ; `script-src 'self'` reste strict. |

### Architecture (lots 3 et 4)

| Constat | Correction |
|---|---|
| Validateurs dans des use cases importés par d'autres use cases ; logique dupliquée | `domain/rules.rs` + `application/services` (constructeur de trajets, gestion des clés). `AuthPolicy` déplacée dans le domaine (seule violation de la règle de dépendance). |
| **I6** import non atomique ; écritures multi-dépôts non atomiques | Port `UnitOfWork` : import, jour + horaire, entrées + horaire, profil + profil actif. |
| N+1 à l'export ; fichiers trop gros | Requêtes par profil ; import et codec découpés. |
| **I4** aucune journalisation | Journal de sécurité minimal dans le keystore, montré après connexion. |
| Front : état serveur recodé ×15, `SESSION_EXPIRED` ×7 (oublié ×4) | TanStack Query, gestion centrale, cache vidé au logout. [ADR 0003](adr/0003-tanstack-query.md) |
| Dépôts instanciés dans la présentation | Conteneur d'injection, composition root unique. |
| IPC non typé | Registre de commandes typé par feature, exhaustivité vérifiée à la compilation. |
| Cycle `presence` ↔ `work-hours`, imports croisés partout | Couches `entities/`, `pages/`, `app/` ; frontières vérifiées par ESLint (seule dérogation : `presence` → `commute`). |
| Lint minimal | ESLint typé, `jsx-a11y`, `dangerouslySetInnerHTML` restreint aux deux vues assainies. |
| Updater | Vérification au démarrage dans un effet, contexte mémoïsé, progression correcte. |

## 3. Reste à faire

| # | Action | Responsable |
|---|---|---|
| E1 | Créer l'environnement GitHub `release` (reviewer requis, tags `v*`), y déplacer `TAURI_SIGNING_*`, supprimer les secrets de dépôt, ruleset de tags, MFA matérielle | Propriétaire du dépôt (réglages GitHub) |
| — | Découper `presence-day-dialog.tsx` et `presence-calendar.tsx` (~350 lignes) | Prochaine itération, avec vérification visuelle |
| — | Migration vers `turso`/AEGIS-256 quand son chiffrement sera stable | [ADR 0004](adr/0004-report-migration-turso.md) |
| — | Tester une mise à jour réelle (`tauri.localtest.conf.json`) : logout avant redémarrage, attestation | Avant la prochaine release |

---

*Leçon : les deux défauts les plus graves (intégrité contournable, transactions partagées) se trouvaient dans du code déjà audité et testé. Leurs tests vérifiaient le cas nominal, pas le contournement ni la concurrence. Les nouveaux tests d'intégration couvrent explicitement ces deux angles.*
