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
- [x] **S2 — Logique TS → core** (2026-11-08)
- [x] **S3 — Socle Slint + login** (2026-11-09) — ⚠️ **go/no-go RAM reporté**
  (voir les notes S3). Prévu à l'origine :
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
- [ ] **S10 — Export .ods et transferts de bundle** :
  - `rfd` ;
  - re-porter `sanitize_file_name` / `with_extension` et leurs 3 tests
    (`legacy/.../presentation/file_dialogs.rs`) ;
  - porter `build-export-labels.ts` (libellés traduits → `ExportLabelsDto`, avec
    `Lang::decimal_separator`), reporté de S2 car il dépend de l'i18n.
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

### S2

- **Markdown** : le port `MarkdownRenderer` expose maintenant
  `render_blocks(&str) -> Vec<MdBlock>` (`domain/services/markdown.rs`) au lieu
  de `render_to_safe_html`.
  - Les blocs (titre, paragraphe, élément de liste avec puce / numéro / case
    à cocher, code coloré, règle, tableau) portent `indent` (profondeur de liste)
    et `quote_depth` (citations). Chaque `MdSpan` porte gras, italique, barré,
    code et lien.
  - Le HTML brut est rendu en **texte littéral**, jamais interprété. Seuls les
    liens `http`, `https` et `mailto` sont conservés (`is_safe_link`). Les images
    sont réduites à leur texte alternatif.
  - `DayNoteDto.html` devient `blocks`, et `GetLicenceUseCase::execute()` renvoie
    `Vec<MdBlock>`.
  - **ammonia est retiré** du core.
- **`copy_day`** (`application/use_cases/copy_day.rs`, `Services::copy_day`) :
  `execute(source_presence_id, target_day)` recopie le type, les trajets,
  l'heure de début et les tâches. Tout s'exécute dans **une seule transaction**,
  car les sous-use cases rejoignent la transaction ambiante. Les heures d'un
  ancien jour travaillé sont purgées par les triggers SQL de changement de type.
  Couvert par un test d'intégration.
- **`application::view`** (nouveau) : helpers d'affichage purs, testés.
  - `format` : `format_minutes`, `parse_clock_time`, `format_clock_time`,
    `format_co2`, et `Lang` avec ses séparateurs décimal et de milliers (espace
    fine insécable en FR, comme `Intl`).
  - `period_stats` :
    - `Granularity`, `bucket_presences` (buckets mois / semaine ISO comblés,
      `until`) ;
    - `period_key`, `breakdown_by_type`, `month_totals` (pastilles du header) ;
    - `civil_date` / `day_ms` (jour en ms UTC ↔ `NaiveDate`).
    - Les clés de mois valent maintenant `année*100 + mois`, au lieu du
      `Date.UTC` du TypeScript.
  - `donut` : `compute_donut_arcs`.
  - `bundle_selection` : `BundleCategory`, `BundleSelection` (`effective`,
    `from_manifest`, conversions vers les DTO), `present_categories`.
  - `methodology` : `factors_by_category`, `worked_examples`.
- `CommuteDto::to_trips()` (port de `commuteToTrips`), `ExportOptionsDto::default()`,
  `PresenceType::ALL` et `PresenceType::is_work()`.
- Reporté en S10 : `build-export-labels.ts`, qui dépend des traductions.
- Bilan : 171 tests au vert, contre 135 en S1.

### S3

- **Crate `crates/desktop`** (bibliothèque `basic_presence` + binaire `basic-presence`).
  Slint `=1.18.1` sans les features par défaut, pour ne pas tirer Qt. Il embarque
  le backend winit (Wayland, X11, Windows, macOS), les renderers femtovg et
  logiciel, et l'accessibilité.
  - **Renderer logiciel**, pour les postes sans GPU : `SLINT_BACKEND=winit-software`.
  - **Dossier de données** : `dirs::data_dir()/com.godef.basic-presence`, le même que
    l'`app_data_dir` de Tauri.
  - **`BASIC_PRESENCE_DATA_DIR`** remplace ce dossier : à utiliser pour les tests et
    les mesures, pour ne jamais toucher au vrai coffre. Avec une clé d'appareil déjà
    présente dans le trousseau, celle-ci est relue sans jamais être réécrite.
- **Pont tokio** (`src/bridge.rs`) : `Bridge::run(work, done)` exécute le use case
  sur le runtime (2 workers), puis `done` sur le thread UI via
  `invoke_from_event_loop`. `lock_blocking()` est appelé à la sortie, une fois la
  boucle d'événements terminée.
- **Erreurs** (`src/errors.rs`) : `UiError { code, arg }`, avec les mêmes codes que
  l'ancien `AppError`. Elles sont traduites côté Slint par `AppState.error-text`.
  Les erreurs internes partent sur stderr, jamais dans l'UI. Un échec au démarrage
  affiche un écran dédié (`startup-error.slint`) au lieu d'un dialogue natif.
