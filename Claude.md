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

> 📚 Documentation détaillée : [documentation/auth.md](documentation/auth.md) (sécurité / authentification) et [documentation/turso.md](documentation/turso.md) (persistance libSQL / Turso).

---

## 2. Stack technique

| Couche      | Technologies                                                                 |
| ----------- | ---------------------------------------------------------------------------- |
| Frontend    | React 19, TypeScript, Vite 7, état via **React Context** (pas de lib externe) |
| Bridge      | `@tauri-apps/api` (core)                                                      |
| Backend     | Rust (édition 2021), Tauri v2                                                 |
| Persistance | **libSQL** (moteur de Turso), local + **chiffré au repos** (AES-256-CBC)      |
| Crypto      | **Argon2id** (hash + dérivation de clé), **XChaCha20-Poly1305** (envelope)   |
| Tests       | `cargo test` (Rust), **Vitest** + Testing Library (front)                    |
| Outillage   | **pnpm**, ESLint 9 (flat config), Prettier                                   |

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
    │   ├── entities/            #    user, account, session, profile
    │   ├── repositories/        #    traits AccountRepository, ProfileRepository
    │   ├── services/            #    ports : PasswordHasher, KeyService, TokenGenerator,
    │   │                        #            SessionStore, VaultManager, Clock
    │   └── error.rs             #    DomainError
    ├── application/             # ── Application (dépend du Domain)
    │   ├── use_cases/           #    register_account, login, logout, check_session, account_exists,
    │   │                        #    create_profile, list_profiles, update_profile, delete_profile, set_active_profile
    │   └── dto/                 #    UserDto, LoginResultDto, SessionStatusDto, ProfileDto, ProfilesDto (serde camelCase)
    ├── infrastructure/          # ── Infrastructure (implémente les ports)
    │   ├── crypto/              #    Argon2PasswordHasher, Argon2KeyService, RandomTokenGenerator
    │   ├── persistence/         #    db (libsql), migrations, account_repository, profile_repository, vault
    │   ├── session/             #    InMemorySessionStore
    │   ├── clock.rs             #    SystemClock
    │   └── config.rs            #    AppConfig (chemins, params Argon2, politique session/lockout)
    └── presentation/            # ── Presentation (frontière IPC)
        ├── commands/            #    auth.rs, profile.rs (#[tauri::command] fines) + error.rs (AppError)
        └── state.rs             #    AppState injecté via .manage()
```

### Conventions backend

- `main.rs` reste minimal. Toute la configuration vit dans `lib.rs` via `run()`.
- **Composition root** dans `lib.rs` (`build_state`) : construire les impls d'infrastructure, les injecter dans les use cases (`Arc<dyn Trait>`), assembler l'`AppState`, puis `.manage(state)`.
- Une **commande Tauri** ne contient pas de logique métier : elle récupère le `State`, appelle le use case, et mappe `DomainError` → `AppError` sérialisable (`{ code, message }`, détails internes masqués).
- Les commandes sont `async` et renvoient un `Result`.
- Pas d'`unwrap()` / `panic!` dans les chemins d'exécution. `cargo fmt` + `cargo clippy -- -D warnings` doivent passer **sans warning**.
- Matériel sensible (mot de passe, KEK, DEK) effacé via `zeroize` / `Zeroizing`.

---

## 6. Structure du frontend React (`src/`)

Organisation **feature-first** : chaque feature porte ses propres couches `domain`/`data`/`presentation`. Le transverse vit dans `core/`.

```
src/
├── main.tsx                     # Entrée React — monte <AuthProvider>
├── App.tsx                      # Gate d'auth (Register / Login / Home)
├── core/                        # Transverse : config, erreurs, wrapper IPC
│   ├── ipc.ts                   #    encapsule invoke() — SEUL à importer @tauri-apps/api
│   ├── errors.ts                #    AppError + normalizeError
│   └── config.ts                #    constantes + noms de commandes
└── features/
    ├── auth/
    │   ├── domain/              #    entities, repositories (interfaces), use-cases (purs)
    │   ├── data/                #    dto, mappers, TauriAuthRepository (→ core/ipc)
    │   └── presentation/        #    AuthProvider (composition root front), hooks, écrans
    └── profile/                 #    Gestion des profils de présence (multi-profils)
        ├── domain/              #    Profile, ProfileRepository, use-cases purs
        ├── data/                #    dto, mapper, TauriProfileRepository (→ core/ipc)
        └── presentation/        #    ProfileProvider, useProfile, formulaire (Dialog),
                                 #    badge = menu compte (profils + langue + thème + déconnexion)
