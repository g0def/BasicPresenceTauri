# Lot 9 — Documentation d'audit & incertitude

> **Statut :** À faire  ·  **Priorité :** P3  ·  **Effort :** moyen  ·  **Dépend de :** Lot 1 (mapping Scope GHG / poste BEGES / `lifecycle_boundary`), Lot 3 (export auditable : facteur, unité, source, méthode par trajet)
> **Gap couvert :** (i) Documentation d'audit incomplète — journal des facteurs/hypothèses, taux d'échantillonnage, appréciation d'incertitude/qualité des données absents.
> **Débloque :** Assurance limitée CSRD (ESRS 1 §QC + ESRS E1-6 incertitude/qualité des données), bonnes pratiques BEGES (chapitre « éléments d'appréciation de l'incertitude »), traçabilité OTI/CAC du GHG Protocol Corporate Standard (chapitre Reporting + principes de pertinence/exactitude).

---

> ⚠️ **Numérotation de migration — lire avant de coder.** Ce document écrit `0013` en supposant un dépôt à `0011`, mais **ce lot ne peut pas garder ce numéro**. Migration **réservée par le plan : `0017_add_audit_log.sql`** (cf. [README §5 — Numérotation des migrations](./README.md#numérotation-des-migrations)). **À l'exécution :** `ls src-tauri/migrations/vault/ | sort | tail -1` → prendre le numéro libre suivant, renommer le fichier `.sql`, corriger les références `00XX` de ce document, et aligner le champ `version:` dans `migrations.rs`.

## 1. Contexte & objectif

Un organisme tiers indépendant (OTI) en assurance limitée CSRD, ou un commissaire aux comptes (CAC), exige de pouvoir **reconstituer chaque chiffre** : quel facteur a été appliqué, d'où il vient, quelle hypothèse a été retenue, et avec quelle confiance. Aujourd'hui BasicPresence fige `co2_kg` + `factor_year` par trajet (R9) mais **ne consigne ni les hypothèses de modélisation, ni la version exacte de chaque facteur appliqué, ni une appréciation de l'incertitude/qualité par poste**. La méthode BEGES recommande de documenter ces « éléments d'appréciation de l'incertitude » et ESRS E1 impose une indication de la qualité des données.

L'objectif de ce lot est de **matérialiser** cette documentation : (1) un **journal des facteurs** réellement utilisés (valeur, source, version, périmètre), (2) un **journal des hypothèses** de modélisation (déjà listées en prose dans `documentation/calcul-impact-co2.md` §7-§8), (3) un **taux d'échantillonnage/couverture** des données d'activité, et (4) une **appréciation qualitative de l'incertitude par poste** (faible/moyen/élevé). Tout ceci doit apparaître dans l'export (feuille « Audit » de l'ODS) et, en lecture seule, sur la page Méthodologie.

Ce lot s'appuie sur les acquis de Lot 1 (champs `ghg_scope`/`beges_category`/`lifecycle_boundary` sur `emission_factor`) et de Lot 3 (export enrichi du facteur/unité/source/méthode par trajet). Il ne change **aucun calcul** : il ne fait qu'**observer et documenter** ce que le moteur a déjà figé.

## 2. État actuel du code (point de départ)

Faits précis tirés de la cartographie et du code lu :

- **Snapshot figé par trajet (R9).** `presence_trip` stocke `co2_kg`, `is_estimated`, `factor_year`, mais **pas** la valeur du facteur, ni sa source, ni la méthode (`src-tauri/migrations/vault/0004_add_co2.sql:73-86`). L'entité `Trip` reflète exactement ces colonnes (`src-tauri/src/domain/entities/trip.rs:16-27`).
- **Le repli fallback est silencieux.** `Co2Calculator::compute_trip` retombe sur `car_average` (`FALLBACK_MODE_ID`, `src-tauri/src/domain/services/co2_calculator.rs:24`) et lève `is_estimated = true` (`co2_calculator.rs:117-120`), mais **la raison du repli n'est pas tracée** : un trajet à occupants par défaut, un mode inconnu et un facteur estimé donnent tous le même `is_estimated = true`. Aucune granularité.
- **Le forçage radiatif dépend de l'année** (`radiative_forcing_factor`, `co2_calculator.rs:14-20` → 1,9 pour ≤ 2025, 1,7 pour ≥ 2026) et l'énergie bâtiment est optionnelle (`compute_day`, `co2_calculator.rs:87-98`). Ces hypothèses sont **portées par `Co2Settings`** (`src-tauri/src/domain/entities/co2_settings.rs:5-19`) mais **jamais consignées au moment de l'encodage** : si l'utilisateur change un réglage plus tard, on ne sait pas avec quel réglage un jour passé a été calculé.
- **L'export ODS n'a aucune feuille d'audit.** `OdsSpreadsheetExporter::write_workbook` produit 4 feuilles : Présences, Tâches, Trajets, Notes (`src-tauri/src/infrastructure/export/ods_writer.rs:71-96`). La feuille Trajets expose `mode_id`, `distance_km`, `round_trip`, `occupants`, `co2_kg`, `factor_year` (`ods_writer.rs:213-249`), sans facteur/source/méthode (ces colonnes arrivent via **Lot 3**).
- **Le use case d'export ne charge ni les facteurs, ni les `Co2Settings`.** `ExportProfileDataUseCase::execute` rassemble présences + entries + trips + notes uniquement (`src-tauri/src/application/use_cases/export_profile_data.rs:52-64`). Aucune injection d'`EmissionFactorRepository` ni de `ProfileSettingsRepository`.
- **`ExportData` ne porte que des jours.** `ExportData { days: Vec<DayExport> }` (`src-tauri/src/domain/services/spreadsheet_exporter.rs:38-42`). `ExportOptions` (`spreadsheet_exporter.rs:11-25`) et `ExportLabels` (`spreadsheet_exporter.rs:49-90`) ne prévoient ni audit, ni méta-réglages.
- **La page Méthodologie a déjà une section « Transparence & limites »** (`src/features/methodology/presentation/pages/methodology-page.tsx:327-340`) alimentée par 5 items i18n (`methodology.transparency.item1..item5`, `src/core/i18n/locales/fr/translation.json:386-394`). C'est l'emplacement naturel pour exposer l'incertitude, mais c'est aujourd'hui un texte figé, **non relié aux données réelles** (taux d'estimation, modes en repli…).
- **Le référentiel documente déjà les écarts** dans `documentation/calcul-impact-co2.md` §7 (sources vérifiées, verdicts ✅/≈/⚠️/❔) et §8 (14 écarts identifiés : étiquettes de millésime, `plane_short` = `plane_domestic`, `public_transport` = moyenne interne, `taxi` = proxy voiture moyenne, `car_phev` = hypothèse optimiste, etc.). **Ce lot transforme cette prose en données structurées et exportables.**
- **Migrations versionnées strictes.** Dernière migration connue : `0011_add_co2_2026.sql` (`src-tauri/src/infrastructure/persistence/migrations.rs:59-66`). Lot 1 et Lot 3 consomment les numéros 0012 (et éventuellement suivants). Ce lot ajoute **`0013_add_audit_log.sql`** (voir §8 pour le risque de collision de numéro).

