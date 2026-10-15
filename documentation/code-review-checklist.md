# Revue de code & checklist « projet propre »

A faire avant que l'utilisateur commit. Donc check le diff et appliquer la checklist pour donner un compte rendu de la situation a l'utilisateurs.

> Référence d'architecture : [../Claude.md](../Claude.md) · Sécurité : [auth.md](auth.md) · Persistance : [turso.md](turso.md)

Checklist pratique à dérouler **avant d'ouvrir une PR** et **pendant une revue**. Le but :
garder la **Clean Architecture** (règle de dépendance), la sécurité « niveau banque » et
l'homogénéité du code au fil des features (auth, profile, présence, CO₂…).

**Comment l'utiliser** : coche au fur et à mesure. Tout n'est pas applicable à chaque PR —
saute les sections hors-sujet. Les ⛔ sont des **règles dures** (à ne jamais enfreindre,
cf. [Claude.md §8](../Claude.md)). Les 🔒 sont critiques pour la sécurité.

---

## 0. Hygiène — la porte d'entrée (à passer avant toute revue)

Tant que ça ne passe pas, inutile de relire le fond.

- [ ] **Front** : `pnpm typecheck` ✅ — `pnpm lint` ✅ — `pnpm test` ✅ — `pnpm format:check` ✅
- [ ] **Rust** (depuis `src-tauri/`) : `cargo fmt --check` ✅ — `cargo clippy --all-targets -- -D warnings` ✅ (zéro warning) — `cargo test` ✅
- [ ] Le diff ne contient **que** ce qu'annonce le titre de la PR (pas de fichier généré — **exception** : [src/routeTree.gen.ts](../src/routeTree.gen.ts), committé volontairement et régénéré par le plugin —, pas de `console.log`/`dbg!`/`println!` oublié, pas de code commenté mort).
- [ ] Pas de `target/`, `dist/`, `node_modules/`, `.db`, ni secret commité (vérifier `.gitignore`).
- [ ] Commits descriptifs (Conventional Commits, **scope en PascalCase** — convention du projet : `feat(Profile): …`, `fix(Auth): …`, `feat(Tooltip): …`).

---

## 1. Règle de dépendance — le cœur de la Clean Architecture ⛔

> _« Les dépendances pointent toujours vers l'intérieur. »_ Le domaine ne connaît ni framework, ni BD, ni Tauri, ni React.

```
  Presentation  ──▶  Application  ──▶  Domain  ◀──  Infrastructure / Data
   (commands /        (use cases)     (entités,        (DB, crypto, IPC,
    React UI)                          contrats)         impl. des contrats)
```

- [ ] ⛔ Le **`domain`** n'importe **rien** de `application`, `infrastructure`/`data`, `presentation`, **React** ni **Tauri**. (Backend : aucun `use crate::infrastructure::…` dans `domain/`. Front : aucun import depuis `data/`, `presentation/`, `@tauri-apps/*` ou React dans `domain/`.)
- [ ] L'**inversion de dépendance** passe par des **traits** (Rust) / **interfaces** (TS), jamais par des types concrets.
- [ ] L'injection des implémentations concrètes se fait **uniquement au point de composition** (`lib.rs` côté Rust ; [app/container.ts](../src/app/container.ts) côté React, fourni par `ContainerProvider` et lu via `useContainer()`) — jamais dans le domaine, les use cases ni la présentation (aucun `new Tauri…Repository()` ailleurs).
- [ ] Le sens des `use`/`import` respecte le schéma : `presentation → application → domain` et `infrastructure → domain`. Aucune flèche ne sort du domaine.
- [ ] Une feature n'importe **aucune** autre feature (seule dérogation : `presence` → `commute`). Le partagé métier vit dans `entities/`, le générique dans `shared/`/`core/`, la composition de plusieurs features dans `pages/`. Vérifié par ESLint (`no-restricted-imports` par couche et par feature, [eslint.config.js](../eslint.config.js)).

---

## 2. Backend Rust (`src-tauri/`)

### 2.1 Domain — pur, aucune dépendance externe