- **i18n** : source de vérité dans `crates/desktop/i18n/{fr,en}.json` (les JSON
  i18next, avec de nouvelles clés `errors.codes.*`, `startup.*` et `home.*`).
  - **Génération** : `build.rs` produit les `.po` dans `OUT_DIR` ; les msgids sont
    les clés pointées, sans contexte de traduction
    (`DefaultTranslationContext::None`).
  - **Variables** : `{{x}}` devient `{0}`, `{1}`… numérotés dans l'ordre
    d'apparition dans la chaîne française.
  - **Pluriels** : `key_one` / `key_other` donnent
    `@tr("key_one" | "key_other" % n)`.
  - **Contrôles à la compilation** : le build échoue si `fr` et `en` n'ont pas les
    mêmes clés, ou si un `@tr("…")` d'un `.slint` cite une clé inconnue.
  - **Langue** : celle du système (`LC_ALL`, `LC_MESSAGES`, `LANG`), changeable à
    chaud. Sa persistance est prévue en S6.
- **Thème** : `ui/theme.slint` reprend les tokens oklch de l'ancien `index.css`
  convertis en hex (script de conversion : sRGB via OKLab).
  - Au démarrage, il suit le schéma de couleurs du système.
  - **Piège** : il faut **figer `Palette.color-scheme`** sur `Theme.dark`. Sinon les
    widgets standard passent seuls en sombre quand le système annonce tardivement
    son schéma, et deviennent invisibles sur une page claire.
- **Composants** : `Button` (variantes, clavier, accessibilité), `Card`, `Field`
  (un `LineEdit` du style Fluent, avec son libellé).
  - **Écrans** : connexion / inscription, accueil provisoire, erreur de démarrage.
  - **Mot de passe** : le champ est vidé dès la soumission. Côté Rust, il est
    enveloppé dans `Zeroizing`. **Limite** : la copie interne `SharedString` de
    Slint n'est pas effacée.
- **Test d'UI headless** (`crates/desktop/tests/ui_flow.rs`) : plateforme Slint
  minimale avec renderer logiciel et file d'événements pompée à la main, branchée
  sur les vrais services (coffre temporaire, clé fixe).
  - **Parcours couvert** : inscription, refus avec erreur traduite, bascule FR → EN
    et thème sombre, connexion, accueil, déconnexion.
  - **Captures PNG** : `BP_SNAPSHOT_DIR=<dir> cargo test -p basic-presence --test ui_flow`.
    **À réutiliser pour vérifier chaque écran**, sans piloter le bureau de
    l'utilisateur.
- **CI** : installation de `libfontconfig1-dev`, `libxkbcommon-dev` et `pkg-config`
  pour compiler Slint.
- **Mesure de la RAM : reportée, à la demande de l'utilisateur.** C'est une
  condition du go/no-go, à faire avant d'aller loin dans S4–S5.
  - **Outils** :
    - binaires release : `cargo build --release -p basic-presence` et
      `cd legacy/tauri-app && pnpm tauri build --no-bundle` ;
    - lancement sur des données jetables : `XDG_DATA_HOME=<tmp>` pour Tauri,
      `BASIC_PRESENCE_DATA_DIR=<tmp>` pour Slint ;
    - mesure : somme du `Pss` de `/proc/<pid>/smaps_rollup` sur tout l'arbre de
      processus, avec les processus WebKit pour Tauri.
  - **Premier point, isolé et non concluant** (écran de connexion, KDE Wayland,
    release, 8 s) :
    - Slint en femtovg : ≈ 120 Mo de PSS, sans doute surtout le pilote OpenGL.
    - Tauri : le seul processus principal pèse déjà ≈ 100 Mo de PSS, auxquels
      s'ajoutent WebKitNetworkProcess et WebKitWebProcess (RSS ≈ 175 et ≈ 300 Mo).
  - **À comparer aussi** : le renderer logiciel (`SLINT_BACKEND=winit-software`),
    et le renderer Skia si femtovg reste lourd.
- Tests manuels encore à faire par l'utilisateur : VM sans GPU, touches mortes
  FR, lecteur d'écran (Orca).
- **cargo-deny** :
  - **Licences autorisées** : `BSL-1.0` (clipboard-win, error-code, via winit) et
    `LicenseRef-Slint-Royalty-free-2.0` (Slint, avec l'attribution `AboutSlint`
    exigée en S4).
  - **Exceptions d'audit** :
    - `RUSTSEC-2026-0441` (pin-utils) : uniquement à la compilation ;
    - `RUSTSEC-2026-0192` (ttf-parser) : décorations de fenêtre winit, polices de
      confiance.