## 3. Travail à réaliser

Tâches ordonnées. Chaque tâche est ancrée sur un fichier réel.

- **T1 — Migration `0013_add_audit_log.sql`.**
  Créer `src-tauri/migrations/vault/0013_add_audit_log.sql`. Deux objets : (a) une table `presence_audit` (un enregistrement par encodage/édition de jour, capturant les hypothèses figées) ; (b) une colonne `estimation_reason TEXT` sur `presence_trip` pour désambiguïser pourquoi un trajet est estimé. DDL complet en §4. Enregistrer la migration dans `VAULT_MIGRATIONS` (`src-tauri/src/infrastructure/persistence/migrations.rs`, ajouter `Migration { version: 13, sql: include_str!(".../0013_add_audit_log.sql") }`).

- **T2 — Entité `PresenceAudit` (domaine).**
  Créer `src-tauri/src/domain/entities/presence_audit.rs` avec la struct `PresenceAudit` (champs en §4) et l'enum `EstimationReason { ExactFactor, FallbackCarAverage, DefaultOccupancy, MissingFactor }` avec `as_str()`/`parse()` sur le modèle de `EmissionUnit`/`EmissionCategory` (`src-tauri/src/domain/entities/emission_factor.rs:17-38, 52-77`). Déclarer le module dans `src-tauri/src/domain/entities/mod.rs`.

- **T3 — Capture des hypothèses à l'encodage.**
  Dans `src-tauri/src/application/use_cases/set_presence.rs`, après le calcul `Co2Calculator::compute_day`, écrire **un enregistrement `presence_audit`** dans la même transaction que la présence + les trips (cohérence atomique, comme `presence_repository.rs`). On y fige : `factor_year`, `grid_country`, `include_radiative_forcing`, `radiative_forcing_factor(factor_year)`, `default_car_occupancy`, `count_building_energy` (tous lus depuis `Co2Settings`), plus `trip_count` et `estimated_trip_count`. En parallèle, calculer `estimation_reason` par trajet (voir §5) et le persister sur `presence_trip`. Ajouter une méthode au `PresenceRepository` (`src-tauri/src/domain/repositories/presence_repository.rs`) : `set_audit(&self, audit: &PresenceAudit)` ou intégrer l'écriture dans `set_for_day` (préféré, pour l'atomicité). Implémenter dans `src-tauri/src/infrastructure/persistence/presence_repository.rs`.

