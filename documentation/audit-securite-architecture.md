# Audit de sécurité & d'architecture — BasicPresence

| | |
|---|---|
| **Date** | 3 juillet 2026 |
| **Version auditée** | commit `cde8ddb` (branche `main`, arbre propre) |
| **Périmètre** | Backend Rust (`src-tauri/`), frontend React (`src/`), configuration Tauri, capabilities, CI/CD, dépendances, documentation |
| **Méthode** | Revue manuelle du cœur cryptographique et des flux d'authentification + trois passes d'analyse parallèles (frontend, architecture backend, configuration/supply chain), affirmations croisées et re-vérifiées sur le code |

> Voir aussi [auth.md](auth.md) (modèle de sécurité documenté) et [turso.md](turso.md) (persistance). Cet audit vérifie notamment que ces documents correspondent au code réel — c'est le cas.

---

## 1. Synthèse exécutive

**Verdict global : très bonne posture de sécurité et architecture exemplaire pour une application de cette taille.** Le cœur cryptographique (envelope encryption, Argon2id, XChaCha20-Poly1305, HMAC d'intégrité, keystore scellé par le trousseau OS) est implémenté avec soin, testé, et fidèle à sa documentation. L'architecture hexagonale est strictement respectée. **Aucune vulnérabilité critique exploitable n'a été identifiée.**

Les constats se concentrent sur trois zones :

1. **Chaîne d'approvisionnement** — la clé privée de signature des mises à jour vit dans les secrets GitHub : la compromission du compte GitHub suffirait à distribuer une mise à jour malveillante signée valide (E1).
2. **Robustesse** — une fenêtre de crash dans la migration du keystore peut détruire définitivement l'accès au coffre (E2).
3. **Défense en profondeur** — chemins de fichiers d'export/import non confinés (M1), session non liée au token de l'appelant (M2), absence de changement de mot de passe (M5), contrôles CI désactivés (M4).

| Domaine | Évaluation |
|---|---|
| Cryptographie & authentification | ★★★★★ (excellent) |
| Architecture backend (hexagonale) | ★★★★★ (excellent) |
| Sécurité frontend (XSS, secrets, IPC) | ★★★★☆ (très bon) |
| Configuration Tauri (CSP, capabilities) | ★★★★☆ (très bon) |
| Chaîne d'approvisionnement & CI | ★★★☆☆ (à durcir) |
| Robustesse / résilience aux crashs | ★★★★☆ (très bon, un point dur) |

**Décompte des constats : 0 critique · 2 élevés · 5 moyens · 6 faibles · 6 informatifs.**

---

## 2. Architecture — évaluation

### 2.1 Respect des couches (hexagonale / clean architecture)

Le backend suit `domain/ → application/ → infrastructure/ + presentation/` avec une discipline rare :

- **Aucune dépendance** de `domain/` vers `infrastructure/` ou `presentation/` (vérifié par analyse des imports).
- Les use cases (`application/use_cases/`) ne dépendent que de traits du domaine ; la seule exception est l'import de `infrastructure::config::AuthPolicy` dans `login.rs:16` — un objet de configuration pur, acceptable.
- Les commandes Tauri (`presentation/commands/`) sont des proxys minces : garde de session, délégation au use case, mapping d'erreur. Zéro logique métier.
- Composition root unique (`lib.rs:176`, `build_state`) : toute l'injection de dépendances en une passe, derrière des `Arc<dyn Trait>`.
- Le frontend reproduit la même rigueur : `src/core/ipc.ts` est le **seul** module autorisé à importer `@tauri-apps/api`, `src/core/external-link.ts` le seul à importer le plugin opener. Aucun contournement détecté.

### 2.2 Accès aux données

- **100 % du SQL est paramétré** (`params![...]`). Les `format!` ne servent qu'à interpoler des listes de colonnes constantes (`SELECT_COLUMNS`), jamais des données.
- Transactions `BEGIN/COMMIT/ROLLBACK` correctes pour toutes les écritures multi-tables (`presence_repository.rs:103`, `commute_repository.rs:81`, `work_entry_repository.rs:67`).
- `PRAGMA foreign_keys = ON` appliqué sur chaque connexion (`db.rs:36`) — indispensable pour les `ON DELETE CASCADE`, et le commentaire explique pourquoi.
- Migrations versionnées, embarquées à la compilation (`include_str!`), idempotentes (table `_migrations`), appliquées par `execute_batch` (atomique par migration).

### 2.3 Concurrence et état

- État partagé minimal : `Mutex<Option<OpenVault>>` (vault) et `Mutex<HashMap>` (sessions), verrous à portée courte, jamais tenus à travers un `await`.
- Récupération des locks empoisonnés via `unwrap_or_else(|p| p.into_inner())` — pattern volontaire, cohérent partout.
- Aucun `unwrap()`/`expect()`/`panic!` en code de production (hors le `.run().expect()` final standard de Tauri, `lib.rs:171`).

### 2.4 Tests

- 2 019 lignes de tests d'intégration (`integration_tests.rs`) couvrant les chemins critiques de bout en bout : flux auth complet avec chiffrement réel, migration keystore clair → scellé, **détection d'altération du vault sous HardFail**, garde de session sur l'accès aux données.
- Tests unitaires sur tous les composants crypto, le store de sessions, les validateurs, le codec de bundle, le rendu Markdown (y compris des tests anti-XSS explicites : `<script>`, `javascript:`).
- Non testé unitairement (couvert seulement en E2E) : les repositories libSQL — acceptable.
- Zéro `TODO`/`FIXME` dans le code de production.

### 2.5 Points d'architecture à connaître (non bloquants)

- **Import de bundle atomique par jour, pas globalement** (`import_profile_bundle.rs`) : un échec au jour N laisse les jours 1..N-1 écrits. La validation complète du fichier se fait *avant* toute écriture, ce qui limite fortement le risque en pratique, et le résumé retourné compte ce qui a réussi. Acceptable, mais à documenter.
- **Aucune journalisation** nulle part (choix assumé de confidentialité). Conséquence : pas de trace exploitable en cas d'incident ou de tentative d'intrusion (voir I4).

---

## 3. Sécurité — évaluation du modèle

### 3.1 Cryptographie (conforme à l'état de l'art)

Le schéma documenté dans [auth.md](auth.md) est fidèle au code :

- **Argon2id** profil OWASP 46 MiB (`m=47104, t=1, p=1`, `config.rs:53`), deux usages à sels distincts : hash d'authentification (PHC) et dérivation du KEK.
- **Envelope encryption** : DEK aléatoire 32 octets, enveloppé par XChaCha20-Poly1305 (AEAD, nonce 192 bits aléatoire par wrap, `key_service.rs:47`). Longueurs de KEK/nonce/DEK vérifiées avant usage.
- **Keystore scellé** au repos par une clé de device de 32 octets vivant dans le trousseau OS (`device_key.rs`) — un vol du seul fichier `keystore.db` ne permet plus le brute-force hors ligne. L'indisponibilité du Secret Service est une erreur de démarrage, pas un repli silencieux en clair.
- **Évidence d'altération du vault** : libSQL 0.9 ne chiffre qu'en AES-256-CBC non authentifié ; compensé par un HMAC-SHA256 du fichier (+ WAL) en sidecar, comparé en temps constant (`subtle::ConstantTimeEq`), avec marqueur « dirty » pour distinguer crash et falsification, politique `HardFail` par défaut. Design réfléchi et testé.
- **Sessions** : tokens 256 bits (`getrandom`), stockés uniquement sous forme de digest SHA-256 avec le token expurgé (`in_memory_session_store.rs:32-41`) — un dump mémoire ne révèle pas les tokens ; pas de signal de timing sur la comparaison.
- **Hygiène mémoire** : `Zeroizing` systématique sur mot de passe, DEK, KEK, clé MAC, y compris dès la frontière IPC (`auth.rs:26,41`). Limite résiduelle des buffers de désérialisation Tauri connue et documentée (auth.md).

### 3.2 Authentification et session

- Anti-brute-force : 5 échecs → verrouillage 5 min, persistant dans le keystore (lui-même scellé).
- Égalisation de timing contre l'énumération d'utilisateur : hash « à blanc » quand le username est inconnu (`login.rs:64-67`).
- Expiration **absolue** de 15 min appliquée côté backend : chaque commande touchant le vault appelle `require_session` avant d'agir ; à l'expiration, sessions purgées **et vault verrouillé** — le frontend ne peut pas prolonger l'accès. Complété côté UI par un timeout d'inactivité de 5 min.
- Redémarrage de l'app = re-login obligatoire (store de sessions en mémoire).

### 3.3 Surfaces d'entrée non fiables

- **Notes Markdown** : rendues côté Rust par comrak avec `render.unsafe_ = false` (tout HTML autorisé est échappé), puis assainies par Ammonia (whitelist) ; les seuls ajouts à la whitelist (`input` de tasklist, `style` sur `pre`/`span`) ne peuvent provenir que du highlighter interne, jamais d'un contenu autoré. Testé contre `<script>` et `javascript:`.
- **Bundles de profil importés** (fichiers JSON externes) : format tagué et versionné, désérialisation serde stricte, **validation complète de tout le contenu avant la moindre écriture**, réutilisant les mêmes validateurs que les use cases nominaux (`import_profile_bundle.rs:11-17`). Très bon.
- **Frontend** : token de session en `useRef` mémoire uniquement, jamais persisté ; mot de passe en state local éphémère ; `localStorage` limité à cinq préférences non sensibles (thème, police, langue…) ; aucun `eval`/`new Function`/`innerHTML` ; les deux `dangerouslySetInnerHTML` n'affichent que du HTML assaini par le backend ; zéro `console.log`.

### 3.4 Configuration Tauri

- **CSP stricte** (`tauri.conf.json`) : `default-src 'self'`, `script-src 'self'`, `object-src 'none'`, `frame-ancestors 'none'`, `base-uri 'self'`. Le seul assouplissement est `style-src 'unsafe-inline'` (F6).
- **Capabilities minimales et cohérentes** : updater, restart (post-update), dialogues open/save (export/import), opener. Pas de permission fs, shell, http, clipboard.
- **Updater signé** : clé publique minisign épinglée dans la conf, endpoint HTTPS GitHub Releases. La conf `tauri.localtest.conf.json` (HTTP + `dangerousInsecureTransportProtocol`) n'est référencée par aucun workflow CI (I3).

---

## 4. Constats détaillés

Sévérités : **Élevé** = à corriger en priorité · **Moyen** = à planifier · **Faible** = amélioration souhaitable · **Info** = à connaître/documenter.

### E1 — Clé privée de signature des mises à jour dans les secrets GitHub

- **Où** : `.github/workflows/release.yml:79-80` (`TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`).
- **Constat** : la clé minisign qui signe les mises à jour auto-installées par tous les clients réside (avec sa passphrase) dans les secrets du dépôt. [release.md](release.md) recommande pourtant de garder la clé locale.
- **Scénario** : compromission du compte GitHub (phishing, vol de token, MFA bypass) → l'attaquant déclenche le workflow, obtient un binaire malveillant **signé valide**, publié sur l'endpoint que tous les clients interrogent → compromission de tout le parc.
- **Impact** : c'est le maillon le plus faible du système — il court-circuite toute la crypto locale.
- **Recommandations** (par ordre croissant d'effort) :
  1. MFA matériel (passkey/FIDO2) obligatoire sur le compte GitHub + vérifier qu'aucun token PAT à large portée ne traîne.
  2. Placer les deux secrets dans un **GitHub Environment** avec *required reviewers* et restriction aux tags protégés — un attaquant ne peut plus signer sans approbation manuelle.
  3. Idéal « niveau banque » : signer localement sur une machine de confiance (la clé ne quitte jamais le poste), la CI ne faisant que construire et publier le draft non signé.

### E2 — Fenêtre de perte définitive du coffre dans la migration du keystore

- **Où** : `keystore_bootstrap.rs:196-199` (`migrate_plaintext_to_encrypted`).
- **Constat** : la bascule finale fait `remove_db_files(path)` **puis** `std::fs::rename(tmp, path)`. Un crash (panique, coupure de courant, OOM-kill) entre les deux laisse : plus de `keystore.db`, et les données migrées orphelines dans `keystore.db.new`.
- **Scénario** : au démarrage suivant, `open_or_migrate_keystore` prend la branche « fresh install » (`!path.exists()`, ligne 66) et crée un keystore **vide**. Le `wrapped_dek` du compte est perdu → le `vault.db` chiffré par le DEK devient **définitivement indéchiffrable**, même avec le bon mot de passe.
- **Impact** : perte totale des données utilisateur. Probabilité faible (fenêtre de quelques ms, migration one-shot des installations legacy), mais conséquence maximale et silencieuse.
- **Recommandation** : rendre la bascule récupérable — par exemple : (1) `rename(path, path.old)`, (2) `rename(tmp, path)`, (3) supprimer `path.old` ; et au démarrage, si `path` est absent mais que `path.old` ou `path.new` existe, récupérer au lieu de créer un keystore neuf. Supprimer les `-wal`/`-shm` reste nécessaire mais peut se faire avant l'étape (1).

### M1 — Chemins de fichiers d'export/import fournis librement par le frontend

- **Où** : `presentation/commands/transfer.rs:14-55`, `presentation/commands/export.rs:13-25`.
- **Constat** : `export_profile_data`, `export_profile_bundle`, `inspect_profile_bundle`, `import_profile_bundle` acceptent un `path: String` arbitraire venant de la WebView. Le backend lit/écrit ce chemin sans confinement. Le dialogue natif choisit le chemin côté JS, mais rien ne force le backend à ne recevoir que des chemins issus du dialogue.
- **Scénario** : si la WebView était compromise (XSS résiduel, dépendance frontend malveillante), ces commandes deviennent une primitive d'**écriture de fichier arbitraire** (export ODS/JSON vers n'importe quel chemin accessible, ex. écraser `~/.bashrc`) et de sonde de lecture (les messages d'erreur d'`inspect` révèlent l'existence/lisibilité de fichiers).
- **Impact** : élévation significative des conséquences d'une compromission WebView, aujourd'hui très improbable (CSP stricte, pas de contenu distant) — d'où « moyen » et non « élevé ».
- **Recommandations** : ouvrir le dialogue natif **côté Rust** (le plugin dialog a une API Rust) pour que le chemin ne transite jamais par le frontend ; à défaut, valider l'extension attendue (`.ods` / `.json`) et refuser d'écraser des fichiers hors des répertoires utilisateur standards.

### M2 — La garde de session ne vérifie pas le token de l'appelant

- **Où** : `application/use_cases/require_session.rs:32-41`.
- **Constat** : `require_session` passe si **une** session valide existe, sans exiger ni vérifier le token de l'appelant. Les commandes vault (`set_presence`, `export…`, etc.) ne prennent d'ailleurs aucun token. Le token retourné au login ne sert réellement qu'à `check_session`/`logout`.
- **Scénario** : tout code capable d'appeler l'IPC (la WebView, donc) accède au vault pendant les 15 minutes, token connu ou non. Dans le modèle Tauri actuel (une seule WebView locale = un seul principal), l'impact concret est quasi nul — c'est un constat de **défense en profondeur** et de cohérence de design.
- **Recommandation** : soit assumer et documenter (« la session est un état global temporel, pas une capacité »), soit passer le token à chaque commande gated et le vérifier — indispensable de toute façon si un jour plusieurs fenêtres/contextes coexistent.

### M3 — Dépendance vulnérable : DOMPurify ≤ 3.4.10 (modéré)

- **Où** : `pnpm audit` → `.>@milkdown/crepe>dompurify` (GHSA-cmwh-pvxp-8882, pollution `ALLOWED_ATTR` via `setConfig()`).
- **Impact** : réel faible — l'app n'appelle pas `setConfig`, l'éditeur Milkdown ne traite que le texte de l'utilisateur lui-même, et la sanitisation de sécurité est de toute façon côté Rust (Ammonia). Mais une dépendance de sanitisation vulnérable dans une app « niveau banque » doit être purgée.
- **Recommandation** : mettre à jour (`pnpm update` / override pnpm vers dompurify ≥ 3.4.11), et corriger au passage esbuild (F5).

### M4 — Contrôles CI désactivés (frontend + audit de dépendances)

- **Où** : `.github/workflows/ci.yml:14-38` (job frontend lint/typecheck/test/audit entièrement commenté) et `:79-89` (`cargo audit` commenté).
- **Constat** : seule la partie Rust (fmt, clippy, test) tourne en CI. Aucune détection automatisée de vulnérabilités de dépendances (ni npm ni crates), pas de vérification TypeScript/ESLint sur PR.
- **Recommandation** : réactiver le job frontend, ajouter `cargo audit` (ou `cargo deny`) et `pnpm audit --audit-level=moderate` ; envisager Dependabot/Renovate (aucun des deux n'est configuré).

### M5 — Pas de changement de mot de passe (ni procédure de récupération)

- **Où** : aucune commande/use case `change_password` (vérifié sur tout le code).
- **Constat** : l'envelope encryption a précisément été choisie pour permettre un changement de mot de passe instantané (re-wrap du DEK et de la clé MAC avec un nouveau KEK, sans re-chiffrer le vault) — mais la fonctionnalité n'existe pas. Par ailleurs, la perte du mot de passe = perte définitive des données (pas de mécanisme de secours), ce qui est un choix défendable mais nulle part affiché à l'utilisateur.
- **Recommandation** : implémenter `change_password` (vérifier l'ancien → dériver nouveau KEK → re-wrap DEK + MAC key → mettre à jour le PHC ; toutes les briques existent déjà dans `KeyService`). Documenter explicitement, à l'inscription, qu'aucune récupération n'est possible.

### F1 — Fermeture du vault incohérente quand plusieurs sessions coexistent

- **Où** : `check_session.rs:37-44`, `logout.rs:19-21`, `login.rs:139`.
- **Constat** : `login` insère une nouvelle session sans purger les précédentes ; à l'inverse, `check_session` (token expiré) et `logout` ferment le vault **même si une autre session encore valide existe**. Une commande gated passerait alors `require_session` mais trouverait le vault fermé (erreur `INTERNAL`).
- **Impact** : incohérence d'état sans enjeu de sécurité (elle ferme trop, pas trop peu), quasi inatteignable en mono-utilisateur.
- **Recommandation** : purger les sessions existantes au login (mono-propriétaire), ce qui rend l'invariant « ≥1 session valide ⇔ vault ouvert » exact.

### F2 — Permission `opener:allow-open-url` non restreinte

- **Où** : `src-tauri/capabilities/default.json:12` ; usage réel dans `src/core/external-link.ts` (sources de la page méthodologie).
- **Constat** : la permission autorise l'ouverture de n'importe quelle URL/chemin par l'OS. Toutes les URL légitimes sont pourtant codées en dur (`sources.ts` : ADEME, DEFRA…).
- **Recommandation** : restreindre la capability à un allowlist (`https://*`, voire les domaines précis) via la configuration fine du plugin opener.

### F3 — Détails d'E/S dans les erreurs de lecture de bundle

- **Où** : `json_bundle_codec.rs:52-57` (`Validation(format!("cannot read file: {e}"))`).
- **Constat** : contrairement au reste du code (qui collapse en `INTERNAL`), l'erreur `Validation` remonte telle quelle à l'UI avec le détail de l'erreur système. Fuite bénigne (app locale), mais incohérente avec la politique d'erreurs.
- **Recommandation** : message générique « fichier illisible » + code dédié.

### F4 — Politique de mot de passe : message inexact, règle minimale

- **Où** : `register_account.rs:100-108`.
- **Constat** : la règle est 8–1024 caractères mais le message d'erreur ne mentionne que le minimum ; et la politique se limite à la longueur (conforme NIST 800-63B, mais 8 reste court face à un vol de keystore *avec* extraction de la clé de device).
- **Recommandation** : corriger le message ; envisager un indicateur de force (zxcvbn-like) côté UI plutôt que des règles de composition.

### F5 — esbuild < 0.28.1 (low, dev uniquement)

- **Où** : `pnpm audit` → `.>vite>esbuild` (GHSA-g7r4-m6w7-qqqr, lecture de fichier via le dev server sous Windows).
- **Impact** : serveur de développement uniquement, pas les binaires distribués.
- **Recommandation** : mise à jour avec M3.

### F6 — `style-src 'unsafe-inline'` dans la CSP

- **Où** : `tauri.conf.json` (app.security.csp).
- **Constat** : requis par les styles inline (highlighter syntect côté backend, libs UI). Vecteur d'exfiltration CSS théorique uniquement, `script-src 'self'` restant strict.
- **Recommandation** : garder, mais documenter la justification dans la conf ; réévaluer si Tauri/outillage permet un jour les nonces.

### I1 — HMAC du vault : concaténation `main‖wal` sans séparateur de longueur

- **Où** : `vault_integrity.rs:52-61`.
- **Constat** : le MAC couvre `fichier principal ‖ WAL` concaténés ; deux découpages différents produisant la même concaténation seraient indistinguables. Aucune exploitation réaliste identifiée (les deux fichiers devraient rester des structures SQLite valides), mais encoder `len(main)` dans le MAC élimine l'ambiguïté à coût nul.

### I2 — Les messages d'erreur backend sont affichés tels quels dans l'UI

- **Où** : `src/core/errors.ts:53`, `auth-provider.tsx:82`.
- **Constat** : le frontend fait confiance au backend pour ne renvoyer que des messages sûrs — ce que `presentation/commands/error.rs` garantit effectivement aujourd'hui (tout détail technique collapse en « Erreur interne »). Le contrat repose sur la discipline du mapping backend ; F3 montre qu'il peut s'éroder. À surveiller en revue de code (la checklist existante peut l'intégrer).

### I3 — `tauri.localtest.conf.json` : transport non sécurisé (dev uniquement)

- **Où** : `src-tauri/tauri.localtest.conf.json` (`dangerousInsecureTransportProtocol: true`, endpoint HTTP localhost).
- **Constat** : sert aux tests locaux de l'updater ; **aucun workflow CI ne le référence** (vérifié). Risque uniquement si quelqu'un buildait une release avec `--config` par erreur. Une ligne d'avertissement en tête du fichier suffirait.

### I4 — Absence totale de journalisation (choix assumé)

- Aucun log nulle part, y compris sur les événements de sécurité (verrouillage de compte, détection d'altération du vault). Cohérent avec l'objectif de confidentialité, mais en cas d'incident il n'existe aucune trace. À documenter comme décision d'architecture ; une option : compteur/timestamp de « dernier événement de sécurité » chiffré dans le keystore, montrable à l'utilisateur.

### I5 — Limite résiduelle IPC documentée

- Le mot de passe transite par les buffers de désérialisation JSON de l'IPC Tauri avant le `Zeroizing` (`auth.rs:26`). Limite connue, documentée dans auth.md, non contournable sans changer de transport. Rien à faire.

### I6 — Import de bundle non atomique globalement

- Voir §2.5. Validation intégrale avant écriture + jours indépendants + résumé fidèle : acceptable. À mentionner dans la doc utilisateur de l'import.

---

## 5. Bonnes pratiques remarquables (à préserver)

- Envelope encryption complète et testée ; sels distincts auth/KEK ; nonces AEAD aléatoires de 192 bits.
- Keystore scellé par une clé de device dans le trousseau OS — neutralise le brute-force hors ligne sur vol de fichier.
- Évidence d'altération du vault : HMAC-SHA256 en comparaison constant-time, marqueur dirty pour les crashs, politique HardFail par défaut, **testée par un test d'intégration qui altère réellement le fichier**.
- Tokens de session jamais stockés en clair, même en mémoire (digest SHA-256, copie expurgée).
- Anti-brute-force avec verrouillage progressif et égalisation de timing contre l'énumération.
- `Zeroizing` systématique des secrets, dès la frontière IPC.
- Frontière d'erreurs stricte : tout détail technique collapse en `INTERNAL` avant la WebView.
- Markdown → HTML entièrement côté backend : comrak `unsafe_=false` + Ammonia whitelist, avec tests anti-XSS.
- SQL 100 % paramétré ; transactions atomiques ; `PRAGMA foreign_keys` par connexion.
- Architecture hexagonale sans aucune violation de dépendance ; IPC et opener confinés chacun dans un module frontend unique.
- CSP stricte ; capabilities minimales ; updater signé minisign avec clé publique épinglée.
- `.gitignore` couvrant secrets et bases (`.env`, `*.key`, `*.pem`, `*.db`) ; aucun secret dans l'historique git (vérifié).
- Documentation de sécurité (auth.md) **exacte** par rapport au code — rare et précieux.

---

## 6. Plan d'action recommandé

| Priorité | Action | Constats | Effort |
|---|---|---|---|
| 1 | Protéger la clé de signature : environment GitHub avec approbation requise, ou signature locale ; MFA matériel | E1 | Faible–moyen |
| 2 | Rendre la bascule de migration keystore récupérable après crash | E2 | Faible |
| 3 | Mettre à jour dompurify (≥ 3.4.11) et esbuild (≥ 0.28.1) | M3, F5 | Trivial |
| 4 | Réactiver la CI frontend + `cargo audit` + `pnpm audit` ; ajouter Renovate/Dependabot | M4 | Faible |
| 5 | Confiner les chemins d'export/import (dialogue côté Rust ou validation) | M1 | Moyen |
| 6 | Implémenter `change_password` (re-wrap DEK/MAC) ; afficher « aucune récupération possible » à l'inscription | M5 | Moyen |
| 7 | Purger les sessions au login ; documenter ou renforcer la sémantique de `require_session` | F1, M2 | Faible |
| 8 | Restreindre `opener` à un allowlist ; message générique dans le codec bundle ; corriger le message de politique de mot de passe | F2, F3, F4 | Trivial |
| 9 | Notes de design : longueur dans le HMAC, absence de logs, non-atomicité de l'import, justification `unsafe-inline` | I1, I4, I6, F6 | Documentation |

---

*Audit réalisé le 3 juillet 2026 sur le commit `cde8ddb`. Les sévérités tiennent compte du modèle de menace réel d'une application desktop locale mono-utilisateur ; plusieurs constats (M1, M2) ne deviendraient exploitables qu'après une compromission préalable de la WebView, aujourd'hui fortement entravée par la CSP et l'absence de contenu distant.*