[../src-tauri/src/domain/](../src-tauri/src/domain/)

- [ ] Entités = données + invariants métier (`User`, `Account`, `Session`, `Profile`). Validation **à la construction** (refuser un état invalide plutôt que de le valider plus tard).
- [ ] Les **ports** sont des traits focalisés sur le métier : un trait par responsabilité (`AccountRepository`, `ProfileRepository`, `PasswordHasher`, `KeyService`, `TokenGenerator`, `SessionStore`, `VaultManager`, `Clock`). Pas de « god trait ».
- [ ] ⛔ Aucun type tiers (libsql, argon2, types Tauri/serde de transport) n'apparaît dans une signature de trait du domaine.
- [ ] Les erreurs métier passent par `DomainError` ([error.rs](../src-tauri/src/domain/error.rs)) — un enum **exhaustif et spécifique** (`InvalidCredentials`, `AccountLocked`, `ProfileNotFound`…), jamais un type d'erreur tiers (`libsql::Error`, `anyhow`) renvoyé tel quel par un port.
- [ ] Les DTO de requête ≠ les entités persistées (ne pas réutiliser une struct de transport comme entité).

### 2.2 Application — use cases, dépend du Domain uniquement

[../src-tauri/src/application/](../src-tauri/src/application/)

- [ ] Un use case = une intention (`LoginUseCase`, `CreateProfileUseCase`, `ListProfilesUseCase`…). Il **orchestre** des ports, il n'implémente pas l'accès technique.
- [ ] Les dépendances sont injectées en `Arc<dyn Trait>` via `new(...)` (cf. [login.rs](../src-tauri/src/application/use_cases/login.rs)). Aucune construction d'impl concrète à l'intérieur.
- [ ] Renvoie un `Result<…, DomainError>` ; aucune dépendance Tauri/HTTP.
- [ ] **DTOs** dans `application/dto/` en `#[serde(rename_all = "camelCase")]` (frontière JS). 🔒 Un DTO **n'expose jamais** `password_hash`, `wrapped_dek`, `kek_salt`, `dek_nonce`, ni le DEK/KEK/token brut.
- [ ] Pas de logique technique (SQL, chiffrement) dans le use case — ça vit dans l'infra.
- [ ] Les règles de validation viennent de [domain/rules.rs](../src-tauri/src/domain/rules.rs) / [policy.rs](../src-tauri/src/domain/policy.rs), jamais d'un autre use case ; un helper partagé entre use cases va dans `application/services/`.
- [ ] Un use case qui écrit via **plus d'un** appel de dépôt passe par le port `UnitOfWork` (tout ou rien).

### 2.3 Infrastructure — implémente les ports

[../src-tauri/src/infrastructure/](../src-tauri/src/infrastructure/)

- [ ] Chaque impl correspond à un port du domaine (`LibsqlAccountRepository : AccountRepository`, `Argon2PasswordHasher : PasswordHasher`, …).
- [ ] Les bibliothèques tierces (libsql, argon2, getrandom) sont **enveloppées** ici et ne fuitent pas vers le haut.
- [ ] Erreurs techniques **mappées** vers `DomainError` (les détails SQL/crypto ne remontent pas tels quels).
- [ ] Migrations SQL embarquées et **versionnées** dans [../src-tauri/migrations/](../src-tauri/migrations/) (`keystore/` clair, `vault/` chiffré). Nouvelle migration = nouveau fichier numéroté, **jamais** d'édition d'une migration déjà livrée.
- [ ] Le choix keystore (scellé par la clé de device) vs vault (chiffré par le DEK) est respecté : 🔒 toute donnée de présence/CO₂ va dans `vault/`, jamais dans `keystore/`.
- [ ] ⛔ Aucune écriture du coffre hors de `LibsqlVaultManager::write` / `execute_write` / `unit_of_work` ; **jamais** de `execute("BEGIN")` manuel (garde CI).
- [ ] Travail bloquant (Argon2, gros fichiers) via `spawn_blocking`, pas sur l'exécuteur async. Fichier écrit pour l'utilisateur → écriture atomique ([infrastructure/fs](../src-tauri/src/infrastructure/fs/mod.rs)).