- **T4 — Service d'agrégation d'audit (domaine, pur).**
  Créer `src-tauri/src/domain/services/audit_report.rs` : un service **sans I/O** (sur le modèle de `Co2Calculator`) qui prend `ExportData` (+ les facteurs résolus + les méta-réglages) et produit un `AuditReport` : (a) `factors_used` (liste dédupliquée des `(mode_id, factor_year, value, unit, source, lifecycle_boundary, ghg_scope, beges_category)` réellement référencés par au moins un trajet) ; (b) `assumptions` (liste de couples clé/valeur figés depuis `Co2Settings` + hypothèses de modélisation documentées au §5) ; (c) `sampling` (taux de couverture, voir §5) ; (d) `uncertainty_by_post` (appréciation qualitative par poste BEGES, table de §5). Sortie 100 % déterministe et testable unitairement.

- **T5 — Enrichir le port d'export.**
  Dans `src-tauri/src/domain/services/spreadsheet_exporter.rs` : ajouter à `ExportOptions` le flag `include_audit: bool` ; ajouter à `ExportData` les champs nécessaires à l'audit (`factors: Vec<EmissionFactor>` résolus pour l'année active, `settings: Co2Settings`, `report_year: i32`) — ou un sous-objet `audit_context: Option<AuditContext>`. Étendre `ExportLabels` avec les libellés de la feuille Audit (liste en §5). Mettre à jour `From<ExportOptionsDto>`, `From<ExportLabelsDto>` et les DTO dans `src-tauri/src/application/dto/export_dto.rs` (ajouter `include_audit` + ~20 labels Audit).

- **T6 — Implémenter la feuille « Audit » dans l'ODS.**
  Dans `src-tauri/src/infrastructure/export/ods_writer.rs`, ajouter `build_audit_sheet(report, labels, styles)` et la pousser **en première position** (avant Présences) quand `options.include_audit` (cf. `write_workbook`, `ods_writer.rs:82-91`). La feuille contient 4 blocs verticaux : « Facteurs utilisés », « Hypothèses », « Échantillonnage », « Incertitude par poste » (gabarit en §5). Réutiliser `register_styles` (`ods_writer.rs:50-69`).

- **T7 — Câbler le use case d'export.**
  Dans `src-tauri/src/application/use_cases/export_profile_data.rs`, injecter `Arc<dyn EmissionFactorRepository>` et `Arc<dyn ProfileSettingsRepository>` (ajouter aux champs + `new`). Charger les facteurs de l'année active + les `Co2Settings` du profil, construire l'`AuditReport` via le service de T4, le passer à l'exporteur. Mettre à jour la composition root `src-tauri/src/lib.rs` (`build_state`) pour fournir les deux repositories au use case.

- **T8 — Exposer l'incertitude réelle sur la page Méthodologie.**
  Étendre `methodology-page.tsx` : transformer la section « Transparence & limites » (`methodology-page.tsx:327-340`) en deux sous-blocs : (a) les limites textuelles existantes (items 1-5) ; (b) un **tableau « Incertitude par poste »** alimenté par les facteurs déjà chargés (`useCo2Referential`) et la table qualitative de §5. Ajouter les clés i18n `methodology.uncertainty.*` dans `src/core/i18n/locales/fr/translation.json` **et** `src/core/i18n/locales/en/translation.json` (parité obligatoire).

- **T9 — Documentation.**
  Compléter `documentation/calcul-impact-co2.md` d'une section **§11 « Audit & incertitude »** : décrire le journal des facteurs, le journal des hypothèses, le taux d'échantillonnage et la grille d'incertitude par poste, en renvoyant explicitement aux §7 et §8 existants (sources vérifiées + 14 écarts). Documenter le mapping `EstimationReason`.

## 4. Modèle de données / migrations

Nouvelle migration **`src-tauri/migrations/vault/0013_add_audit_log.sql`** (DDL strict, idempotent comme les autres : `CREATE TABLE IF NOT EXISTS`, `INSERT OR IGNORE`). Numéro 0013 car 0012 (et suivants) sont réservés à Lot 1/Lot 3 — voir le risque de collision en §8.

