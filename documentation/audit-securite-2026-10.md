# Audit de sécurité n°2 — BasicPresence

| | |
|---|---|
| **Date** | 14 octobre 2026 |
| **Version auditée** | commit `5ec9706` (branche `main`, arbre propre) |
| **Contexte** | Reprise après ~2 mois et demi d'inactivité (dernier commit : 3 août 2026) |
| **Périmètre** | Suivi des constats de l'[audit du 3 juillet](audit-securite-architecture.md) ; code ajouté depuis `cde8ddb` (export ODS « description des transports », +1 659 lignes) ; dépendances npm et crates ; CI/CD ; état du dossier de données sur disque |
| **Méthode** | `pnpm audit`, `cargo audit` + requêtes OSV sur les 718 crates du `Cargo.lock`, traçage des chemins de dépendance (`pnpm why`, `cargo tree -i`), revue manuelle des nouveaux fichiers et des surfaces d'entrée (Markdown, import de bundle, export ODS), inspection des permissions des fichiers de données |

> **Suivi :** les constats encore ouverts ici (E1, M1, M2, M5, F6, I1, I4, I6) et de nouveaux défauts ont été traités le 15 octobre 2026 — voir [audit-securite-2026-10-remediation.md](audit-securite-2026-10-remediation.md).

---

## 1. Synthèse exécutive

**Le code n'a pas régressé, les dépendances oui.** Le cœur cryptographique et l'authentification n'ont pas changé depuis le premier audit et ses conclusions restent valables. Le nouveau code d'export est sain : pas d'injection de formule (cellules texte uniquement), erreurs correctement remontées en `INTERNAL`.