### 2.4 Presentation — commandes Tauri (frontière IPC)

[../src-tauri/src/presentation/](../src-tauri/src/presentation/)

- [ ] ⛔ **Aucune logique métier** dans une `#[tauri::command]` : elle récupère le `State`, appelle le use case, mappe l'erreur. C'est tout.
- [ ] Commandes `async` renvoyant un `Result`. `DomainError → AppError` sérialisable `{ code, message }` ([error.rs](../src-tauri/src/presentation/commands/error.rs)) — 🔒 détails internes masqués (`INTERNAL`).
- [ ] 🔒 Toutes les entrées venant du front sont **validées/normalisées** (trim, longueurs) — côté domaine de préférence.
- [ ] Nouvelle commande = enregistrée dans le `invoke_handler` de [lib.rs](../src-tauri/src/lib.rs), ajoutée à la **liste blanche d'isolation** ([isolation/index.js](../src-tauri/isolation/index.js) — un test le vérifie), déclarée dans le `commands.ts` de sa feature (registre typé) et nommée dans [config.ts](../src/core/config.ts) (exhaustivité vérifiée à la compilation).
- [ ] 🔒 ⛔ Aucun **chemin de fichier** en paramètre de commande : le dialogue natif s'ouvre côté Rust ([file_dialogs.rs](../src-tauri/src/presentation/file_dialogs.rs)).
- [ ] 🔒 Toute commande du coffre appelle `require_session` en première ligne (et à nouveau après un dialogue natif).

### 2.5 Composition root & qualité Rust

- [ ] Toute la DI vit dans `build_state` ([lib.rs](../src-tauri/src/lib.rs)) ; [main.rs](../src-tauri/src/main.rs) reste minimal. Plus `main`/`lib` est mince, plus la zone testable est large.
- [ ] ⛔ Pas d'`unwrap()` / `expect()` / `panic!` dans les chemins d'exécution (un `Mutex` empoisonné = crash). Propager un `Result`.
- [ ] 🔒 Matériel sensible (mot de passe, KEK, DEK) effacé via `Zeroizing` / `zeroize` (cf. [login.rs](../src-tauri/src/application/use_cases/login.rs) ligne `Zeroizing::new`).
- [ ] Pas de `clone()` gratuit ni d'allocation inutile sur les chemins chauds ; `&str`/`&[u8]` plutôt que `String`/`Vec` quand l'emprunt suffit.
- [ ] `cargo clippy --all-targets -- -D warnings` passe (les `#[allow(...)]` ponctuels sont justifiés, ex. `too_many_arguments` sur un constructeur de use case).

---

## 3. Frontend React (`src/`) — feature-first

### 3.1 core/ — transverse

[../src/core/](../src/core/)

- [ ] ⛔ **Seul** [core/ipc.ts](../src/core/ipc.ts) importe `@tauri-apps/api` (règle ESLint `no-restricted-imports`). Tout le reste passe par les repositories.
- [ ] Les noms de commandes IPC sont centralisés dans [config.ts](../src/core/config.ts) et typés par le registre `CommandRegistry` ([ipc.ts](../src/core/ipc.ts)) : pas de `invoke<T>` à type deviné, pas de chaîne magique.
- [ ] Les erreurs IPC sont normalisées via [errors.ts](../src/core/errors.ts) (`AppError` / `normalizeError`).

### 3.2 domain/ (feature) — entités, interfaces, use cases purs

ex. [../src/features/profile/domain/](../src/features/profile/domain/)

- [ ] ⛔ Le domaine front n'importe ni React, ni `@tauri-apps/*`, ni la couche `data/`. 100 % framework-agnostique.
- [ ] L'interface de repository (`ProfileRepository`, `AuthRepository`) vit dans `domain/repositories/` ; les use cases dépendent de l'interface, pas de l'impl Tauri.
- [ ] Les entités du domaine sont distinctes des DTO de transport.

### 3.3 data/ (feature) — DTO, mappers, repository

ex. [../src/features/profile/data/](../src/features/profile/data/)