```sql
-- Journal d'audit (vault chiffré) : pour chaque encodage/édition d'un jour, fige
-- les hypothèses de calcul réellement appliquées. Permet à un OTI/CAC de
-- reconstituer "avec quels réglages ce jour a été calculé", indépendamment des
-- réglages courants du profil (qui peuvent avoir changé depuis).
CREATE TABLE IF NOT EXISTS presence_audit (
    id                        TEXT PRIMARY KEY,          -- UUID v7
    presence_id               TEXT NOT NULL,             -- FK -> presence(id)
    factor_year               INTEGER NOT NULL,          -- millésime résolu
    grid_country              TEXT NOT NULL,             -- 'FR'|'BE'|'DE'|'EU'
    include_radiative_forcing INTEGER NOT NULL,          -- 0/1
    radiative_forcing_factor  REAL NOT NULL,             -- 1.9 (<=2025) / 1.7 (>=2026)
    default_car_occupancy     INTEGER NOT NULL,          -- hypothèse covoiturage
    count_building_energy     INTEGER NOT NULL,          -- 0/1
    trip_count                INTEGER NOT NULL,          -- nb de trajets du jour
    estimated_trip_count      INTEGER NOT NULL,          -- nb de trajets is_estimated
    recorded_at               INTEGER NOT NULL,          -- epoch ms (= updated_at du jour)
    FOREIGN KEY (presence_id) REFERENCES presence(id) ON DELETE CASCADE
);
CREATE INDEX IF NOT EXISTS idx_presence_audit_presence ON presence_audit(presence_id);

-- Désambiguïse le drapeau is_estimated par trajet : pourquoi l'estimation ?
-- NULL pour les trajets exacts (rétro-compat : jours pré-Lot-9 restent NULL).
ALTER TABLE presence_trip ADD COLUMN estimation_reason TEXT;  -- voir §5 (enum)
```

Entité domaine `PresenceAudit` (fichier `src-tauri/src/domain/entities/presence_audit.rs`) :

```rust
pub struct PresenceAudit {
    pub id: String,
    pub presence_id: String,
    pub factor_year: i32,
    pub grid_country: String,
    pub include_radiative_forcing: bool,
    pub radiative_forcing_factor: f64,
    pub default_car_occupancy: i64,
    pub count_building_energy: bool,
    pub trip_count: i64,
    pub estimated_trip_count: i64,
    pub recorded_at: i64,
}

pub enum EstimationReason {
    ExactFactor,         // facteur du référentiel trouvé pour (mode_id, factor_year)
    FallbackCarAverage,  // mode_id inconnu -> repli car_average (R6)
    DefaultOccupancy,    // occupants non saisis -> default_car_occupancy
    MissingFactor,       // mode inconnu ET pas de car_average -> co2 = 0, signalé
}
```

> **Pas de colonne `co2_t` ni de mapping Scope/poste ici** : ils sont fournis par Lot 3 (tCO2e) et Lot 1 (`ghg_scope`/`beges_category`/`lifecycle_boundary`). Ce lot **consomme** ces colonnes, il ne les crée pas.

## 5. Spécification détaillée

### 5.1 Détermination de `estimation_reason` (par trajet, à l'encodage)

Calculée dans `set_presence.rs` à partir du résultat de `Co2Calculator::compute_trip`. Priorité de haut en bas :

| Condition (sur le trajet et le référentiel résolu) | `estimation_reason` |
|---|---|
| `mode_id` absent du référentiel **et** `car_average` absent | `missing_factor` |
| `mode_id` absent du référentiel **et** `car_average` présent (repli R6) | `fallback_car_average` |
| `mode_id` présent, mais occupants saisis absents → `default_car_occupancy` appliqué (uniquement modes `veh.km`) | `default_occupancy` |
| `mode_id` présent, occupants saisis | `exact_factor` (colonne = NULL pour économie) |

`presence_audit.estimated_trip_count` = nombre de trajets dont `estimation_reason ∈ {fallback_car_average, missing_factor}` (cohérent avec le `is_estimated` du moteur, `co2_calculator.rs:85`). `default_occupancy` n'est **pas** une estimation du facteur mais une hypothèse d'activité : compté séparément dans le rapport (§5.4), pas dans `estimated_trip_count`.

### 5.2 Journal des facteurs utilisés (feuille Audit, bloc 1)

Liste **dédupliquée** des facteurs réellement référencés par au moins un trajet de la période exportée (clé `(mode_id, factor_year)`). Colonnes :

