# Portage Tauri + React → Slint : journal de suivi

> **À lire en début de chaque session de portage.** Ce fichier fait foi pour l'état
> d'avancement. Le plan complet (contexte, choix de Slint, chiffrage) a été validé
> le 2026-11-08.

## Pourquoi

La RAM consommée par la version Tauri vient presque entièrement de WebKitGTK
(`WebKitWebProcess` + `NetworkProcess`, soit ≈150 à 300 Mo). Slint est une UI native
(Rust + langage `.slint`) qui offre plusieurs avantages :

- empreinte mémoire bien plus faible ;
- API stable en 1.x ;
- renderer logiciel, donc fonctionne sans GPU ;
- accessibilité via AccessKit.

GPUI a été écarté : il est encore pré-1.0, exige un GPU et son écosystème est trop
fragile. Licence retenue : Slint *Royalty-free 2.0*. Elle oblige à afficher
**`AboutSlint` dans un écran « À propos »** accessible depuis le menu.

## Organisation

- **Branche longue `feat/slint`.** `main` reste en Tauri jusqu'à la bascule (S13).
  En début de session : `git switch feat/slint && git merge main` si `main` a bougé.
- **Archive :** `legacy/tauri-app/` contient l'app Tauri + React telle que
  `v1.1.0-tauri` (tag). Elle est **figée** et autonome : son propre `Cargo.lock`,
  exclue du workspace, sans dépendance vers `crates/core`. Pour la compiler :
  `cd legacy/tauri-app && pnpm install && pnpm tauri build`. Seule modification :
  le chemin `include_str!` vers `LICENCE`, resté à la racine.
- **Workspace Cargo** (racine) :
  - `crates/core` → `basic-presence-core` : domain, application et infrastructure,
    plus `services` (composition) et `error` (`AppError`) ;
  - `crates/desktop` → binaire Slint `basic-presence` (à partir de S3).
- **Données partagées** avec la version Tauri. Le binaire Slint doit utiliser :
  - l'identifiant `com.godef.basic-presence`, pour pointer sur le même
    `app_data_dir` (résolu via la crate `dirs`) ;
  - les mêmes entrées du trousseau OS ;
  - le nom de paquet `basic-presence`.
- **Appels backend depuis l'UI :** le runtime tokio multi-thread exécute le use case,
  puis le résultat revient au thread UI via `slint::invoke_from_event_loop`. Ne jamais
  appeler `block_on` sur le thread UI.

## Fin de session (obligatoire)

```bash
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
cargo deny check
```

Ensuite : cocher la session ci-dessous, noter les décisions et écarts, puis faire
un commit sur `feat/slint`.

## Sessions

- [x] **S1 — Archivage, branche et workspace** (2026-11-08)
- [ ] **S2 — Logique TS → core** :
  - `stats/domain/aggregate.ts`, `profile-transfer/.../bundle.ts`,
    `worked-examples.ts`, `commuteToTrips`, `duration-format.ts`,
    `donut-geometry.ts`, `build-export-labels.ts` ;
  - totaux mensuels (`routes/_authenticated.tsx`) ;
  - use case `copy_day` (transactionnel) ;
  - `MarkdownRenderer` → blocs (`MdBlock`), en retirant ammonia.
- [ ] **S3 — Socle Slint + login → go/no-go** :
  - `slint-build`, bridge tokio, thème, i18n `.po` + test de parité, navigation ;
  - **mesure de la RAM (PSS)** legacy vs Slint, VM sans GPU, touches mortes.
- [ ] **S4 — Coquille applicative** :
  - composants partagés, header, session (compte à rebours + inactivité 5 min) ;
  - avis de sécurité, mot de passe, profils ;
  - écran À propos (`AboutSlint`).
- [ ] **S5 — Calendrier de présence** : dialog jour en 2 étapes, copier/coller,
  pastilles du mois.
- [ ] **S6 — Trajets et réglages** : préférences locales dans un fichier de config
  du data dir.
- [ ] **S7 — Heures de travail** : presets, sliders, donut, sélecteur d'heure,
  autosave.
- [ ] **S8 — Note du jour, licence, méthodologie** : `MarkdownView`, `TextEdit`,
  liens `https` via `open`.
- [ ] **S9 — Statistiques** : `BarChart` maison (zoom, moyenne, tooltip), donut,
  filtres.
- [ ] **S10 — Export .ods et transferts de bundle** : `rfd` ; re-porter
  `sanitize_file_name` / `with_extension` et leurs 3 tests
  (`legacy/.../presentation/file_dialogs.rs`).
- [ ] **S11 — Packaging et CI de release** :
  - `cargo-packager`, rpm, PKGBUILD ;
  - réécrire puis réactiver `release.yml` (désactivé par `if: false` sur `detect`).
- [ ] **S12 — Updater** :
  - minisign + `latest.json` compatibles ;
  - re-porter `is_package_managed` et son test (`legacy/.../commands/install.rs`) ;
  - test de transition Tauri → Slint.
- [ ] **S13 — Recette et bascule** : parité, mesure finale de la RAM, documentation,
  ADR, PR vers `main`, release 2.0.0.

## Notes et décisions

### S1

- `AppState` est renommé **`Services`** (`crates/core/src/services.rs`) et
  `build_state` devient **`build_services`**. Ajouts :
  - `open_services(data_dir)` : prépare le dossier en 0700, lit la clé d'appareil
    dans le trousseau, puis appelle `build_services` ;
  - `Services::lock()` : révoque les sessions et ferme le vault. Elle remplace
    `lock_vault` de Tauri.
- `PendingImports` est supprimé du core : en natif, l'UI conserve le `PathBuf`.
- `AppError` (`crates/core/src/error.rs`) dérive maintenant `Clone` et `PartialEq` ;
  `AppError::new` devient public.
- Tests d'intégration : `tauri::async_runtime::*` remplacé par un `block_on` local
  (runtime tokio multi-thread) et par `tokio::spawn`. Résultat : 135 tests au vert
  contre 140 en legacy. Les 5 tests manquants sont tous en `presentation/` :
  - isolation (1) : abandonné ;
  - `file_dialogs` (3) : à re-porter en S10 ;
  - `install` (1) : à re-porter en S12.
- Les modules du core sont désormais publics, ce qui déclenche le lint clippy
  `new_without_default`. Corrigé par `#[derive(Default)]` sur
  `JsonProfileBundleCodec`.
- `deny.toml` et `.cargo/audit.toml` sont copiés à la racine, sans
  `RUSTSEC-2024-0370` qui n'est plus atteint. À revoir en S12–S13 : les exceptions
  zip, zbus et quick-xml dont la justification cite des crates Tauri.
- CI : le job `backend` porte sur le workspace. Les jobs `legacy-*` ne tournent
  qu'en `workflow_dispatch`.