```

> Le header authentifié ne garde que le timer de session ; le **badge de profil**
> (tout à droite) ouvre le menu compte (changer/ajouter/éditer/supprimer un profil,
> langue, thème, déconnexion). `App.tsx` injecte `onSessionExpired={logout}` au
> `ProfileProvider` et `Home` injecte `onLogout` au badge — la feature `profile`
> ne dépend pas de `auth` (langue/thème viennent de `core`/`shared`).

### Conventions frontend

- **Seul `core/ipc.ts`** importe `@tauri-apps/api` (règle vérifiée par ESLint `no-restricted-imports`). Le reste passe par les repositories.
- Une **impl de repository** (couche data) appelle `invoke('cmd', {...})` puis mappe le DTO → entité du domaine.
- Les **composants** ne contiennent ni appel IPC, ni logique métier complexe ; ils consomment des données via des hooks.
- Les **hooks** orchestrent les use cases (ex : `useAuth`).
- **Alias de chemin** `@/...` (configuré dans `tsconfig.json` + `vite.config.ts`) — pas de `../../../`.
- Fichiers `kebab-case`, composants `PascalCase`, hooks `useCamelCase`. Typage strict, pas d'`any`.

---

## 7. Sécurité (résumé — détails dans [documentation/auth.md](documentation/auth.md))

- **Mot de passe** : haché en **Argon2id** (jamais stocké en clair, jamais envoyé au front).
- **Chiffrement au repos** : _envelope encryption_ à clé dérivée du mot de passe. Un DEK aléatoire chiffre le coffre `vault.db` ; le DEK est enveloppé (XChaCha20-Poly1305) par un KEK dérivé du mot de passe. Le coffre ne s'ouvre qu'après login.
- **Deux bases** : `keystore.db` (clair : hash + DEK enveloppé) et `vault.db` (chiffré).
- **Session** : 15 min **absolue**, en mémoire uniquement (jamais persistée) → mot de passe redemandé à chaque démarrage. **Anti-bruteforce** (verrouillage après échecs répétés).
- **Aucun secret** (token Turso, clés) commité ni exposé au frontend — ils restent côté Rust.

---

## 8. Règles importantes (à ne jamais enfreindre)

- **Ne jamais violer la règle de dépendance.** Le `domain` n'importe rien de `application`, `infrastructure`/`data`, `presentation`, React ou Tauri.
- **Ne jamais mettre de logique métier dans une commande Tauri ni dans un composant React.**
- **Sécurité Tauri** : minimum de permissions dans `capabilities/` (moindre privilège). Valider toutes les entrées du frontend.
- **Aucun secret** commité ni exposé au frontend. Les secrets restent côté Rust.
- **Seul `core/ipc.ts`** (front) parle à `@tauri-apps/api`.
- Épingler les versions de dépendances pour la production.

---

## 9. Workflow attendu

1. Implémenter en respectant les couches (domain → application → infra/presentation).
2. Avant de terminer une série de changements :
   - Rust : `cargo fmt`, `cargo clippy -- -D warnings`, `cargo test`.
   - Front : `pnpm typecheck`, `pnpm lint`, `pnpm test`.
3. Préférer les tests ciblés pendant le dev ; lancer la suite complète avant un commit.
4. Commits descriptifs (style Conventional Commits recommandé).

---

## 10. État du projet & feuille de route

**Implémenté (socle v1)** : architecture propre Rust + React, persistance libSQL locale chiffrée, première entité **User** + authentification complète (Argon2id, envelope encryption, session 15 min, anti-bruteforce). Entité **Profile** (profils de présence) : CRUD complet dans le coffre chiffré, gestion **multi-profils** (création, sélection du profil actif, édition, suppression) ; après login, l'accueil propose la création si aucun profil, sinon un badge (photo + nom) en haut à droite.

**Phase 2 (à venir)** :

- **Synchronisation cloud Turso** (isolée dans `infrastructure/persistence/db.rs` ; arbitrer le compromis chiffrement-vs-sync de libSQL — voir [documentation/turso.md](documentation/turso.md)).
- Entités **présence / mode de déplacement / CO₂** dans le coffre chiffré (rattachées à un profil).
- `change_password`, intégrité-au-repos, idle-timeout, multi-comptes.