| Colonne ODS | Source (champ) |
|---|---|
| Mode | `EmissionFactor.label` (`emission_factor.rs:86`) |
| `mode_id` | `EmissionFactor.id` |
| Millésime | `factor_year` du trajet (audit/R9, `trip.rs:25`) |
| Facteur | `EmissionFactor.value` |
| Unité | `EmissionUnit::as_str()` (`emission_factor.rs:18-24`) |
| Périmètre cycle de vie | `lifecycle_boundary` (renommé en Lot 1 ; ex. `usage(WtW)+fabrication`, cf. seed `0011_add_co2_2026.sql:31`) |
| Scope GHG | `ghg_scope` (Lot 1) |
| Poste BEGES | `beges_category` (Lot 1) |
| Source | `EmissionFactor.source` (ex. `ADEME Base Carbone`, `DEFRA/DESNZ 2024`) |

> Le facteur **aérien** est stocké forçage radiatif inclus ; la ligne d'audit reprend le périmètre `usage+forçage radiatif (1,7)` tel que seedé (`0011_add_co2_2026.sql:51-54`). Le multiplicateur effectif est aussi rappelé dans le bloc Hypothèses.

### 5.3 Journal des hypothèses (feuille Audit, bloc 2)

Couples clé/valeur. Une partie est figée par jour (`presence_audit`) ; les hypothèses de modélisation transverses sont reprises de `documentation/calcul-impact-co2.md` §7-§8 :

| Hypothèse | Valeur exportée | Origine |
|---|---|---|
| Millésime du référentiel | `factor_year` (ex. 2026) | `Co2Settings.factor_year` |
| Forçage radiatif aérien | activé/désactivé · ×1,7 (≥2026) ou ×1,9 (≤2025) | `include_radiative_forcing` + `radiative_forcing_factor` (`co2_calculator.rs:14-20`) |
| PRG (horizon) | 100 ans (GIEC) | constante de modélisation (immuable) |
| Pays de recharge (grille élec.) | `grid_country` (ex. BE) | `Co2Settings.grid_country` |
| Occupation voiture par défaut | `default_car_occupancy` (ex. 1) | `Co2Settings` |
| Énergie du bâtiment | comptée/non comptée (office_day 3,5 · home_day 2,7 kgCO₂e/j) | `count_building_energy` (`co2_calculator.rs:87-98`) |
| Hybride rechargeable (`car_phev`) | hypothèse d'usage majoritairement électrique (optimiste) | calcul-impact-co2.md §7.1, §8 #14 |
| Taxi/VTC | proxy = voiture moyenne (pas de FE dédié) | seed `0011...:36` ; §7.1 |
| Transport en commun (mix) | moyenne pondérée interne (pas un FE ADEME publié) | seed `0011...:42` ; §7.3, §8 #6 |
| Scooter électrique | aligné sur la trottinette (pas de FE dédié) | seed `0011...:41` ; §8 #8 |
| Avion moyen-courrier | interpolation court/long (pas de bande DEFRA dédiée) | seed `0011...:53` ; §8 #4 |
| Train SNCB | donnée de traction historique (chiffre courant plus bas) | §7.4, §8 |
| Unité de restitution | kgCO₂e (et tCO₂e si Lot 3 actif) | calcul-impact-co2.md §4, Lot 3 |

### 5.4 Taux d'échantillonnage / couverture (feuille Audit, bloc 3)