- [ ] L'impl de repository (`TauriProfileRepository`) appelle `invoke(...)` via `core/ipc` puis **mappe** DTO → entité du domaine (un mapper dédié, testé).
- [ ] Les DTO reflètent exactement le contrat Rust (camelCase). Tout changement de DTO Rust ↔ mis à jour ici **en même temps**.
- [ ] Le mapping vit dans `mappers/` (pas dans le composant, pas dans le hook).

### 3.4 presentation/ (feature) — provider, hooks, composants

ex. [../src/features/profile/presentation/](../src/features/profile/presentation/)

- [ ] ⛔ Les **composants** ne font ni appel IPC, ni logique métier : ils consomment des hooks. UI séparée de la logique.
- [ ] Les **hooks** (`useAuth`, `useProfile`) orchestrent les use cases ; les dépôts viennent de `useContainer()`, jamais d'un `new` local.
- [ ] État serveur via **TanStack Query** (clés incluant le `profileId`), écritures via `useCommand()` ; `SESSION_EXPIRED` n'est **jamais** traité localement (géré une fois par le query client). État d'UI : local ou contexte ([ADR 0003](adr/0003-tanstack-query.md)).
- [ ] Composant nettoie ses effets (`useEffect` cleanup : timers, listeners — cf. [use-session-timer.ts](../src/features/auth/presentation/hooks/use-session-timer.ts)).

### 3.5 Routing (TanStack Router, file-based)

[../src/routes/](../src/routes/) · [../src/router.ts](../src/router.ts) — conventions détaillées dans [Claude.md §6](../Claude.md)

