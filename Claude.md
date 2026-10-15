# CLAUDE.md

Contexte projet pour Claude Code. Application desktop **Tauri v2** avec un backend
**Rust** et un frontend **React + TypeScript**, organisée selon la **Clean Architecture**.

---

## 1. Vue d'ensemble

- **Quoi** : **BasicPresence** — application de présence (bureau / remote) avec le **mode de déplacement** utilisé, afin de calculer les **émissions de CO₂** par personne. Les données de présence sont jugées **très confidentielles** (exigence de sécurité « niveau banque »).
- **Backend** : Rust (logique métier, accès système, persistance) exposé via les commandes Tauri (IPC).
- **Frontend** : React + TypeScript dans la WebView (UI uniquement).
- **Communication** : IPC Tauri (`invoke` côté JS ↔ `#[tauri::command]` côté Rust) + événements.
- **Offline-first** : persistance 100 % locale et chiffrée en v1 (moteur libSQL/Turso) ; synchro cloud prévue en Phase 2.

Tauri suit un modèle **Core-Shell** : le _Core_ Rust ne doit jamais exposer d'accès
système direct au _Shell_ (la WebView). Tout passe par des commandes validées.

> 📚 Documentation détaillée : [documentation/auth.md](documentation/auth.md) (sécurité / authentification), [documentation/turso.md](documentation/turso.md) (persistance libSQL / Turso), [documentation/adr/](documentation/adr/README.md) (décisions d'architecture) et [documentation/audit-securite-2026-10-remediation.md](documentation/audit-securite-2026-10-remediation.md) (dernier audit).

---

## 2. Stack technique

| Couche      | Technologies                                                                 |
| ----------- | ---------------------------------------------------------------------------- |
| Frontend    | React 19, TypeScript, Vite 7, **TanStack Router** (file-based), **TanStack Query** (état serveur) + React Context (état d'UI), conteneur d'injection (`core/di`) |
| Bridge      | `@tauri-apps/api` (core)                                                      |
| Backend     | Rust (édition 2024), Tauri v2 (**pattern d'isolation**)                       |
| Persistance | **libSQL** (moteur de Turso), local + **chiffré au repos** (AES-256-CBC)      |
| Crypto      | **Argon2id** (hash + dérivation de clé), **XChaCha20-Poly1305** (envelope)   |
| Tests       | `cargo test` (Rust), **Vitest** + Testing Library (front)                    |
| Outillage   | **pnpm 11** (âge minimum 7 j, builds bloqués), ESLint 9 (typé + frontières de couches + jsx-a11y), Prettier, cargo-deny, zizmor |

---

## 3. Commandes

Le gestionnaire de paquets est **pnpm**.

```bash
# Développement (lance Vite + la fenêtre Tauri avec hot-reload)
pnpm tauri dev

# Build de production (binaire natif)
pnpm tauri build

# Frontend seul
pnpm dev           # serveur Vite
pnpm build         # tsc && vite build
pnpm typecheck     # tsc --noEmit
pnpm lint          # eslint .
pnpm format        # prettier --write
pnpm test          # vitest run

# Backend Rust (depuis src-tauri/)
cargo check        # vérification rapide
cargo test         # tests unitaires + intégration
cargo clippy -- -D warnings   # lint Rust (zéro warning)
cargo fmt          # formatage
cargo deny check   # advisories, licences, sources des crates
```

> Privilégie l'exécution d'un **test ciblé** plutôt que toute la suite pendant le dev.

---

## 4. Architecture — principes Clean Architecture

La règle fondamentale est la **règle de dépendance** : _les dépendances pointent
toujours vers l'intérieur_. Le domaine ne connaît rien des frameworks, de la base de
données, de Tauri, ni de React.

```
  Presentation  ──▶  Application  ──▶  Domain  ◀──  Infrastructure
   (commands /        (use cases)     (entités,        (DB, clients,
    React UI)                          contrats)         impl. des contrats)
```

- **Domain** : ne dépend de rien. Entités, value objects, contrats (traits/interfaces de repositories & services), erreurs métier.
- **Application** : dépend uniquement du Domain. Orchestration des cas d'usage (use cases), DTOs.
- **Infrastructure** : implémente les contrats du Domain (accès BD, crypto, sessions, etc.).
- **Presentation** : point d'entrée (commandes Tauri côté Rust, composants/hooks côté React). Fine couche de traduction.

L'inversion de dépendance se fait via des **traits** (Rust) / **interfaces** (TS), et
l'injection des implémentations concrètes se fait au **point de composition** (au
démarrage de l'app), jamais dans le domaine.

---

## 5. Structure du backend Rust (`src-tauri/`)

```
src-tauri/
├── Cargo.toml
├── tauri.conf.json
├── capabilities/                # Permissions Tauri v2 (moindre privilège)
├── migrations/                  # SQL embarqué (keystore/ et vault/)
└── src/
    ├── main.rs                  # Entrée binaire — appelle basic_presence_lib::run()
    ├── lib.rs                   # POINT DE COMPOSITION : setup, DI (build_state), commandes
    ├── integration_tests.rs     # Test e2e backend (register→login→session + chiffrement)
    ├── domain/                  # ── Domain (pur, aucune dépendance externe)
    │   ├── policy.rs            #    AuthPolicy, KdfParams, règles de mot de passe (politiques de sécurité)
    │   ├── rules.rs             #    TOUTES les règles de validation/bornes des entrées (commandes + import)
    │   ├── entities/            #    user, account, session, profile, profile_settings, presence, commute,
    │   │                        #    trip, emission_factor, co2_settings, task_preset, work_entry, work_day_schedule,
    │   │                        #    security_event
    │   ├── repositories/        #    traits AccountRepository, ProfileRepository, ProfileSettingsRepository,
    │   │                        #    PresenceRepository, CommuteRepository, EmissionFactorRepository,
    │   │                        #    TaskPresetRepository, WorkEntryRepository
    │   ├── services/            #    ports : PasswordHasher, KeyService, TokenGenerator, SessionStore,
    │   │                        #    VaultManager, UnitOfWork, Clock + service pur Co2Calculator
    │   │                        #    (+ repositories/security_log : journal de sécurité)
    │   └── error.rs             #    DomainError
    ├── application/             # ── Application (dépend du Domain)
    │   ├── services/            #    helpers partagés entre use cases : credentials (unlock/seal des clés,
    │   │                        #    verrouillage), commute_builder, factor_maps
    │   ├── use_cases/           #    auth + profils + présences (set/list/delete/import) +
    │   │                        #    commute (create/update/delete/list), emission_factors, presence_trips +
    │   │                        #    task_preset (create/list/update/delete), work_entries/work_schedule (get/set),
    │   │                        #    profile_settings (get/set)
    │   └── dto/                 #    *Dto (serde camelCase) : user/login/session/profile/profile_settings/
    │                            #    presence/commute/trip/emission_factor/task_preset/work_entry
    ├── infrastructure/          # ── Infrastructure (implémente les ports)
    │   ├── crypto/              #    Argon2PasswordHasher, Argon2KeyService (AAD v2), RandomTokenGenerator,
    │   │                        #    device_key (trousseau OS), run_blocking (spawn_blocking)
    │   ├── persistence/         #    db (libsql), migrations (transactionnelles), *_repository,
    │   │                        #    vault (write/unit_of_work), vault_integrity + integrity_store (état scellé
    │   │                        #    dans le keystore), security_log_repository, keystore_bootstrap
    │   ├── fs/                  #    écriture atomique (temp + fsync + rename)
    │   ├── session/             #    InMemorySessionStore
    │   ├── clock.rs             #    SystemClock
    │   └── config.rs            #    AppConfig (chemins, params Argon2, politique session/lockout)
    └── presentation/            # ── Presentation (frontière IPC)
        ├── commands/            #    auth.rs, profile.rs, settings.rs, presence.rs, commute.rs, work_hours.rs,
        │                        #    export.rs, transfer.rs, legal.rs (fines) + error.rs (AppError)
        ├── file_dialogs.rs      #    dialogues natifs ouverts CÔTÉ RUST + handles d'import (aucun chemin dans l'IPC)
        └── state.rs             #    AppState injecté via .manage()
```

`src-tauri/isolation/` = application d'isolation Tauri (HTML + JS sans dépendance) : **liste blanche
des commandes** ; toute nouvelle commande doit y être ajoutée (un test le vérifie).

### Conventions backend

- `main.rs` reste minimal. Toute la configuration vit dans `lib.rs` via `run()`.
- **Composition root** dans `lib.rs` (`build_state`) : construire les impls d'infrastructure, les injecter dans les use cases (`Arc<dyn Trait>`), assembler l'`AppState`, puis `.manage(state)`.
- Une **commande Tauri** ne contient pas de logique métier : elle récupère le `State`, appelle le use case, et mappe `DomainError` → `AppError` sérialisable (`{ code, message }`, détails internes masqués).
- Les commandes sont `async` et renvoient un `Result`. **Aucun chemin de fichier** en paramètre : les dialogues natifs sont ouverts côté Rust (`presentation/file_dialogs.rs`).
- **Écritures du coffre** : uniquement via `LibsqlVaultManager::write` / `execute_write` / `unit_of_work` (connexion dédiée + verrou). Jamais de `execute("BEGIN")` manuel. Plusieurs écritures dans un use case → port `UnitOfWork`.
- **Validation** : règles dans `domain/rules.rs` (ou `policy.rs`), jamais importées d'un autre use case.
- Pas d'`unwrap()` / `panic!` dans les chemins d'exécution. `cargo fmt` + `cargo clippy -- -D warnings` doivent passer **sans warning**.
- Matériel sensible (mot de passe, KEK, DEK) effacé via `zeroize` / `Zeroizing`.

---

## 6. Structure du frontend React (`src/`)

Organisation **feature-first** : chaque feature porte ses propres couches `domain`/`data`/`presentation`. Le transverse vit dans `core/`.

```
src/
├── main.tsx                     # Entrée React — ContainerProvider > QueryClientProvider > AuthProvider > updater > App
├── App.tsx                      # Bridge AuthContext → routeur + logout central (SESSION_EXPIRED) + purge du cache
├── router.ts                    # Singleton createRouter (hash history, contexte { auth })
├── routeTree.gen.ts             # GÉNÉRÉ par @tanstack/router-plugin — committé, jamais édité
├── app/                         # COMPOSITION ROOT : container.ts (SEUL lieu des `new Tauri…Repository()`),
│                                #    query-client.ts, session-aware-updater.tsx, __tests__/ (tests app complète)
├── routes/                      # Routes file-based (fichiers MINCES : composition seulement)
├── pages/                       # Écrans composant plusieurs features (settings/ + sections/)
├── features/<feature>/          # Une capacité métier ; n'importe AUCUNE autre feature (sauf presence → commute)
│   ├── domain/                  #    entités, ports (interfaces), use cases purs
│   ├── data/                    #    dto, mappers, Tauri*Repository, commands.ts (déclaration IPC typée)
│   ├── presentation/            #    providers (TanStack Query), hooks, composants, pages
│   └── di.ts                    #    ports pris dans le conteneur (augmentation de `Container`)
├── entities/                    # Modèles/contextes partagés entre features : profile (actif), presence,
│                                #    commute (formats, libellés), work-day (actions jour)
├── shared/ components/ lib/     # UI kit, hooks et formats génériques
└── core/                        # ipc.ts (SEUL import @tauri-apps/api, registre typé), di.ts, query.ts,
                                 #    errors.ts, i18n, config.ts (COMMANDS, vérifié exhaustif)
```

Features : `auth`, `profile`, `profile-settings`, `presence`, `commute`, `work-hours`, `export`,
`profile-transfer`, `stats`, `methodology`, `licence`, `updater`.

> **Frontières vérifiées par ESLint** ([eslint.config.js](eslint.config.js)) : une couche n'importe
> que celles du dessous (`app` > `routes` > `pages` > `features` > `entities` > `shared` > `core`),
> une feature n'importe aucune autre feature, le `domain` reste pur, les imports relatifs parents
> (`../`) sont interdits (alias `@/` obligatoire).
>
> - **Vocabulaire partagé** (`formatCo2`, `summarizeSegments`, libellés de modes, profil actif,
>   présences, durée) : dans `entities/` ou `shared/`, importable par toutes les features.
> - **Dérogation unique : `presence` → `commute`.** L'empreinte CO₂ étant attachée à un jour de présence,
>   le dialog de présence consomme `SegmentEditor`/`useCommute`. Dérogation **délibérée** à l'isolation
>   des features, justifiée par le couplage métier (détail dans [documentation/auth.md](documentation/auth.md))
>   — à ne pas étendre à d'autres features sans le même niveau de justification.

> Le header authentifié contient : le timer de session, les **badges de résumé mensuel**
> (heures travaillées + CO₂ émis pour le mois en cours — affichés uniquement sur `/` via
> `useLocation()`) et le **badge de profil** (tout à droite) qui ouvre le menu compte
> (changer/ajouter/éditer/supprimer un profil, langue, thème, déconnexion).
> `routes/_authenticated.tsx` injecte `onSessionExpired={logout}` aux providers et
> `onLogout`/`onOpenCommutes` au badge — les features `profile`/`commute` ne dépendent
> ni de `auth` ni du routeur (langue/thème viennent de `core`/`shared`).
> Le résumé mensuel est calculé côté front via `useMemo` sur `PresenceContext.presencesByDay`
> (champ `workMinutes` chargé par le backend) filtré par `currentMonth` (état partagé dans
> `PresenceContext`). Voir § `presence → commute` ci-dessous pour la dépendance de feature.

### Routing (TanStack Router, file-based)

- **Fichiers de routes minces** : un fichier de `src/routes/` fait `createFileRoute(...)` + `beforeLoad` éventuel et importe un composant de page — **aucune UI métier ni logique** dedans. Les écrans routés vivent dans `features/<feature>/presentation/pages/`.
- **`routeTree.gen.ts` est généré** par le plugin Vite (`@tanstack/router-plugin`, déclaré **avant** `react()` avec `autoCodeSplitting: true`). Il est **committé** mais **jamais édité à la main** (ignoré par ESLint/Prettier). Le plugin est **désactivé sous vitest** (`process.env.VITEST`) : sa transformation de code-splitting casse les chunks lazy en jsdom ; l'arbre committé suffit aux tests.
- **Hash history obligatoire** (`createHashHistory` dans `router.ts`) : le protocole asset de Tauri n'a pas de fallback SPA en prod — un chargement sur un chemin profond ferait un 404 en browser history.
- **Garde d'auth** : pattern officiel « authenticated routes » — contexte routeur `{ auth }`, route pathless `_authenticated` (`beforeLoad` → `redirect("/login")`), `/login` redirige vers `/` si connecté. **L'auth reste router-agnostique** : `App.tsx` fait `router.invalidate()` quand `isAuthenticated` change ; c'est l'unique mécanisme qui transforme login/logout/idle-timeout/expiration en redirection. Pas de `navigate()` dans la feature auth.
- Le routeur est un **singleton module-level** (`src/router.ts`) — jamais créé dans un composant (StrictMode).
- **Navigation typée** : `<Link to="...">` / `useNavigate()` (params vérifiés par TS via `Register`). Les composants réutilisables entre contextes reçoivent des **callbacks** (`onDone`, `onOpenCommutes`) plutôt que d'importer le routeur.
- **Page vs dialog** : une destination (liste, formulaire de création/édition) = une **page routée** ; une interaction contextuelle (confirmation de suppression, sélection dans un wizard) = un **dialog**.
- Nouvelle page = ① composant dans `features/<feature>/presentation/pages/`, ② fichier de route sous `src/routes/_authenticated/`, ③ clés i18n `fr` + `en`, ④ relancer `pnpm dev`/`pnpm build` pour régénérer `routeTree.gen.ts` avant commit.

### Conventions frontend

- **Seul `core/ipc.ts`** importe `@tauri-apps/api` (règle ESLint) ; les plugins Tauri ne sont importés que par leur fichier frontière. Le reste passe par les repositories.
- **IPC typé** : chaque commande est déclarée dans le `data/commands.ts` de sa feature (augmentation de `CommandRegistry`) ; `invoke(COMMANDS.x, args)` infère arguments et résultat.
- **Injection** : les dépôts viennent de `useContainer()` ; nouvelle feature = `di.ts` + entrée dans `app/container.ts`.
- **État serveur** : TanStack Query (clés avec `profileId`) ; écritures via `useCommand()`. Ne **jamais** traiter `SESSION_EXPIRED` localement (géré par le query client).
- Une **impl de repository** (couche data) appelle `invoke('cmd', {...})` puis mappe le DTO → entité du domaine.
- Les **composants** ne contiennent ni appel IPC, ni logique métier complexe ; ils consomment des données via des hooks.
- Les **hooks** orchestrent les use cases (ex : `useAuth`).
- **Alias de chemin** `@/...` (configuré dans `tsconfig.json` + `vite.config.ts`) — pas de `../../../`.
- Fichiers `kebab-case`, composants `PascalCase`, hooks `useCamelCase`. Typage strict, pas d'`any`.

---

## 7. Sécurité (résumé — détails dans [documentation/auth.md](documentation/auth.md))

- **Mot de passe** : haché en **Argon2id** (jamais stocké en clair, jamais envoyé au front).
- **Chiffrement au repos** : _envelope encryption_ à clé dérivée du mot de passe. Un DEK aléatoire chiffre le coffre `vault.db` ; le DEK est enveloppé (XChaCha20-Poly1305) par un KEK dérivé du mot de passe. Le coffre ne s'ouvre qu'après login.
- **Deux bases, toutes deux chiffrées au repos** : `keystore.db` (hash + DEK/clé MAC enveloppés) **scellé par une clé de device du trousseau OS** (libsecret/Keychain/Credential Manager) → fichier volé inutilisable hors de l'appareil ; `vault.db` chiffré par le DEK, avec **évidence d'altération** (HMAC-SHA256 sidecar, libSQL CBC n'étant pas authentifié) — politique **`HardFail` par défaut** (un mismatch après arrêt propre refuse l'ouverture).
- **Session** : 15 min **absolue** + **inactivité** (UI 5 min, **backend 6 min**), en mémoire uniquement (jamais persistée, tokens stockés **hachés SHA-256**) → mot de passe redemandé à chaque démarrage. **Anti-bruteforce** progressif (verrouillage doublé à chaque échec, compteur atomique, tentatives sérialisées). L'expiration est **appliquée côté Rust** via `require_session`. Session globale assumée ([ADR 0001](documentation/adr/0001-session-globale.md)).
- **KEK** dérivée avec un profil Argon2id dédié (64 MiB, t=3) stocké par compte ; clés enveloppées avec **AAD** (rôle + compte) ; `change_password` ré-enveloppe sans re-chiffrer le coffre.
- **Intégrité du coffre** : référence HMAC et « session ouverte » **dans le keystore scellé** (jamais dans des fichiers annexes) ([ADR 0002](documentation/adr/0002-integrite-du-coffre.md)) ; journal de sécurité minimal montré après connexion.
- **Aucun secret** (token Turso, clés) commité ni exposé au frontend — ils restent côté Rust.
- ⚠️ **Contrainte runtime (Linux)** : un Secret Service (GNOME Keyring/KWallet) doit tourner pour ouvrir le keystore ; sinon le démarrage échoue explicitement.

---

## 8. Règles importantes (à ne jamais enfreindre)

- **Ne jamais violer la règle de dépendance.** Le `domain` n'importe rien de `application`, `infrastructure`/`data`, `presentation`, React ou Tauri.
- **Ne jamais mettre de logique métier dans une commande Tauri ni dans un composant React.**
- **Sécurité Tauri** : minimum de permissions dans `capabilities/` (moindre privilège). Valider toutes les entrées du frontend.
- **Aucun secret** commité ni exposé au frontend. Les secrets restent côté Rust.
- **Seul `core/ipc.ts`** (front) parle à `@tauri-apps/api`.
- **Aucun chemin de fichier** venant de la WebView ; aucun `BEGIN` manuel ; aucun état de sécurité dans un fichier non authentifié.
- Épingler les versions de dépendances (versions exactes, overrides exacts).

---

## 9. Workflow attendu

1. Implémenter en respectant les couches (domain → application → infra/presentation).
2. Avant de terminer une série de changements :
   - Rust : `cargo fmt`, `cargo clippy --all-targets -- -D warnings`, `cargo test`, `cargo deny check`.
   - Front : `pnpm typecheck`, `pnpm lint`, `pnpm test`, `pnpm build`.
3. Préférer les tests ciblés pendant le dev ; lancer la suite complète avant un commit.
4. Commits descriptifs (style Conventional Commits recommandé).

---

## 10. État du projet & feuille de route

**Implémenté (socle v1)** : architecture propre Rust + React, **routeur TanStack file-based** (pages `/login`, `/` calendrier, `/commutes` + création/édition de presets en pages), persistance libSQL locale chiffrée, première entité **User** + authentification complète (Argon2id, envelope encryption, session 15 min, anti-bruteforce). Entité **Profile** (profils de présence) : CRUD complet dans le coffre chiffré, gestion **multi-profils** (création, sélection du profil actif, édition, suppression) ; après login, l'accueil propose la création si aucun profil, sinon un badge (photo + nom) en haut à droite. **Présences** : calendrier mensuel (office/remote/vacation/holiday, un upsert par jour) + import depuis l'ancienne application. **Empreinte CO₂** : référentiel de facteurs d'émission versionné, modèles de trajet réutilisables, calcul + snapshot par jour de présence (tout dans le coffre chiffré). **Paramètres par profil** : table `profile_settings` typée (heure de départ par défaut semée à la création d'un jour travaillé, police des notes, mode d'affichage des cellules, configuration CO₂), éditable depuis l'écran Réglages — le thème et la langue restant au niveau de l'appareil.

**Phase 2 (à venir)** :

- **Synchronisation cloud Turso** (isolée dans `infrastructure/persistence/db.rs` ; arbitrer le compromis chiffrement-vs-sync de libSQL — voir [documentation/turso.md](documentation/turso.md)).
- Projections annuelles CO₂ (`workingDaysPerYear`) et référentiel multi-millésime (`factorYear`, aujourd'hui figé à l'année seedée). _(La config CO₂ de base — pays réseau, forçage radiatif, énergie bâtiment — est désormais réglable par profil, cf. `profile_settings`.)_
- Chiffrement authentifié at-rest (crate `turso`, AEGIS-256) quand il sera stable ([ADR 0004](documentation/adr/0004-report-migration-turso.md)), multi-comptes.