BasicPresence est un tracker **par déclaration journalière** (pas d'enquête par échantillon) : la « couverture » est donc le ratio de jours/trajets réellement documentés vs. attendus. Indicateurs :

| Indicateur | Définition | Source |
|---|---|---|
| Jours documentés | nombre de présences exportées | `ExportData.days.len()` |
| Jours porteurs d'empreinte | présences `office`/`remote` avec ≥1 trajet | filtrage sur `presence.kind` + `trips` |
| Jours ouvrés attendus | `working_days_per_year` (annuel) ou plage de la période | `Co2Settings.working_days_per_year` (défaut 220) |
| Taux de couverture | jours documentés ÷ jours attendus | calculé |
| Trajets totaux / estimés | Σ trajets · Σ `estimation_reason ∈ {fallback, missing}` | `presence_trip` |
| Part estimée | trajets estimés ÷ trajets totaux (%) | calculé |
| Jours à occupation par défaut | Σ trajets `default_occupancy` | `presence_trip.estimation_reason` |

> Si une **consolidation par échantillon** est introduite plus tard (Lot 4 entité/site), ce bloc accueillera le **taux d'échantillonnage de l'enquête** (répondants ÷ effectif). Aujourd'hui : couverture = exhaustivité déclarative.

### 5.5 Appréciation de l'incertitude par poste (feuille Audit, bloc 4 + page Méthodologie)

Grille qualitative (faible / moyen / élevé) par famille de modes, dérivée des verdicts de `calcul-impact-co2.md` §7 (✅ → faible, ≈ → moyen, ⚠️/❔ → élevé) :

| Poste / famille | Modes concernés | Niveau d'incertitude | Justification (réf. §7-§8) |
|---|---|---|---|
| Voiture thermique | `car_petrol`, `car_diesel`, `car_average` | Moyen | Millésime ADEME antérieur ; ordre de grandeur correct (§7.1) |
| Voiture hybride rechargeable | `car_phev` | Élevé | Hypothèse usage électrique optimiste (§7.1, §8 #14) |
| Voiture électrique | `car_ev` (+ variantes pays) | Faible-Moyen | FR/EU bien reconstruits ; DE bord haut ; BE corrigé 2026 (§7.7) |
| Taxi / VTC | `taxi` | Élevé | Proxy voiture moyenne, pas de FE dédié (§7.1) |
| Mobilité active | `walk`, `bike`, `ebike`, `escooter` | Faible | Valeurs ADEME exactes (§7.2) |
| Scooter électrique | `scooter_elec` | Élevé | Recopie trottinette, pas de FE dédié (§7.2, §8 #8) |
| Transport en commun (mix) | `public_transport` | Élevé | Moyenne pondérée interne, non publiée (§7.3, §8 #6) |
| Bus / métro / autocar | `bus`, `metro_tram`, `coach` | Faible-Moyen | Autocar exact ; bus/métro corrigés 2026 (§7.3) |
| Train (FR) | `train_ter`, `train_hs_fr` | Faible | SNCF Open Data périmètre complet (§7.4) |
| Train (SNCB/opérateurs) | `train_sncb`, `train_eurostar`, `train_thalys` | Moyen | SNCB daté ; Thalys = estimation (§7.4, §8 #10) |
| Avion | `plane_*` | Moyen-Élevé | RF ; moyen-courrier interpolé (§7.5, §8 #3-#4) |
| Énergie du bâtiment | `office_day`, `home_day` | Moyen | Forfaits CIBSE/DEFRA, journée 8 h supposée (§7.6) |

### 5.6 Labels i18n à ajouter

- **Backend (`ExportLabels` + `ExportLabelsDto`)** : `sheet_audit`, `audit_factors_title`, `audit_assumptions_title`, `audit_sampling_title`, `audit_uncertainty_title`, `audit_factor`, `audit_unit`, `audit_lifecycle`, `audit_ghg_scope`, `audit_beges`, `audit_source`, `audit_millesime`, `audit_assumption`, `audit_value`, `audit_indicator`, `audit_post`, `audit_level`, `audit_justification`, `audit_coverage_rate`, `audit_estimated_share`. (camelCase en wire, cf. `export_dto.rs:34-65`.)
- **Frontend (`methodology.uncertainty.*`, fr ET en)** : `title`, `intro`, `post`, `level`, `justification`, `levelLow`, `levelMedium`, `levelHigh`.

## 6. Critères d'acceptation

- **CA1.** Quand un jour `office`/`remote` est encodé, alors une ligne `presence_audit` est créée dans la même transaction, avec `factor_year`, `grid_country`, `include_radiative_forcing`, `radiative_forcing_factor`, `default_car_occupancy`, `count_building_energy`, `trip_count` et `estimated_trip_count` figés depuis les `Co2Settings` du moment.
- **CA2.** Quand un trajet retombe sur `car_average` (mode inconnu), alors `presence_trip.estimation_reason = 'fallback_car_average'` ; quand le mode est connu et les occupants saisis, alors `estimation_reason` est `NULL` (= exact).
- **CA3.** Quand l'export est lancé avec `include_audit = true`, alors l'ODS contient une feuille « Audit » **en première position**, avec les 4 blocs (Facteurs utilisés, Hypothèses, Échantillonnage, Incertitude par poste).
- **CA4.** Le bloc « Facteurs utilisés » ne liste **que** les facteurs réellement référencés par au moins un trajet de la période, sans doublon `(mode_id, factor_year)`, et chaque ligne porte valeur + unité + source + périmètre (+ scope GHG / poste BEGES si Lot 1 actif).
- **CA5.** Le bloc « Échantillonnage » affiche un taux de couverture = jours documentés ÷ jours attendus, et une part estimée = trajets estimés ÷ trajets totaux, cohérente avec `estimated_trip_count`.
- **CA6.** Le bloc « Incertitude par poste » classe chaque famille en faible/moyen/élevé conformément à la grille §5.5, et la même grille apparaît sur la page Méthodologie.
- **CA7. (Rétro-compat R9.)** Un jour encodé **avant** ce lot (sans ligne `presence_audit`, `estimation_reason` NULL) s'exporte sans erreur : le rapport indique « audit non disponible (jour antérieur) » pour ce jour, et son `co2_kg` figé reste inchangé.
- **CA8.** Aucun calcul de `co2_kg`/`total_kg` n'est modifié : les tests `ac1..ac10` (`co2_calculator.rs:237-478`) passent inchangés.
- **CA9.** Parité i18n fr/en : toutes les clés `methodology.uncertainty.*` et libellés Audit existent dans les deux locales.

## 7. Tests à écrire / mettre à jour

- **Rust — service `audit_report` (pur, nouveau).**
  Dans `src-tauri/src/domain/services/audit_report.rs`, module `#[cfg(test)]` (sur le modèle `co2_calculator.rs:166-479`) :
  - `factors_used_is_deduplicated` : 3 trajets dont 2 même `(mode_id, factor_year)` → 2 lignes.
  - `factors_used_only_referenced_modes` : un mode du référentiel non utilisé n'apparaît pas.
  - `coverage_rate_computation` : 110 jours documentés / 220 attendus → 0,5.
  - `estimated_share_counts_fallback_and_missing_only` : `default_occupancy` exclu du compte estimé.
  - `uncertainty_grid_maps_modes_to_levels` : `taxi`→élevé, `train_hs_fr`→faible, `car_phev`→élevé.
- **Rust — `EstimationReason` (entité).**
  `as_str`/`parse` round-trip pour les 4 variantes ; `parse("inconnu")` → `DomainError::Validation`.
- **Rust — `ods_writer` (mettre à jour `ods_writer.rs:276-407`).**
  Étendre `writes_a_readable_multi_sheet_ods` ou ajouter `writes_audit_sheet_when_enabled` : activer `include_audit`, ré-ouvrir avec `calamine`, vérifier que la feuille « Audit » existe et que ses en-têtes de blocs sont présents. Vérifier l'**absence** de la feuille Audit quand `include_audit = false`.
- **Rust — `export_profile_data` / `integration_tests.rs`.**
  Ajouter au parcours d'export existant (`src-tauri/src/integration_tests.rs`) : encoder un jour avec un mode inconnu, exporter avec audit, vérifier que la feuille Audit signale ≥1 trajet estimé et le bon taux. Vérifier qu'un jour pré-audit (sans `presence_audit`) s'exporte sans panique (CA7).
- **Rust — `set_presence`.**
  Test : encoder un jour, relire `presence_audit` (mock repo) et asserter les hypothèses figées (CA1) ; encoder un mode inconnu et asserter `estimation_reason = fallback_car_average` (CA2).
- **Frontend — vitest.**
  `methodology-page.test.tsx` : monter la page, vérifier le rendu du tableau « Incertitude par poste » avec au moins une ligne par niveau ; vérifier l'absence de clé i18n brute (regex `methodology.uncertainty.`).

Commandes :

```sh
# Backend
cd src-tauri && cargo test
cargo fmt --check && cargo clippy --all-targets -- -D warnings
# Frontend
pnpm test && pnpm typecheck && pnpm lint
```

## 8. Points d'attention & pièges

- **Rétro-compatibilité R9 (figeage des jours) — critique.** Ce lot **n'altère jamais** un `co2_kg`/`factor_year` déjà figé (`presence_trip`). `presence_audit` et `estimation_reason` sont des **métadonnées ajoutées**, NULL pour les jours antérieurs. Ne **jamais** recalculer rétroactivement un jour pour « remplir » l'audit : si un audit manque, on l'affiche comme indisponible (CA7). L'audit reflète l'encodage **tel qu'il a eu lieu**.
- **Collision de numéro de migration.** Lot 1 prend `0012` (mapping GHG/BEGES). Si Lot 3 prend aussi une migration (tCO2e stocké), elle pourrait être `0013`. **Avant d'écrire 0013, vérifier le dernier numéro réellement présent** dans `src-tauri/migrations/vault/` et incrémenter ; ne jamais éditer une migration déjà livrée (reproductibilité, code-review §2.3). Renuméroter au besoin (`0013`→`0014`) et ajuster l'entrée `VAULT_MIGRATIONS` (`migrations.rs`).
- **Dépendance dure aux colonnes de Lot 1.** Le bloc « Facteurs utilisés » lit `lifecycle_boundary`, `ghg_scope`, `beges_category`. Si Lot 1 n'est pas mergé, ces colonnes n'existent pas : prévoir un repli (afficher le `scope` legacy + « Scope GHG : à mapper ») pour ne pas bloquer l'export. Idem Lot 3 pour la colonne facteur/source par trajet — l'audit **réutilise** ces chargements plutôt que de les dupliquer (éviter le N+1 : charger tous les facteurs de l'année **une fois**, cf. `factor_maps.rs`).
- **Couches DDD.** Le service `audit_report` doit rester **pur** (zéro I/O, zéro type tiers), comme `Co2Calculator`. Le chargement des facteurs/réglages se fait dans le use case (`export_profile_data.rs`), pas dans le service. Les nouveaux libellés restent **côté frontend** (`ExportLabels` reçoit du déjà-traduit) : le backend n'i18n jamais (`ods_writer.rs` n'a aucun texte en dur).
- **i18n.** Parité fr/en obligatoire (`methodology.uncertainty.*` + libellés Audit). Une clé manquante côté `en` casse l'export ou rend une chaîne brute (cf. risque `i18n keys missing`).
- **Atomicité.** Écrire `presence_audit` dans **la même transaction** que `presence` + `presence_trip` (`presence_repository.rs` est déjà transactionnel) — sinon un crash entre les deux laisse un jour sans audit, qui apparaîtra à tort comme « antérieur ».
- **Non-régression export.** La feuille Audit est **opt-in** (`include_audit`, défaut `false`) : les exports existants restent identiques. Les 4 feuilles actuelles (`ods_writer.rs:82-91`) ne changent pas.
- **Mouvance réglementaire.** La grille d'incertitude (§5.5) et le journal d'hypothèses (§5.3) sont dérivés de `calcul-impact-co2.md` §7-§8 : **toute correction de facteur via un nouveau millésime** (jamais un UPDATE rétroactif du seed, R9) doit être répercutée dans la grille et la prose. L'horizon PRG (100 ans GIEC) est une constante de modélisation, à ne pas confondre avec le millésime des facteurs.
- **Faux sens « scope ».** Ne pas exporter le `scope` legacy comme « Scope GHG » : c'est le **périmètre cycle de vie** (renommé `lifecycle_boundary` en Lot 1). Le Scope GHG (1/2/3) est une colonne distincte. L'audit doit afficher les deux séparément (cf. risque gap a).

## 9. Références réglementaires

- **Méthode BEGES (Bilan GES réglementaire, art. L229-25 Code de l'environnement)** — Guide méthodologique ADEME/MTE : exige des « éléments d'appréciation sur les incertitudes » et la documentation des facteurs et hypothèses retenus. https://bilans-ges.ademe.fr/ (méthode pour la réalisation des bilans d'émissions de GES).
- **ADEME — Base Empreinte® / Base Carbone® (facteurs, incertitudes associées)** : https://base-empreinte.ademe.fr/ — chaque facteur publie une incertitude ; à rapprocher de la grille §5.5.
- **CSRD / ESRS — Règlement délégué (UE) 2023/2772** : ESRS 1 (caractéristiques qualitatives de l'information, dont l'exactitude et la documentation des estimations) et **ESRS E1-6** (émissions brutes Scopes 1/2/3 en **tCO₂e**, avec indication de la qualité des données et des hypothèses). https://eur-lex.europa.eu/legal-content/FR/TXT/?uri=CELEX:32023R2772
- **Assurance limitée CSRD — Directive (UE) 2022/2464 (CSRD)** : obligation d'assurance par un OTI/CAC, qui suppose une **piste d'audit** reconstituant donnée d'activité → facteur → résultat. https://eur-lex.europa.eu/legal-content/FR/TXT/?uri=CELEX:32022L2464
- **GHG Protocol Corporate Standard** — chapitres « Reporting » et principes (pertinence, exactitude, transparence) : la documentation des facteurs et hypothèses est requise pour la vérifiabilité. https://ghgprotocol.org/corporate-standard
- **GHG Protocol — Corporate Value Chain (Scope 3) Standard**, catégorie 7 « Employee commuting » : recommande de documenter la méthode (distance-based vs. average-data) et la qualité des données. https://ghgprotocol.org/standards/scope-3-standard
- **GIEC (IPCC) — PRG/GWP 100 ans** : horizon de caractérisation des gaz, constante de modélisation à déclarer dans la méthodologie d'export. https://www.ipcc.ch/
- **Sources des facteurs et écarts** (base de la grille d'incertitude) : `documentation/calcul-impact-co2.md` §7 (sources vérifiées, verdicts) et §8 (14 écarts identifiés).