- [ ] ⛔ Les fichiers de [src/routes/](../src/routes/) restent **minces** : `createFileRoute` + `beforeLoad` + import d'une page. **Aucune UI métier ni logique** dedans — les écrans vivent dans `features/<feature>/presentation/pages/`.
- [ ] ⛔ [routeTree.gen.ts](../src/routeTree.gen.ts) **jamais édité à la main** (généré ; ignoré ESLint/Prettier). Route ajoutée/renommée → régénéré (`pnpm dev` ou `pnpm build`) et committé **dans la même PR**.
- [ ] **Hash history** conservée dans [router.ts](../src/router.ts) (le protocole asset Tauri n'a pas de fallback SPA en prod) ; le routeur reste un **singleton module-level**, jamais créé dans un composant.
- [ ] La garde d'auth passe par `beforeLoad` + `redirect` (`_authenticated`, `/login`) et le contexte `{ auth }`. 🔒 Toute nouvelle page sous contenu authentifié est **sous `_authenticated/`** (jamais en route racine).
- [ ] L'auth reste **router-agnostique** : pas de `navigate()`/import du routeur dans `features/auth` — les changements d'état passent par le `router.invalidate()` de [App.tsx](../src/App.tsx).
- [ ] Navigation **typée** (`<Link to>` / `useNavigate`, params via `Route.useParams()`) — pas de `window.location` ni de chaîne d'URL bricolée. Les composants réutilisables reçoivent des **callbacks** (`onDone`…) plutôt que d'importer le routeur.
- [ ] Bon choix page vs dialog : destination (liste, création/édition) = **page routée** ; interaction contextuelle (confirmation, étape de wizard) = **dialog**.
- [ ] Une page paramétrée (`$id`) gère **chargement** et **introuvable** (les données contexte arrivent en async — cf. [\$commuteId.edit.tsx](../src/routes/_authenticated/commutes/$commuteId.edit.tsx)).
- [ ] Le plugin router reste **désactivé sous vitest** dans [vite.config.ts](../vite.config.ts) (le code-splitting casse les chunks lazy en jsdom) ; les devtools routeur restent chargés **en dev uniquement**.

### 3.6 Qualité React / TypeScript

- [ ] ⛔ Typage strict — **pas d'`any`** (ni `as` abusif). Préférer `unknown` + narrowing.
- [ ] Alias `@/...` partout — **pas de `../../../`**.
- [ ] Conventions de nommage : fichiers `kebab-case`, composants `PascalCase`, hooks `useCamelCase`.
- [ ] Pas de secret ni de logique de sécurité côté front (le front est dans la WebView = non fiable).
- [ ] Textes visibles passés par i18n ([locales](../src/core/i18n/locales/), clés `en` **et** `fr` à jour) — pas de chaîne en dur dans le JSX.
- [ ] Composants UI partagés réutilisés depuis [../src/components/ui/](../src/components/ui/) (shadcn) plutôt que redéveloppés.

---

## 4. Sécurité 🔒 (exigence « niveau banque » — détails dans [auth.md](auth.md))

- [ ] ⛔ Mot de passe **jamais** stocké en clair ni loggé ; haché en Argon2id.
- [ ] ⛔ `password_hash`, DEK, KEK, `wrapped_dek`, `kek_salt`, `token` brut **jamais** envoyés au frontend ni écrits dans un log.
- [ ] Données de présence/CO₂ uniquement dans le **vault chiffré** ; rien de sensible dans `keystore.db` (clair).
- [ ] 🔒 Secrets (token Turso Phase 2, clés) **restent côté Rust** ; aucun secret commité (vérifier le diff + `.gitignore`).
- [ ] Session : en mémoire uniquement, **jamais** dans `localStorage`/`sessionStorage` ; expiration absolue 15 min respectée ; coffre fermé + DEK oublié au logout/expiration.
- [ ] Anti-bruteforce préservé (verrouillage, erreur **générique** `InvalidCredentials`, timing égalisé pour username inconnu — cf. [login.rs](../src-tauri/src/application/use_cases/login.rs)).
- [ ] **Tauri capabilities** : moindre privilège — toute nouvelle permission dans [../src-tauri/capabilities/](../src-tauri/capabilities/) est justifiée et minimale (rappel : une `#[tauri::command]` applicative ne nécessite **pas** d'entrée capability).
- [ ] Mémoire sensible zeroïsée ; pas de copie persistante du mot de passe.
- [ ] Entrées bornées (longueurs, nombres, tailles) via `domain/rules.rs` — y compris pour l'import de fichiers.
- [ ] `dangerouslySetInnerHTML` uniquement sur du HTML assaini côté Rust (règle ESLint).
- [ ] Aucun test ne « prouve » une protection uniquement sur le cas nominal : tester aussi le **contournement** (fichiers annexes supprimés/créés, appels concurrents, entrées géantes).

---

## 5. Tests

- [ ] Nouveau use case Rust → test unitaire avec **ports mockés** (pas de vraie BD pour tester l'orchestration).
- [ ] Nouvelle impl crypto/persistence → test ciblé (ex. hash/verify, wrap/unwrap, round-trip repo).
- [ ] Parcours critique → test d'intégration backend ([integration_tests.rs](../src-tauri/src/integration_tests.rs)) : happy path + erreurs clés (lockout, session expirée).
- [ ] Front : mapper testé, repository testé via `mockIPC`, hook/timer testés, parcours écran testé (Testing Library) — voir les `*.test.ts(x)` existants.
- [ ] Les tests vérifient un **comportement**, pas l'implémentation ; ils ne dépendent pas de l'horloge réelle (`Clock` injecté) ni de l'ordre d'exécution.
- [ ] 🔒 Un test prouve qu'une donnée sensible est bien chiffrée/absente du front quand c'est pertinent.

---

## 6. Documentation & cohérence

- [ ] Changement d'architecture/sécurité/persistance → [../Claude.md](../Claude.md), [auth.md](auth.md) ou [turso.md](turso.md) mis à jour **dans la même PR**.
- [ ] Nouvelle commande IPC → tableau des commandes ([auth.md](auth.md) §Commandes IPC) à jour.
- [ ] [../README.md](../README.md) à jour si l'installation, les commandes ou la structure changent.
- [ ] Liens de fichiers dans la doc toujours valides (chemins relatifs).
- [ ] Décision non triviale (compromis, « limite connue ») notée dans la doc plutôt que perdue dans un commit.

---

## 7. Recette — ajouter une feature de bout en bout (sanity check)

Pour une nouvelle entité (ex. `Presence`), l'ordre **domain → application → infra → presentation**, des deux côtés :

1. **Rust domain** : entité + invariants, trait `XRepository`, variantes `DomainError`.
2. **Rust application** : use case(s) (`Arc<dyn …>` injectés) + DTO camelCase.
3. **Rust infra** : `LibsqlXRepository` + migration SQL (dans `vault/` si sensible).
4. **Rust presentation** : commandes fines + mapping `AppError` ; câblage dans `build_state` + `invoke_handler` ([lib.rs](../src-tauri/src/lib.rs)).
5. **Front domain** : entité, interface repository, use cases purs.
6. **Front data** : DTO (miroir Rust), mapper, `TauriXRepository`.
7. **Front presentation** : provider (composition root), hook, composants (UI sans logique) ; si l'écran est une **destination**, page dans `presentation/pages/` + fichier de route mince sous `src/routes/_authenticated/` (puis régénérer `routeTree.gen.ts`).
8. **Transverse** : noms de commandes dans [config.ts](../src/core/config.ts), clés i18n `en`+`fr`.
9. **Tests + doc** des deux côtés.
10. Dérouler les §0–6 de cette checklist.

---

## 8. Red flags — odeurs à refuser en revue

- 🚩 Un `import` / `use` qui **sort** du domaine (vers infra, data, React, Tauri).
- 🚩 De la logique métier dans une commande Tauri ou un composant React.
- 🚩 Un type tiers (libsql, argon2, axum-like) dans une signature de trait/interface du domaine.
- 🚩 `unwrap()`/`panic!` (Rust) ou `any`/`../../../` (TS) qui se glissent dans le diff.
- 🚩 `invoke(...)` ou `@tauri-apps/api` ailleurs que dans `core/ipc.ts`.
- 🚩 Un `path: String` dans une commande Tauri, un `execute("BEGIN")` dans un dépôt, un `new Tauri…Repository()` hors de `app/container.ts`.
- 🚩 Un état de sécurité (marqueur, empreinte, compteur) stocké dans un fichier non authentifié.
- 🚩 Un champ sensible (hash, clé, token) dans un DTO renvoyé au front.
- 🚩 Édition d'une migration déjà livrée au lieu d'en ajouter une nouvelle.
- 🚩 Chaîne UI en dur (sans i18n) ou clé présente dans `en` mais pas `fr`.
- 🚩 Une feature qui en importe une autre par ses couches internes.
- 🚩 De l'UI métier ou de la logique dans un fichier de `src/routes/`, ou `routeTree.gen.ts` édité à la main.
- 🚩 Une page authentifiée déclarée hors de `_authenticated/`, ou une navigation via `window.location` au lieu de `Link`/`useNavigate`.
- 🚩 `cargo clippy`/`pnpm lint` qui ne passent plus « juste pour cette fois ».

---

## Sources (clean / hexagonal architecture)

- [Master Hexagonal Architecture in Rust — howtocodeit.com](https://www.howtocodeit.com/guides/master-hexagonal-architecture-in-rust) — ports/adapters via traits, séparation DTO/entité, erreurs de domaine, `main` minimal.
- [Hexagonal Architecture in Rust — Barrage](https://www.barrage.net/blog/technology/how-to-apply-hexagonal-architecture-to-rust)
- [Ports and Adapters in Rust with traits — Medium](https://medium.com/@bugsybits/ports-and-adapters-in-rust-but-with-traits-instead-of-pain-e88eabc09cb1)
- [Clean Architecture in React — Alex Kondov](https://alexkondov.com/full-stack-tao-clean-architecture-react/)
- [Modularizing React Applications — Martin Fowler](https://martinfowler.com/articles/modularizing-react-apps.html)
- [React Clean Architecture guide — DhiWise](https://www.dhiwise.com/blog/design-converter/react-clean-architecture-guide-for-better-code-structure)
- Référence interne : [../Claude.md](../Claude.md) (principes), [auth.md](auth.md) (sécurité), [turso.md](turso.md) (persistance).
