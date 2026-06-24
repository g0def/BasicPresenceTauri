# Lot 4 — Consolidation individu → entreprise (multi-employés, par poste/site/entité)

> **Statut :** À faire  ·  **Priorité :** P1  ·  **Effort :** moyen  ·  **Dépend de :** Lot 1 (mapping Scope GHG / poste BEGES), Lot 2 (restitution tCO2e), et s'appuie sur la feature `profile-transfer` (import/validation de bundles, commit `0bd3220`)
> **Gap couvert :** (d) Pas de consolidation individu → entreprise : aucun total par poste, par site, ni par entité (SIREN/SIRET).
> **Débloque :** BEGES (déclaration par personne morale / SIREN, restitution par poste réglementaire, ch.9) · CSRD / ESRS E1-6 (consolidation au périmètre de l'entité déclarante, ventilation par site possible) · GHG Protocol Corporate Standard (frontière organisationnelle, agrégation des sources Scope 3 cat.7).

---

> ⚠️ **Numérotation de migration — lire avant de coder.** Ce document écrit `0012` en supposant un dépôt à `0011`, mais **ce lot ne peut pas garder ce numéro** : il entre en collision avec le Lot 1. Migration **réservée par le plan : `0013_add_consolidation.sql`** (cf. [README §5 — Numérotation des migrations](./README.md#numérotation-des-migrations)). **À l'exécution :** `ls src-tauri/migrations/vault/ | sort | tail -1` → prendre le numéro libre suivant, renommer le fichier `.sql`, corriger les références `00XX` de ce document, et aligner le champ `version:` dans `migrations.rs`.

## 1. Contexte & objectif

Aujourd'hui BasicPresence est strictement **mono-profil** : un profil = une personne, et l'export (ODS comme bundle JSON) ne produit jamais que les données d'un individu. Or BEGES et CSRD raisonnent au niveau d'une **entité juridique** (identifiée par son SIREN) et de ses **établissements / sites** (SIRET), avec une restitution **par poste réglementaire** (déplacements domicile-travail = poste 3.3 du BEGES, Scope 3 catégorie 7 du GHG Protocol) et **en tCO2e**.

Ce lot conçoit un **module d'agrégation multi-employés** qui ne change pas le modèle mono-profil : la consolidation se fait par **import et somme de plusieurs bundles** de profils (le format de transfert existant), puis production d'un **rapport de consolidation** (totaux par poste, par site, par entité, pour une **année de reporting** donnée), exportable en ODS. Il ajoute les champs d'identité d'entité/site (SIREN, SIRET) et gère l'**échantillonnage** (enquête représentative, admise par BEGES pour les grandes organisations) avec extrapolation et **anonymisation** des données nominatives. Résultat attendu : un déclarant peut produire un tableau « poste 3.3 = X tCO2e pour l'entité SIREN Y, année N », auditable et reproductible.

## 2. État actuel du code (point de départ)

Faits précis tirés de la cartographie et du code lu :

- **Profil = conteneur mono-utilisateur, sans identité d'entité structurée.**
  - `src-tauri/src/domain/entities/profile.rs:4-12` : `Profile { id, first_name, last_name, enterprise, poste: Option<String>, created_at, updated_at }`. `enterprise` est une **simple chaîne libre** (ex. « Acme Corp »), `poste` un intitulé optionnel non structuré.
  - `src-tauri/migrations/vault/0002_add_profiles.sql:3-11` : table `profile (id, first_name, last_name, enterprise, poste, created_at, updated_at)`. **Aucune** colonne SIREN, SIRET, code site, ni table `site`/`organisation`.
- **Présences et trajets sont per-profil, snapshots figés (R9).**
  - `presence` (`0003_add_presence.sql:4-13`) : `UNIQUE(profile_id, day)`, `co2_kg REAL NULL`, `is_estimated`, `type ∈ {office, remote, vacation, holiday}`.
  - `presence_trip` (`0004_add_co2.sql:73-86`) : `co2_kg`, `is_estimated`, `factor_year`. Le `co2_kg` est figé à l'encodage avec son `factor_year` (R9).
- **L'export ODS est strictement mono-profil et sans agrégation.**
  - `src-tauri/src/application/use_cases/export_profile_data.rs:31-81` : `ExportProfileDataUseCase::execute(profile_id, …)` charge **un seul** profil (`self.presences.list_by_profile(profile_id)`), produit `ExportData { days: Vec<DayExport> }` ligne par ligne, **aucun sous-total**, **aucun pivot** par poste/site/entité.
  - `src-tauri/src/presentation/commands/export.rs:12-25` : commande `export_profile_data` session-gated, déléguant à ce use case.
- **La feature profile-transfer fournit déjà l'import multi-bundle et sa validation (commit `0bd3220`).**
  - `src-tauri/src/application/use_cases/import_profile_bundle.rs` : `ImportProfileBundleUseCase::execute(path, selection, target, conflict_strategy)`. La passe de validation (`prepare_days`, `validate_co2`, `validate_day`, `prepare_commutes`…) construit les objets domaine **avant** toute écriture ; un fichier corrompu avorte proprement (`DomainError::Validation`).
  - `src-tauri/src/infrastructure/transfer/json_bundle_codec.rs:69-109` : `BundleFile { format: "basic-presence-profile", version, exported_at, app, profile: FileProfile{first_name,last_name,enterprise,poste}, settings, task_presets, commutes, days }`. **Le bundle ne transporte ni SIREN, ni SIRET, ni année de reporting.**
  - `src-tauri/src/application/dto/profile_bundle_dto.rs` : `BundleManifestDto` (inspection avant import), `BundleImportTargetDto { kind: "new"|"existing", … }`, `BundleImportSummaryDto`.
  - `SUPPORTED_BUNDLE_VERSION` est défini dans `src-tauri/src/domain/services/profile_bundle.rs` ; un fichier de version supérieure est rejeté (`import_profile_bundle.rs:108-113`).
- **Unité : kgCO2e partout.** Aucune restitution tCO2e (gap c, traité par le Lot 2 dont ce lot dépend).
- **Aucun mapping poste BEGES / Scope GHG** sur les facteurs (gap a, traité par le Lot 1 dont ce lot dépend). Le champ `emission_factor.scope` (`0011_add_co2_2026.sql:30-56`) désigne le **périmètre cycle de vie** (well-to-wheel), pas le Scope GHG.
- **Migrations** : ordre strict, dernière connue `0011_add_co2_2026.sql` ; registre dans `src-tauri/src/infrastructure/persistence/migrations.rs:22-67` (`VAULT_MIGRATIONS`, ajouter une entrée `version: 12`). Application idempotente via `_migrations` (`run()`).

**Conséquence :** rien dans le code ne permet aujourd'hui de répondre à « quel est le total poste 3.3 en tCO2e pour l'entité SIREN 552081317, établissement SIRET …00024, année 2025 ? ». C'est l'objet de ce lot.

## 3. Travail à réaliser

Tâches ordonnées. Le principe directeur : **ne pas casser le mono-profil**. La consolidation est un **agrégateur en lecture** alimenté par l'import de bundles ; chaque bundle reste un profil individuel importable normalement.

- **T1 — Migration `0012` : identité d'entité/site + table de consolidation.**
  Créer `src-tauri/migrations/vault/0012_add_consolidation.sql` (DDL détaillé §4). Trois changements :
  1. `ALTER TABLE profile` : ajouter `siren TEXT`, `siret TEXT` (établissement de rattachement), `site_label TEXT` (libellé lisible du site) — tous **nullable**, rétro-compatibles.
  2. `CREATE TABLE consolidation_run` : un run de consolidation (entité, année de reporting, méthode totale vs échantillon, paramètres).
  3. `CREATE TABLE consolidation_member` : la trace, par run, de chaque bundle/profil agrégé (identité éventuellement anonymisée, totaux figés par poste).
  Enregistrer la migration dans `migrations.rs` (`Migration { version: 12, sql: include_str!("../../../migrations/vault/0012_add_consolidation.sql") }`).

- **T2 — Domaine : entités de consolidation.**
  Créer `src-tauri/src/domain/entities/consolidation.rs` avec :
  - `enum SamplingMethod { Census, Sample }` (recensement complet vs enquête échantillon).
  - `struct ConsolidationRun { id, siren: Option<String>, entity_label: String, reporting_year: i32, method: SamplingMethod, headcount_total: Option<i64>, sampled_count: i64, created_at }`.
  - `struct ConsolidationMember { id, run_id, source_label: String, anonymized: bool, days_count: i64, working_days_per_year: i64, factor_year: i32, siret: Option<String>, site_label: Option<String> }`.
  - `struct PostTotal { ghg_scope: String, beges_post: String, total_kg: f64, total_t: f64, day_count: i64, member_count: i64, is_estimated_share: f64 }` — une ligne de pivot par poste (clé = `(ghg_scope, beges_post)` issus du Lot 1).
  - `struct SiteTotal { siret: Option<String>, site_label: String, total_kg: f64, total_t: f64, member_count: i64 }`.
  - `struct ConsolidationReport { run: ConsolidationRun, by_post: Vec<PostTotal>, by_site: Vec<SiteTotal>, grand_total_kg: f64, grand_total_t: f64, extrapolation_factor: f64, sampling_note: String }`.

- **T3 — Domaine : service pur d'agrégation.**
  Créer `src-tauri/src/domain/services/consolidation_aggregator.rs` : fonction **pure, sans I/O** (comme `Co2Calculator`), qui prend une liste de profils agrégés (présences + trips résolus avec leur poste/scope du Lot 1) et les paramètres du run, et retourne un `ConsolidationReport`. Règles métier détaillées §5. Tests unitaires en bas de fichier (cf. convention `co2_calculator.rs`).

- **T4 — Application : use case d'inspection + de consolidation par import de bundles.**
  Créer `src-tauri/src/application/use_cases/consolidate_bundles.rs` :
  - `ConsolidateBundlesUseCase::inspect(paths: Vec<String>) -> Vec<ConsolidationSourceDto>` : pour chaque chemin, lit le bundle via `ProfileBundleCodec` (réutilise la validation/sanitization de `0bd3220`), renvoie identité (anonymisable), nombre de jours, `factor_year`, `working_days_per_year`, modes inconnus. **Aucune écriture.**
  - `ConsolidateBundlesUseCase::run(request: ConsolidateRequestDto) -> ConsolidationReport` : lit tous les bundles, filtre les jours par `reporting_year`, résout chaque trip vers son poste/scope (Lot 1) via `EmissionFactorRepository`, appelle `consolidation_aggregator`, applique l'extrapolation d'échantillon, et **optionnellement** persiste le run (`consolidation_run` + `consolidation_member`) pour audit/reproductibilité.
  - Réutiliser strictement la passe de validation existante : ne pas dupliquer `validate_co2`/`validate_day` ; importer les fonctions partagées ou factoriser un helper commun `validate_bundle_day` réutilisé par `import_profile_bundle.rs` ET ce use case.

- **T5 — Application : use case d'export du rapport consolidé.**
  Étendre l'export ODS. Ajouter une variante `export_consolidation` (nouvelle commande + nouveau use case `ExportConsolidationUseCase`) qui écrit un classeur dédié via `SpreadsheetExporter`, avec une feuille **« Consolidation »** (pivot par poste, en tCO2e), une feuille **« Sites »** (pivot par SIRET/site) et une feuille **« Membres »** (liste des sources agrégées, anonymisables). Réutiliser l'infrastructure `ods_writer.rs` (ajouter `build_consolidation_sheet`, `build_sites_sheet`, `build_members_sheet`). Labels traduits passés depuis le front (jamais de texte codé en dur côté Rust, cf. convention `ExportLabelsDto`).

- **T6 — DTOs (frontière IPC, camelCase).**
  Créer `src-tauri/src/application/dto/consolidation_dto.rs` :
  - `ConsolidationSourceDto`, `ConsolidateRequestDto { siren: Option<String>, entityLabel, reportingYear, method: "census"|"sample", headcountTotal: Option<i64>, anonymize: bool, sources: Vec<{ path, siret: Option<String>, siteLabel: Option<String> }> }`, `ConsolidationReportDto`, `PostTotalDto`, `SiteTotalDto`.
  - Mappers `From<…>` domaine ↔ DTO (testés, cf. convention mappers).

- **T7 — Présentation : commandes Tauri.**
  Créer `src-tauri/src/presentation/commands/consolidation.rs` : `inspect_consolidation_sources`, `run_consolidation`, `export_consolidation` — toutes **session-gated** (`state.require_session.execute()?`), déléguant aux use cases, mappant `DomainError → AppError`. Les enregistrer dans `invoke_handler` (`src-tauri/src/lib.rs`) et injecter les use cases dans `build_state` (DI via `Arc<dyn Trait>`).

- **T8 — Bundle : transporter SIREN/SIRET (montée de version).**
  Étendre `BundleFile`/`FileProfile` (`json_bundle_codec.rs`) avec `siren: Option<String>`, `siret: Option<String>`, `site_label: Option<String>` (`#[serde(default, skip_serializing_if = "Option::is_none")]`) et **incrémenter `SUPPORTED_BUNDLE_VERSION` de 1** (`profile_bundle.rs`). Les bundles v1 restent lisibles (champs absents ⇒ `None`). Propager ces champs dans `create_target_profile` (`import_profile_bundle.rs:321-362`) et dans l'export de bundle (`export_profile_bundle.rs`).

- **T9 — Frontend : feature `consolidation`.**
  Créer `src/features/consolidation/` (domain/data/presentation) : entités TS (`ConsolidationReport`, `PostTotal`, `SiteTotal`, `ConsolidationSource`), repository Tauri (`tauri-consolidation.repository.ts` via `core/ipc.ts`), hook `use-consolidation.ts`, page `consolidation-page.tsx` (sélection multi-fichiers, saisie SIREN/entité/année, choix méthode recensement/échantillon + effectif total, toggle anonymisation, affichage du pivot par poste en tCO2e, bouton export ODS). Ajouter les clés i18n `consolidation.*` (`fr` **et** `en`). Ajouter la route file-based dans `src/routes/` (sous `_authenticated/`).

- **T10 — Documentation.**
  Mettre à jour `documentation/calcul-impact-co2.md` : nouvelle section « Consolidation entité (BEGES/CSRD) » décrivant l'agrégation par poste/site/SIREN, l'extrapolation d'échantillon, l'anonymisation, et l'année de reporting. Documenter que la consolidation **n'altère jamais** les `co2_kg` figés (R9) : elle ne fait que les **lire et sommer**.

## 4. Modèle de données / migrations

Nouvelle migration **`src-tauri/migrations/vault/0012_add_consolidation.sql`** (numérotation croissante après `0011`). Toutes les colonnes ajoutées à `profile` sont **nullable** (rétro-compat : profils existants inchangés). À enregistrer comme `version: 12` dans `migrations.rs`.

```sql
-- Lot 4 — Consolidation individu → entreprise (gap d).
-- Ajoute l'identité d'entité/site sur le profil (nullable, rétro-compatible) et
-- deux tables traçant un run de consolidation et ses membres. AUCUNE modification
-- des co2_kg figés (R9) : la consolidation lit et somme, jamais ne réécrit.

-- 1) Identité d'entité juridique / établissement sur le profil. Nullable :
--    un profil personnel sans entité reste parfaitement valide.
ALTER TABLE profile ADD COLUMN siren      TEXT;   -- entité juridique (9 chiffres FR) ; libre pour BE
ALTER TABLE profile ADD COLUMN siret      TEXT;   -- établissement de rattachement (14 chiffres FR)
ALTER TABLE profile ADD COLUMN site_label TEXT;   -- libellé lisible du site (ex. « Siège Lyon »)

-- 2) Un run de consolidation : entité déclarante + année de reporting + méthode.
CREATE TABLE IF NOT EXISTS consolidation_run (
    id              TEXT PRIMARY KEY,              -- UUID v7
    siren           TEXT,                          -- entité (nullable : BE/structure informelle)
    entity_label    TEXT NOT NULL,                 -- raison sociale / libellé lisible
    reporting_year  INTEGER NOT NULL,              -- année de reporting (ex. 2025)
    method          TEXT NOT NULL DEFAULT 'census' -- recensement complet vs enquête échantillon
                        CHECK (method IN ('census','sample')),
    headcount_total INTEGER,                       -- effectif total (requis si method='sample')
    sampled_count   INTEGER NOT NULL DEFAULT 0     -- nb de membres effectivement agrégés
                        CHECK (sampled_count >= 0),
    anonymized      INTEGER NOT NULL DEFAULT 0      -- les membres ont-ils été anonymisés
                        CHECK (anonymized IN (0,1)),
    created_at      INTEGER NOT NULL               -- epoch ms
);

CREATE INDEX IF NOT EXISTS idx_consolidation_run_year ON consolidation_run(reporting_year);

-- 3) Chaque source/profil agrégé dans un run. Les totaux par poste sont figés
--    ici au moment du run (snapshot d'audit), exprimés en kgCO2e (conversion
--    tCO2e à l'affichage/export, cf. Lot 2). beges_post / ghg_scope viennent du
--    mapping du Lot 1.
CREATE TABLE IF NOT EXISTS consolidation_member (
    id                    TEXT PRIMARY KEY,        -- UUID v7
    run_id                TEXT NOT NULL,
    source_label          TEXT NOT NULL,           -- identité ou pseudonyme (« Employé 07 »)
    anonymized            INTEGER NOT NULL DEFAULT 0 CHECK (anonymized IN (0,1)),
    siret                 TEXT,                    -- établissement du membre
    site_label            TEXT,
    days_count            INTEGER NOT NULL DEFAULT 0 CHECK (days_count >= 0),
    working_days_per_year INTEGER NOT NULL DEFAULT 220 CHECK (working_days_per_year >= 0),
    factor_year           INTEGER NOT NULL,        -- millésime des facteurs des jours agrégés
    FOREIGN KEY (run_id) REFERENCES consolidation_run(id) ON DELETE CASCADE
);

CREATE INDEX IF NOT EXISTS idx_consolidation_member_run ON consolidation_member(run_id);

-- 4) Total figé par membre et par poste (pivot d'audit). PK composite garantit
--    une ligne unique (membre, scope GHG, poste BEGES).
CREATE TABLE IF NOT EXISTS consolidation_member_post (
    member_id  TEXT NOT NULL,
    ghg_scope  TEXT NOT NULL,                      -- '1' | '2' | '3' (Lot 1)
    beges_post TEXT NOT NULL,                      -- ex. '3.3' (Lot 1)
    total_kg   REAL NOT NULL DEFAULT 0,            -- somme figée en kgCO2e (R9)
    day_count  INTEGER NOT NULL DEFAULT 0,
    estimated_kg REAL NOT NULL DEFAULT 0,          -- part is_estimated (incertitude)
    PRIMARY KEY (member_id, ghg_scope, beges_post),
    FOREIGN KEY (member_id) REFERENCES consolidation_member(id) ON DELETE CASCADE
);
```

> **Note SIREN/SIRET et cadre belge :** les formats FR (SIREN 9 chiffres, SIRET 14) ne sont **pas** contraints par `CHECK` car le cadre belge (gap g, autre lot) emploie le **numéro d'entreprise BCE** (10 chiffres). La validation de format reste **applicative** et **tolérante** ; on stocke la chaîne normalisée (sans espaces/tirets) pour fiabiliser le dédoublonnage par entité.

## 5. Spécification détaillée

### 5.1 Pipeline de consolidation (vue d'ensemble)

```
Bundles JSON (1 par employé)
   └─ inspect()  ──► validation/sanitization (réutilise 0bd3220)
   └─ run()
        1. lire chaque bundle (ProfileBundleCodec.read → validation up-front)
        2. filtrer les jours par reporting_year (année civile de `day`)
        3. ne garder que office/remote (vacation/holiday → 0, R7)
        4. pour chaque trip : résoudre (mode_id, factor_year) → (ghg_scope, beges_post)  [Lot 1]
        5. sommer co2_kg FIGÉ par (membre, scope, poste)        [R9 : on LIT, on ne recalcule pas]
        6. pivoter : by_post, by_site, grand_total
        7. si method='sample' : extrapoler (×) avec extrapolation_factor
        8. convertir kg → t pour la restitution (×0,001)        [Lot 2]
        9. (option) persister consolidation_run / _member / _member_post
```

### 5.2 Résolution de l'année de reporting

`day` est un epoch ms à minuit **UTC** (`presence`). L'année civile d'un jour = année UTC de `day`. Un jour appartient au run si `year_utc(day) == reporting_year`. **Tout** est en UTC, aucune conversion de fuseau (convention du projet).

### 5.3 Mapping mode → poste / scope (dépendance Lot 1)

La consolidation **ne définit pas** le mapping : elle le **consomme**. Les colonnes `ghg_scope` / `beges_post` proviennent du Lot 1 (sur `emission_factor` ou table de mapping). Référence attendue pour ce lot :

| Catégorie (`EmissionCategory`) | Modes (`mode_id`) | Scope GHG | Poste BEGES |
|---|---|---|---|
| Car | `car_petrol`, `car_diesel`, `car_average`, `car_phev`, `car_ev`, `taxi` | 3 | 3.3 (déplacements domicile-travail) |
| Active | `walk`, `bike`, `ebike`, `escooter`, `scooter_elec` | 3 | 3.3 |
| PublicTransport | `public_transport`, `bus`, `coach`, `metro_tram` | 3 | 3.3 |
| Rail | `train_sncb`, `train_ter`, `train_hs_fr`, `train_eurostar`, `train_thalys` | 3 | 3.3 |
| Air | `plane_domestic`, `plane_short`, `plane_medium`, `plane_long` | 3 | 3.3 |
| Building (`office_day`) | énergie bâtiment bureau | 2 (location/market, gap h) | 2.x (énergie) |
| Building (`home_day`) | énergie domicile télétravail | 3 | cat.7 / poste dédié |

> Si le Lot 1 n'est pas encore livré au moment d'implémenter, fournir un **mapping de repli statique** dans `consolidation_aggregator.rs` (constante documentée) et le remplacer par la lecture DB dès que le Lot 1 expose les colonnes. Le repli ne doit jamais silencieusement classer un mode inconnu : un mode sans mapping → poste `« non classé »` explicite, comptabilisé à part et signalé dans le rapport.

### 5.4 Échantillonnage et extrapolation (grandes organisations)

BEGES admet, pour les grandes structures, une **enquête représentative** plutôt qu'un recensement exhaustif.

- `method = "census"` (recensement) : `extrapolation_factor = 1,0`. Le total agrégé est le total déclaré.
- `method = "sample"` (échantillon) : l'utilisateur fournit `headcount_total` (effectif total de l'entité). `sampled_count` = nombre de membres effectivement agrégés (= nb de bundles valides). 

  `extrapolation_factor = headcount_total / sampled_count` (avec garde : `sampled_count ≥ 1`, sinon erreur de validation). Chaque total par poste est multiplié par `extrapolation_factor`. Le rapport affiche **les deux** : total échantillon (brut) ET total extrapolé, plus une `sampling_note` rappelant la méthode.

Edge cases :
- `method="sample"` sans `headcount_total` ou `headcount_total < sampled_count` → `DomainError::Validation`.
- `sampled_count = 0` (aucun bundle valide) → `DomainError::Validation("aucune source valide à consolider")`.

### 5.5 Anonymisation

- `anonymize = true` : `source_label` du membre devient un **pseudonyme déterministe** (« Membre 01 », « Membre 02 », … dans l'ordre d'import). Les noms/prénoms du bundle **ne sont jamais persistés** dans `consolidation_member` ni écrits dans l'export. Le SIRET/site_label restent (ce sont des données d'entité, pas nominatives).
- `anonymize = false` : `source_label = "Prénom Nom"` (depuis `FileProfile`).
- Le flag est figé sur le run (`consolidation_run.anonymized`) et propagé à chaque membre. Pour CSRD/ESRS, l'anonymisation par défaut **recommandée** est `true` (donnée RH sensible).

### 5.6 Sommation et figeage (R9)

- La consolidation **somme des `co2_kg` déjà figés** (présences importées / `presence_trip`). Elle **ne ré-exécute jamais** `Co2Calculator`. Un jour encodé en 2025 (facteurs 2025, RF 1,9) garde sa valeur ; un jour 2026 garde la sienne. Le `factor_year` de chaque membre est tracé dans `consolidation_member.factor_year` (peut être hétérogène si l'entité mélange des millésimes — c'est signalé dans le rapport, jamais corrigé silencieusement).
- Conséquence : agréger des profils encodés sous des millésimes différents est **autorisé** mais **documenté** (colonne « millésime » dans la feuille Membres ; note si > 1 millésime distinct dans un run).

### 5.7 Conversion d'unité (dépendance Lot 2)

Tous les stockages restent en **kgCO2e**. La conversion `tCO2e = kgCO2e × 0,001` est appliquée **uniquement à la restitution** (rapport + export), en réutilisant l'helper du Lot 2. Arrondis recommandés : tCO2e à 3 décimales pour les totaux par poste, kgCO2e à 2 décimales pour le détail.

### 5.8 Formats de colonnes — feuille ODS « Consolidation » (pivot par poste)

| Colonne | Source | Exemple |
|---|---|---|
| Scope GHG | `PostTotal.ghg_scope` (Lot 1) | `3` |
| Poste BEGES | `PostTotal.beges_post` (Lot 1) | `3.3` |
| Total (tCO2e) | `total_t` (échantillon brut) | `12,480` |
| Total extrapolé (tCO2e) | `total_t × extrapolation_factor` | `124,800` |
| Total (kgCO2e) | `total_kg` | `12480,00` |
| Jours comptés | `day_count` | `4 380` |
| Membres | `member_count` | `35` |
| Part estimée | `is_estimated_share` (%) | `4,2 %` |

Feuille « Sites » : `SIRET | Site | Total (tCO2e) | Membres`. Feuille « Membres » : `Source (ou pseudonyme) | Site | Jours | Jours ouvrés/an | Millésime`.

## 6. Critères d'acceptation

- Quand on importe N bundles valides pour `reporting_year=2025` en mode `census`, alors le rapport produit un **total par poste 3.3** égal à la **somme exacte** des `co2_kg` figés (à l'arrondi près), converti en tCO2e, et le grand total = Σ des totaux par poste.
- Quand un bundle contient des jours d'années autres que `reporting_year`, alors ces jours sont **exclus** du total (filtre par année UTC).
- Quand `method="sample"` avec `headcount_total=350` et `sampled_count=35`, alors `extrapolation_factor=10,0` et le total extrapolé par poste = total échantillon × 10.
- Quand `method="sample"` sans `headcount_total` (ou `headcount_total < sampled_count`), alors l'appel échoue avec `DomainError::Validation` (aucune écriture).
- Quand `anonymize=true`, alors **aucun** nom/prénom n'apparaît dans `consolidation_member` ni dans l'export ODS ; les labels sont « Membre 01 … 0N ».
- Quand un bundle est corrompu (co2 négatif/NaN, `day` non aligné minuit UTC, mode vide), alors la consolidation **avorte avant toute persistance** avec une erreur de validation (réutilisation de la passe `0bd3220`).
- Quand les profils agrégés mélangent `factor_year` 2025 et 2026, alors le rapport **les agrège sans les recalculer** (R9) et **signale** la présence de plusieurs millésimes.
- Quand un `mode_id` n'a pas de mapping poste (Lot 1 ou repli), alors il est comptabilisé sous « non classé » et **listé** dans le rapport (jamais silencieusement rangé en 3.3).
- L'import d'un **bundle v1** (sans SIREN/SIRET) reste possible après la montée de `SUPPORTED_BUNDLE_VERSION` (champs ⇒ `None`).
- Toutes les commandes consolidation sont **session-gated** : un appel hors session renvoie l'erreur de session, sans toucher au vault.

## 7. Tests à écrire / mettre à jour

**Rust — unitaires du service pur** (`src-tauri/src/domain/services/consolidation_aggregator.rs`, `#[cfg(test)] mod tests`) :
- `consolidates_two_members_census_sums_by_post` : 2 membres, poste 3.3 = somme exacte.
- `excludes_days_outside_reporting_year` : jours 2024 ignorés pour `reporting_year=2025`.
- `sample_extrapolation_factor` : `headcount_total/sampled_count` appliqué, total brut + extrapolé corrects.
- `sample_requires_headcount` : `sample` sans `headcount_total` ⇒ erreur.
- `anonymize_strips_names` : labels pseudonymisés, aucun nom propre dans la sortie.
- `mixed_factor_years_not_recalculated` : deux membres millésimes différents ⇒ somme des valeurs figées, flag « multi-millésime ».
- `unmapped_mode_goes_to_unclassified` : mode sans poste ⇒ bucket « non classé », non agrégé en 3.3.
- `grand_total_equals_sum_of_posts` (miroir de l'esprit d'AC10 du `co2_calculator`).

**Rust — use case / validation** (`src-tauri/src/application/use_cases/consolidate_bundles.rs`) :
- `inspect_rejects_corrupted_bundle` : bundle co2 négatif ⇒ `Validation` (réutilise les helpers de `import_profile_bundle.rs`).
- `run_persists_run_and_members_when_requested` : vérifie l'écriture de `consolidation_run`/`_member`/`_member_post` (ports mockés, cf. style `integration_tests.rs`).

**Rust — codec bundle** (`src-tauri/src/infrastructure/transfer/json_bundle_codec.rs`) :
- `reads_v1_bundle_without_siren` : un bundle de version précédente se lit, `siren=None`.
- `roundtrips_siren_siret` : écriture/lecture conserve SIREN/SIRET/site_label.

**TS — vitest** (`src/features/consolidation/`) :
- mapper DTO → entité (`*.test.ts`) : `PostTotalDto → PostTotal`, conversion tCO2e affichée.
- hook `use-consolidation` avec `mockIPC` (succès + rejet, langue `fr`).
- rendu de `consolidation-page` : pivot par poste, toggle anonymisation, bouton export.

**Commandes pour lancer :**
```
cd src-tauri && cargo test consolidation
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings
pnpm test src/features/consolidation
pnpm typecheck && pnpm lint
```

## 8. Points d'attention & pièges

- **R9 — figeage des jours :** la consolidation **lit et somme**, jamais ne recalcule. Interdiction absolue de ré-exécuter `Co2Calculator` ou de réécrire `presence_trip.co2_kg`. Tracer `factor_year` par membre ; signaler (sans corriger) les runs multi-millésimes.
- **Dépendances de lot :** le mapping poste/scope vient du **Lot 1** et la conversion tCO2e du **Lot 2**. Prévoir un repli statique documenté pour le mapping si le Lot 1 n'est pas encore mergé, mais **ne jamais** classer un mode inconnu en 3.3 par défaut.
- **Réutilisation de la sanitization (`0bd3220`) :** ne pas dupliquer `validate_co2` / `validate_day` ; factoriser un helper partagé entre `import_profile_bundle.rs` et `consolidate_bundles.rs`. Un bundle corrompu doit avorter **avant** toute écriture.
- **Couches DDD :** service d'agrégation = **pur** (domain, zéro I/O, zéro type tiers) ; lecture des bundles et persistance = application/infrastructure ; commandes = présentation (session-gated, `DomainError→AppError`). DI via `Arc<dyn Trait>` dans `build_state` (`lib.rs`).
- **Montée de version du bundle :** incrémenter `SUPPORTED_BUNDLE_VERSION` **et** garder la lecture des versions antérieures (`#[serde(default)]` sur les nouveaux champs). Un fichier de version supérieure reste rejeté proprement (`import_profile_bundle.rs:108-113`).
- **i18n :** toutes les nouvelles clés `consolidation.*` doivent exister en **`fr` ET `en`** ; le backend ne code jamais de libellé (labels passés via DTO). Penser aux libellés de postes (3.3, etc.) et aux notes d'échantillon/anonymisation.
- **Anonymisation & RGPD :** par défaut anonymiser pour les exports d'entité ; ne jamais persister de nom propre quand `anonymize=true`. SIRET/site ne sont pas nominatifs et peuvent rester.
- **Format SIREN/SIRET :** validation applicative tolérante (pas de `CHECK` SQL) pour rester compatible avec le numéro d'entreprise belge (gap g). Normaliser (sans espaces/tirets) pour le dédoublonnage par entité ; attention aux faux doublons (casse, espacement) sur `enterprise` libre.
- **Mono-profil préservé :** ce lot **n'introduit pas** le multi-utilisateur natif. La consolidation reste un agrégateur de bundles importés — ne pas régresser le parcours profil individuel ni l'export mono-profil existant (`export_profile_data`).
- **Mouvance réglementaire :** la liste des postes BEGES et le périmètre CSRD évoluent (méthode BEGES v5, ESRS révisés). Garder le mapping **piloté par données** (Lot 1) plutôt que codé en dur, pour absorber les évolutions sans toucher l'agrégateur.
- **Performance / N+1 :** charger le référentiel de facteurs **une fois par `factor_year`** (réutiliser `factor_maps.rs`) plutôt qu'une requête par trip lors de la résolution du poste.

## 9. Références réglementaires

- **BEGES (Bilan d'Émissions de Gaz à Effet de Serre)** — Article L.229-25 du Code de l'environnement ; obligation par **personne morale** (≥ 500 salariés, ≥ 250 en outre-mer) ; restitution par **poste** réglementaire. Méthode et postes : guide ADEME / référentiel « méthode pour la réalisation des bilans d'émissions de GES » (Base Empreinte / Bilans GES — bilans-ges.ademe.fr). Le **poste 3.3 « Déplacements domicile-travail »** est la cible directe de BasicPresence.
- **BEGES — échantillonnage :** la méthode admet l'**enquête représentative** (échantillon extrapolé à l'effectif total) pour les postes diffus comme le domicile-travail dans les grandes organisations.
- **GHG Protocol — Corporate Standard & Corporate Value Chain (Scope 3) Standard** — frontière organisationnelle (control/equity), **Scope 3 catégorie 7 « Employee commuting »**, agrégation par entité déclarante (ghgprotocol.org).
- **CSRD / ESRS E1 (Climate change)** — datapoints E1-6 (émissions brutes Scope 1/2/3 et total), restitution en **tCO2e** au périmètre de l'entité consolidée ; ventilation possible par activité/site (EFRAG — ESRS E1).
- **Unités :** restitution en **tCO2e** (BEGES ch. 9 et ESRS E1-6) — assurée par le Lot 2, consommée ici.
- **Numéros d'identification :** SIREN (entité, 9 chiffres) / SIRET (établissement, 14 chiffres) — INSEE (référentiel SIRENE) ; cadre belge : numéro d'entreprise BCE (10 chiffres) — à articuler avec le gap g.