En revanche, l'inactivité a laissé les dépendances dériver : **38 vulnérabilités npm (3 critiques, 23 élevées)** et **une vingtaine d'advisories Rust**, dont certaines touchent des composants embarqués dans le binaire (éditeur de notes Milkdown/ProseMirror, sanitiseur Ammonia, TLS de l'updater). Conséquence directe : **la CI était rouge**, donc le filet de sécurité réactivé en juillet ne protégeait plus rien. La cause racine est l'absence de Dependabot/Renovate (reste du constat M4).

Aucune vulnérabilité **exploitable** n'a été identifiée en l'état : la CSP `script-src 'self'` bloque les vecteurs XSS côté WebView, la configuration Ammonia n'active pas les balises concernées, et les mises à jour sont signées (minisign) en plus du TLS.

**Décompte : 0 critique · 1 élevé · 3 moyens · 3 faibles · 4 informatifs** (nouveaux constats), plus 4 constats de juillet encore ouverts (E1, M1, M2, M5).

---

## 2. Constats

Sévérités : **Élevé** = à corriger en priorité · **Moyen** = à planifier · **Faible** = amélioration souhaitable · **Info** = à connaître/documenter.

### N1 — CI cassée par la dérive des dépendances (Élevé)

> ✅ **Corrigé le 14 octobre 2026** — dépendances mises à jour (voir N2–N5), `pnpm audit` et `cargo audit` repassent au vert ; Dependabot ajouté (`.github/dependabot.yml`).

- **Où** : `.github/workflows/ci.yml` — étapes `pnpm audit --audit-level=high` et `cargo audit`.
- **Constat** : les deux audits échouaient. Une CI durablement rouge n'est plus lue : une vraie régression passerait inaperçue.
- **Cause racine** : aucun outil de mise à jour automatique (Renovate/Dependabot), déjà relevé en juillet (M4).

### N2 — Dépendances frontend vulnérables embarquées dans l'application (Moyen)

> ✅ **Corrigé le 14 octobre 2026** — `@milkdown/crepe` 7.22, overrides `dompurify >= 3.4.16`, `katex >= 0.18.2` ; `prosemirror-view` résolu en 1.42.6.

- **Où** : `@milkdown/crepe` (éditeur de notes) → `prosemirror-view` < 1.42.3 (XSS au collage), `dompurify` ≤ 3.4.15 (3 advisories ; l'override de juillet `>=3.4.11` ne suffisait plus), `katex` < 0.18.2, `nanoid` 5.x.
- **Impact** : ces paquets sont dans le bundle distribué. Exploitation fortement entravée par la CSP (`script-src 'self'` : ni script inline ni gestionnaire d'événement) et par le fait que l'utilisateur ne colle que son propre contenu.

### N3 — Ammonia 4.1.2, sanitiseur des notes Markdown (Moyen)

> ✅ **Corrigé le 14 octobre 2026** — `ammonia = "4.1.4"` dans `Cargo.toml` (résolu en 4.2.3).

- **Où** : `infrastructure/markdown/comrak_renderer.rs:75` ; RUSTSEC-2026-0193 (mXSS MathML `annotation-xml`), RUSTSEC-2026-0213 / GHSA-m6mh-2hw2-555x (SVG `set`/`animate`).
- **Constat** : **non exploitable avec notre configuration** — whitelist par défaut + `input`, et `style` sur `pre`/`span` ; ni `math`, ni `svg`, ni `set`/`animate`. Mais c'est la brique de sécurité centrale du rendu HTML : elle doit être à jour.

### N4 — rustls 0.23.40 dans l'updater (Moyen)

> ✅ **Corrigé le 14 octobre 2026** — `cargo update` → rustls 0.23.46.

- **Où** : `tauri-plugin-updater` → `reqwest` → `rustls` ; RUSTSEC-2026-0285, -0346, -0348, -0349 (dont rétrogradation de l'échange de clés post-quantique).
- **Impact** : limité — le transcript TLS reste authentifié et chaque mise à jour est vérifiée par signature minisign avec clé publique épinglée.

### N5 — Vulnérabilités d'outillage (dev/build uniquement) (Faible)

> ✅ **Corrigé le 14 octobre 2026** — `vitest` 4.1 (tinypool corrigé), mises à jour dans les plages semver, override `postcss-selector-parser >= 7.1.6`. Un mock de test a été adapté à Vitest 4 (implémentation `function` au lieu de fonction fléchée, requise pour `new`).

- `tinypool` (critique, via vitest), `seroval` (critique, via router-devtools), `@vitest/mocker`, `postcss`, `brace-expansion`, `js-yaml`, `browserslist`, `source-map-js`, `nanoid` 3.x…
- **Impact** : non distribués, mais exécutés sur le poste de développement et en CI.

### N6 — Advisories Rust résiduelles sans correctif disponible (Info)

> ✅ **Traité le 14 octobre 2026** — ignorées explicitement, avec justification par entrée, dans `src-tauri/.cargo/audit.toml`.

- **Pile client distant de libsql 0.9** (`hyper` 0.14, `h2` 0.3, `rustls` 0.22, `rustls-webpki` 0.102) : jamais utilisée, la v1 n'ouvre la base qu'en `Builder::new_local`. À réévaluer lors de la synchro Turso (Phase 2) ou de la migration vers libsql 0.10.
- **quick-xml 0.38/0.39** (DoS au parsing) : `spreadsheet-ods` ne fait qu'*écrire* nos propres `.ods`, `plist` (via syntect) ne lit que des thèmes embarqués à la compilation, `calamine` est une dépendance de test.
- **Crates non maintenues** (bincode, yaml-rust, proc-macro-error, glib 0.18) : transitives via tauri/syntect, données de confiance ou build uniquement.

### N7 — Dossier de données lisible par les autres comptes locaux (Faible)

> ✅ **Corrigé le 14 octobre 2026** — `infrastructure/persistence/data_dir.rs` : dossier en `0700`, fichiers existants en `0600` (Unix), appliqué à chaque démarrage. Tests unitaires associés.

- **Où** : `lib.rs` (`create_dir_all` du dossier `app_data_dir`).
- **Constat** : `~/.local/share/com.godef.basic-presence/` était en `0755` et `keystore.db`, `vault.db`, `vault.db.hmac` en `0644`. Le contenu est chiffré/scellé, mais tout compte local pouvait copier le chiffré pour une attaque hors ligne.
- **Note** : restreindre le dossier suffit à protéger aussi les fichiers créés plus tard (WAL, `-shm`, sidecars). Sous Windows/macOS, l'emplacement par utilisateur est déjà protégé par ACL.

### N8 — Lecture d'un bundle d'import sans limite de taille (Faible)

> ✅ **Corrigé le 14 octobre 2026** — plafond de 50 Mio dans `json_bundle_codec.rs` (lecture bornée par `take`, pas de course entre contrôle et lecture), message `Validation` générique. Test unitaire sur fichier creux.

- **Constat** : `read_to_string` chargeait intégralement le fichier choisi ; un fichier de plusieurs Go provoquait un épuisement mémoire (plantage de l'app).

### N9 — Chaîne CI : actions non épinglées, permissions par défaut (Faible)

> ✅ **Corrigé le 14 octobre 2026** — toutes les actions épinglées par SHA de commit (tag en commentaire) dans `ci.yml` et `release.yml` ; `permissions: contents: read` dans `ci.yml` ; Dependabot surveille aussi `github-actions`.

- **Constat** : `tauri-action@v0`, `rust-cache@v2`, `rust-toolchain@stable`… sont des refs mutables. Le workflow de release, qui reçoit la clé de signature (E1), exécutait donc du code tiers modifiable par ses mainteneurs.

### N10 — Exports écrits en clair (Info)

- Les exports ODS et JSON sortent volontairement du coffre chiffré. Vérifié : aucune injection de formule possible (cellules texte `spreadsheet-ods`, aucune formule générée).
- **Recommandation** : afficher dans les dialogues d'export un avertissement « ce fichier n'est pas chiffré ».

### N11 — Messages d'erreur de l'export (Info)

- `DomainError::Export(format!("write file: {e}"))` contient le détail OS, mais `presentation/commands/error.rs` le fait collapser en `INTERNAL` avant la WebView. Aucune fuite.

---

## 3. Suivi des constats de l'audit du 3 juillet

| # | Sévérité | Statut au 14/10/2026 |
|---|---|---|
| E1 | Élevé | ⏳ **Toujours ouvert** — clé de signature dans les secrets GitHub. À faire : GitHub Environment `release` avec *required reviewer* contenant `TAURI_SIGNING_*` (+ `environment: release` dans `release.yml`), MFA matériel. N9 réduit la surface mais ne remplace pas cette mesure. |
| E2 | Élevé | ✅ Corrigé (juillet). |
| M1 | Moyen | ⏳ Toujours ouvert — chemins d'export/import libres depuis la WebView. |
| M2 | Moyen | ⏳ Toujours ouvert — `require_session` ne vérifie pas le token appelant. |
| M3 | Moyen | ✅ Corrigé (juillet), puis de nouveau dérivé → retraité en N2. |
| M4 | Moyen | ✅ Clos — Dependabot ajouté (N1). |
| M5 | Moyen | ⏳ Toujours ouvert — pas de `change_password`. |
| F1–F5 | Faible | ✅ Corrigés (juillet). |
| F6, I1, I4, I6 | Info | ⏳ Notes de design toujours à rédiger. |

---

## 4. Plan d'action

| Priorité | Action | Constats | Statut |
|---|---|---|---|
| 1 | Mettre à jour les dépendances npm et crates ; ignorer explicitement les résiduelles justifiées | N1–N6 | ✅ Fait (14/10/2026) |
| 2 | Ajouter Dependabot (npm, cargo, actions) | N1, M4 | ✅ Fait (14/10/2026) |
| 3 | Protéger la clé de signature (Environment + reviewers, MFA matériel) | E1 | ⏳ Réglages GitHub à faire |
| 4 | Épingler les actions par SHA, permissions CI minimales | N9 | ✅ Fait (14/10/2026) |
| 5 | Dossier de données en `0700` / fichiers en `0600` | N7 | ✅ Fait (14/10/2026) |
| 6 | Plafond de taille à l'import de bundle | N8 | ✅ Fait (14/10/2026) |
| 7 | Confiner les chemins d'export/import (dialogue côté Rust) | M1 | ⏳ À faire |
| 8 | Implémenter `change_password` ; avertir « aucune récupération possible » | M5 | ⏳ À faire |
| 9 | Trancher M2 ; avertissement « fichier non chiffré » à l'export ; notes de design | M2, N10, F6, I1, I4, I6 | ⏳ À faire |
| 10 | Migrer vers libsql 0.10 (retire la pile rustls 0.22 / h2 0.3) | N6 | ⏳ À évaluer (format de chiffrement à valider) |

---

*Audit réalisé le 14 octobre 2026 sur le commit `5ec9706`. Leçon principale : pour une application « niveau banque », une période d'inactivité suffit à accumuler des vulnérabilités sans qu'aucune ligne de code ne change — la mise à jour automatisée des dépendances fait partie du modèle de sécurité.*
