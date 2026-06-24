# Lot 3 — Export auditable (traçabilité source → résultat)

> **Statut :** À faire  ·  **Priorité :** P1  ·  **Effort :** faible-moyen  ·  **Dépend de :** Lot 1 (classification Scope GHG / poste BEGES / ESRS), Lot 2 (conversion tCO2e), idéalement Lot 6 (en-tête normatif)
> **Gap couvert :** (b) L'export ODS/JSON ne contient ni le facteur d'émission, ni son unité, ni sa source/version, ni la méthode, ni la donnée d'activité brute : un OTI/CAC ne peut pas retracer un résultat.
> **Débloque :** BEGES (traçabilité des données d'activité et facteurs, art. R.229-46 et seq.), GHG Protocol Corporate Standard / Scope 3 Standard (catégorie 7 « Employee commuting »), CSRD/ESRS E1 (assurance limitée : piste d'audit donnée → facteur → résultat).

---

## 1. Contexte & objectif

Aujourd'hui l'export d'un profil (`.ods`) écrit, par trajet, uniquement `mode_id`, `distance_km`, `round_trip`, `occupants`, `co2_kg` et `factor_year`. Le **facteur** utilisé, son **unité**, sa **source/version** et la **méthode** (distance-based vs average-data) sont absents : impossible pour un Organisme Tiers Indépendant (OTI) ou un Commissaire aux Comptes (CAC) de reconstituer le calcul sans la base de l'app. L'objectif de ce lot est de rendre chaque ligne d'export **autonome et réversible** : depuis la donnée d'activité brute jusqu'au résultat en kgCO2e **et** tCO2e, en passant par le facteur exact, sa source datée et sa classification (Scope GHG, catégorie 7, poste BEGES 3.3, ESRS). Résultat attendu : un classeur que l'on peut intégrer tel quel à un BEGES ou présenter en assurance limitée CSRD, sans pièce externe.

## 2. État actuel du code (point de départ)

- **Port d'export** : `src-tauri/src/domain/services/spreadsheet_exporter.rs`
  - `trait SpreadsheetExporter::write_workbook(data, options, labels, path)` (lignes 116-124).
  - `struct DayExport { presence, entries, trips, note }` (lignes 29-36) : **un jour porte ses `Vec<Trip>` mais AUCUN `EmissionFactor`** — le facteur n'est jamais chargé pour l'export.
  - `struct ExportOptions { include_carbon, include_hours, include_tasks, include_trips, include_notes, include_timestamps }` (lignes 10-25).
  - `struct ExportLabels { … }` (lignes 48-90) : 28 champs `String` traduits, fournis par le front. Colonnes Trajets actuelles : `mode`, `distance`, `round_trip`, `occupants`, `factor_year` + `co2` partagé.
- **Implémentation ODS** : `src-tauri/src/infrastructure/export/ods_writer.rs`
  - `build_trips_sheet` (lignes 213-249) : en-têtes `[date, mode, distance, round_trip, occupants, co2, factor_year]` (lignes 215-223) ; données écrites colonnes 0→6 (lignes 231-241). C'est ici qu'on ajoutera les colonnes d'audit.
  - `build_presences_sheet` (lignes 100-170) ; `Styles { header, date, datetime }` (lignes 44-48).
- **Use case** : `src-tauri/src/application/use_cases/export_profile_data.rs`
  - `ExportProfileDataUseCase { presences, work_entries, exporter }` (lignes 12-16) — **ne possède PAS de `EmissionFactorRepository`**.
  - `execute` (lignes 31-80) : charge `presences.list_by_profile`, puis par jour `work_entries.list_by_presence`, `presences.list_trips`, `presences.get_note`, assemble `DayExport`. C'est le point d'injection du référentiel.
- **Snapshot trajet** : `src-tauri/src/domain/entities/trip.rs`
  - `Trip { id, mode_id, distance_km, round_trip, occupants, co2_kg, is_estimated, factor_year, position }` (lignes 16-27). `distance_km` est **one-way** (commentaire ligne 1-2) ; `round_trip` est le flag journée appliqué à chaque leg.
- **Référentiel facteurs** : `src-tauri/src/domain/entities/emission_factor.rs`
  - `EmissionFactor { id, label, value, unit, category, is_param, scope, source }` (lignes 82-96). `scope` = **périmètre cycle de vie** (well-to-wheel), PAS le Scope GHG ; renommé `lifecycle_boundary` au Lot 1.
  - `EmissionUnit::as_str()` → `"kgCO2e/veh.km" | "kgCO2e/passenger.km" | "kgCO2e/km" | "kgCO2e/day"` (lignes 18-25).
  - `EmissionCategory` : `Car | Active | PublicTransport | Rail | Air | Building` (lignes 43-50).
- **Repository facteurs** : `src-tauri/src/infrastructure/persistence/emission_factor_repository.rs`
  - `list(year) -> Vec<EmissionFactor>` (lignes 57-71), `list_grid_variants(year) -> Vec<GridVariant>` (lignes 73-88). Charge tout le millésime en une requête (pas de N+1 si l'on indexe en `HashMap`).
- **Calculateur (référence pour la méthode/donnée d'activité)** : `src-tauri/src/domain/services/co2_calculator.rs`
  - `compute_trip` (lignes 110-163) : fallback `FALLBACK_MODE_ID = "car_average"` (ligne 24) si `mode_id` absent → `is_estimated = true` (lignes 117-120). Variante réseau électrique = **remplace** la valeur de base (lignes 130-133). RF aérien = **divise** si désactivé (lignes 134-138). Division par occupants **uniquement** pour `VehKm` (lignes 145-149).
- **DTO & commande** : `src-tauri/src/application/dto/export_dto.rs` (`ExportOptionsDto`, `ExportLabelsDto`, `ExportSummaryDto`, camelCase) ; `src-tauri/src/presentation/commands/export.rs` (`export_profile_data`).
- **Frontend** : `src/features/export/domain/entities/export.ts` (`ExportOptions`, `ExportLabels`, `DEFAULT_EXPORT_OPTIONS`) ; `src/features/export/presentation/build-export-labels.ts` (`buildExportLabels(t)`) ; `src/features/export/presentation/components/export-dialog.tsx` (`OPTION_KEYS`). i18n sous `export.sheet.cols.*` (`src/core/i18n/locales/fr/translation.json` lignes 200-227, miroir `en`).

> **Point clé** : l'export ne recharge jamais le référentiel ; le facteur est donc à injecter via un nouveau dépôt dans le use case (gap structurel), pas seulement à formater.

## 3. Travail à réaliser

Ordre d'implémentation (chaque tâche s'appuie sur la précédente) :

- **T1 — Enrichir le port d'export avec le facteur résolu par trajet.**
  Fichier : `src-tauri/src/domain/services/spreadsheet_exporter.rs`.
  - Ajouter une structure `TripAudit` portant le trajet **et** sa fiche facteur résolue :
    ```rust
    /// A trip plus the resolved emission-factor metadata it was computed against,
    /// so the export carries a full activity → factor → result audit trail.
    #[derive(Debug, Clone)]
    pub struct TripAudit {
        pub trip: Trip,
        /// Factor row for (trip.mode_id, trip.factor_year); `None` when the mode
        /// is unknown for that year (fell back to car_average at encode time).
        pub factor: Option<EmissionFactorAudit>,
        /// Effective per-unit factor actually applied (after grid variant / RF).
        pub effective_value: f64,
        /// Resolution method (see §5): Distance-based or AverageData (fallback).
        pub method: TripMethod,
        /// Grid country active at export (the calc applied its car_ev variant).
        pub grid_country: String,
    }
    ```
  - Ajouter `EmissionFactorAudit { mode_id, label, base_value, unit, source, lifecycle_boundary, ghg_scope, ghg_category, beges_poste, esrs_datapoint }` (les 4 derniers proviennent du Lot 1 ; voir §5 & §8).
  - Ajouter `enum TripMethod { DistanceBased, AverageData }`.
  - Remplacer le champ `trips: Vec<Trip>` de `DayExport` par `trips: Vec<TripAudit>` (lignes 33). Le note/entries/presence restent inchangés.

- **T2 — Charger le référentiel dans le use case et résoudre chaque trajet.**
  Fichier : `src-tauri/src/application/use_cases/export_profile_data.rs`.
  - Injecter `factors: Arc<dyn EmissionFactorRepository>` et `profile_settings: Arc<dyn ProfileSettingsRepository>` dans `ExportProfileDataUseCase` (champs + `new`, lignes 12-29).
  - Dans `execute` : charger une seule fois `Co2Settings` du profil (via `profile_settings.load(profile_id)`) pour récupérer `grid_country`. Puis, **pour chaque `factor_year` distinct rencontré dans les trips**, charger `factors.list(year)` et `factors.list_grid_variants(year)`, et les indexer en `HashMap<String, EmissionFactor>` / `HashMap<(String,String), f64>` (réutilise la logique de `factor_maps.rs`). Mettre ces maps en cache local `HashMap<i32, …>` pour éviter tout N+1.
  - Pour chaque `Trip`, construire le `TripAudit` : résoudre `factor = maps[year].get(mode_id)` ; si `None`, méthode = `AverageData` et tenter `maps[year].get("car_average")` pour la fiche affichée ; calculer `effective_value` (variante réseau si `(mode_id, grid_country)` matche, sinon RF aérien selon settings — **mêmes règles que `compute_trip`**, voir §5).
  - Mapper `EmissionFactor → EmissionFactorAudit` (recopier `ghg_scope`/`beges_poste`/`esrs_datapoint` issus du Lot 1).

- **T3 — Écrire les nouvelles colonnes d'audit dans la feuille Trajets.**
  Fichier : `src-tauri/src/infrastructure/export/ods_writer.rs`, `build_trips_sheet` (lignes 213-249).
  - Réorganiser les en-têtes selon le **modèle de colonnes GHG Protocol Scope 3 cat.7** du §5.2. Écrire chaque cellule en respectant le type (réels pour les facteurs/distances/résultats, texte pour les libellés). Le résultat **kgCO2e** et le résultat **tCO2e** (= `co2_kg / 1000.0`, Lot 2) sont deux colonnes distinctes.
  - Ajouter une colonne **« Émissions évitées (kgCO2e) »** laissée vide par défaut (voir §5.4) — jamais soustraite du total.
  - Élargir les `set_col_width` pour les nouvelles colonnes texte (source, méthode, classification).

- **T4 — Étendre `ExportLabels` (port + DTO) avec les nouveaux libellés de colonnes.**
  Fichiers : `spreadsheet_exporter.rs` (struct `ExportLabels`, bloc « Trajets columns » lignes 81-86), `src-tauri/src/application/dto/export_dto.rs` (`ExportLabelsDto` + `From` impl).
  - Ajouter les champs `String` : `distance_one_way`, `passenger_km`, `wfh_days`, `factor_value`, `factor_unit`, `factor_source`, `method`, `method_distance_based`, `method_average_data`, `ghg_scope`, `ghg_category`, `beges_poste`, `esrs_datapoint`, `co2_t` (si non déjà ajouté au Lot 2), `avoided_emissions`, `sheet_audit_legend`.
  - Mettre à jour la conversion `From<ExportLabelsDto> for ExportLabels`.

- **T5 — Fournir les libellés depuis le front (i18n).**
  Fichiers : `src/features/export/domain/entities/export.ts` (interface `ExportLabels`), `src/features/export/presentation/build-export-labels.ts`, `src/core/i18n/locales/fr/translation.json` + `…/en/translation.json` (bloc `export.sheet.cols.*`).
  - Ajouter les clés FR/EN listées en §5.5, et les câbler dans `buildExportLabels(t)`.

- **T6 — (Optionnel mais recommandé) Refléter la traçabilité dans le bundle JSON.**
  Fichiers : `src-tauri/src/infrastructure/transfer/json_bundle_codec.rs`, `src-tauri/src/application/use_cases/export_profile_bundle.rs`.
  - Enrichir chaque trip du bundle avec `factorValue`, `factorUnit`, `factorSource`, `ghgScope`, `begesposte` (camelCase). **Bump `version` du `BundleFile` à 2** (la lecture v1 reste tolérée : champs absents → `None`). Ne pas casser l'import existant (cf. §8).

- **T7 — Brancher la commande / DI.**
  Fichier : `src-tauri/src/lib.rs` (composition root `build_state`) : passer les nouveaux `Arc<dyn …>` au constructeur `ExportProfileDataUseCase::new`. La signature de la commande `export_profile_data` (`presentation/commands/export.rs`) **n'a pas besoin de changer** (mêmes DTO d'entrée enrichis de labels).

## 4. Modèle de données / migrations

**Sans objet pour une nouvelle migration de schéma.** Ce lot **n'ajoute aucune colonne SQL** : toute l'information d'audit existe déjà (les inputs figés dans `presence_trip` + le facteur dans `emission_factor` du bon millésime). La traçabilité se reconstitue **à l'export** en recroisant `presence_trip.factor_year` avec `emission_factor(id, year)`.

> **Dépendance Lot 1** : les colonnes de classification (`ghg_scope`, `beges_poste`, `esrs_datapoint`, renommage `scope → lifecycle_boundary`) sont apportées par la migration du Lot 1 (numérotée ≥ `0012_*.sql`, la dernière connue étant `0011`). Ce lot **lit** ces colonnes, ne les crée pas. Si le Lot 1 n'est pas encore livré, livrer T1-T5 avec des valeurs de classification `None`/vide (dégradation gracieuse) et compléter au merge du Lot 1.

> **R9 respecté** : on ne réécrit ni `presence_trip.co2_kg` ni `factor_year`. Le facteur affiché est celui du **millésime figé du trajet** (`trip.factor_year`), donc reproductible même si le profil est passé à un millésime plus récent entre-temps.

## 5. Spécification détaillée

### 5.1 Donnée d'activité brute (par trajet)

| Donnée exportée | Source dans le code | Remarque |
|---|---|---|
| Mode | `Trip.mode_id` | Brut (`car_petrol`, `train_ter`, …) — pas le label localisé, pour réversibilité machine. Le label humain va dans la colonne « Facteur — libellé ». |
| Distance one-way (km) | `Trip.distance_km` | **One-way** (cf. `trip.rs` l.1-2). En-tête doit le préciser : « Distance aller (km) ». |
| Aller-retour | `Trip.round_trip` | Oui/Non. Le ×2 est appliqué dans le résultat, jamais dans la distance affichée. |
| Distance comptabilisée (km) | dérivée | `distance_km * (round_trip ? 2 : 1)` — la distance réellement multipliée par le facteur (utile à l'OTI pour reproduire). |
| Passenger-km | dérivée | Pour les facteurs `passenger.km`/`km` : `distance_comptabilisée` (déjà par personne). Pour `veh.km` : `distance_comptabilisée / max(occupants,1)`. Colonne « Donnée d'activité (unité du facteur) ». |
| Occupants | `Trip.occupants` | Diviseur appliqué **uniquement** si unité = `veh.km` (R2). |
| Jours télétravail | dérivé (Lot 5) | Hors périmètre direct ; si le Lot 5 modélise `home_day`, exposer le nb de jours WFH agrégé en feuille de synthèse (Lot 4). Ici, laisser la colonne présente mais alimentée seulement quand `mode_id = "home_day"`. |

### 5.2 Modèle de colonnes — feuille « Trajets » (GHG Protocol Scope 3 cat.7)

Ordre proposé (réversible donnée → facteur → résultat → classification) :

| # | En-tête (clé i18n) | Type | Contenu / formule |
|---|---|---|---|
| 0 | Date (`date`) | date | `presence.day` (UTC midnight) |
| 1 | Mode (`mode`) | texte | `trip.mode_id` (brut) |
| 2 | Facteur — libellé (`factor_label`*) | texte | `factor.label` (ex. « Train régional (TER) ») |
| 3 | Distance aller (km) (`distance_one_way`) | réel | `trip.distance_km` |
| 4 | Aller-retour (`round_trip`) | texte | Oui/Non |
| 5 | Occupants (`occupants`) | entier | `trip.occupants` |
| 6 | Donnée d'activité (`activity_value`*) | réel | passenger-km/veh-km/km comptabilisés (cf. §5.1) |
| 7 | Facteur (`factor_value`) | réel | `effective_value` (après variante réseau / RF) |
| 8 | Unité du facteur (`factor_unit`) | texte | `factor.unit.as_str()` (ex. `kgCO2e/passenger.km`) |
| 9 | Source + version (`factor_source`) | texte | `factor.source` + millésime (ex. « SNCF Open Data (périmètre complet 2024) — millésime 2026 ») |
| 10 | Année facteur (`factor_year`) | entier | `trip.factor_year` |
| 11 | Méthode (`method`) | texte | « Distance-based » ou « Average-data » (cf. §5.3) |
| 12 | Périmètre cycle de vie (`lifecycle_boundary`*) | texte | `factor.lifecycle_boundary` (ex-`scope`, ex. « usage(WtW)+fabrication ») |
| 13 | Scope GHG (`ghg_scope`) | texte | 1 / 2 / 3 (Lot 1) — commuting ⇒ Scope 3 |
| 14 | Catégorie GHG (`ghg_category`) | texte | « 7 » (Employee commuting) pour les trajets ; « — » pour `home_day`/`office_day` (Scope 2) |
| 15 | Poste BEGES (`beges_poste`) | texte | « 3.3 » (Déplacements domicile-travail) — Lot 1 |
| 16 | Point de donnée ESRS (`esrs_datapoint`) | texte | ex. « E1-6 » (Lot 1) |
| 17 | CO₂ (kgCO2e) (`co2`) | réel | `trip.co2_kg` (figé) |
| 18 | CO₂ (tCO2e) (`co2_t`) | réel | `trip.co2_kg / 1000.0` (Lot 2) |
| 19 | Émissions évitées (kgCO2e) (`avoided_emissions`) | réel | vide par défaut ; **jamais** dans le total (cf. §5.4) |

\* clés à ajouter au-delà de celles déjà présentes (`date`, `mode`, `round_trip`, `occupants`, `factor_year`, `co2`).

### 5.3 Règle de méthode (distance-based vs average-data)

| Condition à l'export | Méthode | `is_estimated` attendu |
|---|---|---|
| `factor = maps[year].get(mode_id)` retourne une fiche (mode connu) | **Distance-based** | `false` |
| `mode_id` absent du millésime → fallback `car_average` au calcul | **Average-data** | `true` |

La cellule « Méthode » dérive donc de `trip.is_estimated` + présence de la fiche. Aligner les libellés sur le GHG Protocol Scope 3 (Technical Guidance, cat.7) : « distance-based method » / « average-data method ».

### 5.4 Émissions évitées — champ séparé

Le GHG Protocol **interdit** de soustraire des émissions évitées (avoided emissions, ex. covoiturage « économisé », km télétravail « évités ») du total déclaré. On expose donc une colonne dédiée **« Émissions évitées (kgCO2e) »**, et :
- elle **n'entre dans aucun total** (ni feuille Trajets, ni Lot 4 consolidation) ;
- elle reste **vide** dans ce lot (pas de calcul d'évitement encore modélisé — c'est le Lot 5) ; on réserve uniquement la colonne et la convention.
- Un commentaire d'en-tête (légende, `sheet_audit_legend`) rappelle : « Les émissions évitées sont reportées séparément et ne sont jamais déduites du total (GHG Protocol). »

### 5.5 Effective_value — réplique exacte de `compute_trip`

Pour que la colonne « Facteur » corresponde **au facteur réellement appliqué** (pas la valeur de base), reproduire la résolution de `co2_calculator.rs:129-138` :

```text
ef = factor.value
si grid_variants.contains((mode_id, grid_country)) :   ef = grid_variants[(mode_id, grid_country)]   // remplace
sinon si factor.category == Air ET !include_radiative_forcing : ef = ef / radiative_forcing_factor(factor_year)  // divise
```

Edge cases :
- **Mode inconnu** (`factor = None`) : afficher la fiche `car_average` (libellé, source) avec mention « (fallback) » et méthode « Average-data ». `effective_value` = valeur `car_average` du millésime.
- **`car_average` lui-même absent** : `effective_value = 0.0`, source « — », méthode « Average-data ».
- **Vacances/jours fériés** : pas de trips → aucune ligne (R7), rien à exporter.
- **Plusieurs millésimes dans un même profil** : chaque ligne porte SON `factor_year` ; charger les maps par année (cache local).

### 5.6 Libellés i18n (FR / EN) à ajouter sous `export.sheet.cols.*`

| Clé | FR | EN |
|---|---|---|
| `factorLabel` | Facteur — libellé | Factor — label |
| `distanceOneWay` | Distance aller (km) | One-way distance (km) |
| `activityValue` | Donnée d'activité | Activity data |
| `factorValue` | Facteur | Emission factor |
| `factorUnit` | Unité du facteur | Factor unit |
| `factorSource` | Source + version | Source + version |
| `method` | Méthode | Method |
| `methodDistanceBased` | Distance (distance-based) | Distance-based |
| `methodAverageData` | Moyenne (average-data) | Average-data |
| `lifecycleBoundary` | Périmètre cycle de vie | Life-cycle boundary |
| `ghgScope` | Scope GHG | GHG Scope |
| `ghgCategory` | Catégorie GHG | GHG category |
| `begesPoste` | Poste BEGES | BEGES item |
| `esrsDatapoint` | Point de donnée ESRS | ESRS datapoint |
| `co2T` | CO₂ (t) | CO₂ (t) |
| `avoidedEmissions` | Émissions évitées (kg) | Avoided emissions (kg) |

Plus une clé hors `cols` : `export.sheet.auditLegend` (texte de légende §5.4).

## 6. Critères d'acceptation

1. **Réversibilité totale** : pour toute ligne de la feuille Trajets, `Donnée d'activité × Facteur` (en respectant l'unité) ≈ `CO₂ (kgCO2e)` à ±0,01 kg, et `CO₂ (tCO2e) = CO₂ (kgCO2e) / 1000`. *(Benchmark OTI/CAC : retracer un résultat tCO2e jusqu'à sa donnée d'activité et son facteur sans pièce externe.)*
2. Quand un trajet a `mode_id = "train_ter"` et `factor_year = 2026`, alors la colonne « Source + version » contient « SNCF Open Data (périmètre complet 2024) — millésime 2026 » et « Unité du facteur » = `kgCO2e/passenger.km`.
3. Quand un trajet utilise un mode inconnu (fallback), alors « Méthode » = « Average-data », « Facteur — libellé » mentionne « car_average (fallback) », et « Estimé »/`is_estimated` = Oui.
4. Quand `grid_country = "BE"` et `mode_id = "car_ev"` / `factor_year = 2026`, alors « Facteur » = `0.11` (variante réseau appliquée), pas `0.11`/base sans variante divergente.
5. Quand un trajet a `round_trip = true` et `distance_km = 40`, alors « Distance aller (km) » = 40 et la « Donnée d'activité » reflète 80 (×2) divisée par occupants si `veh.km`.
6. La colonne « Émissions évitées (kgCO2e) » existe, est vide par défaut, et **n'est sommée nulle part** ; la légende le rappelle.
7. Chaque ligne porte « Scope GHG » = 3, « Catégorie GHG » = 7, « Poste BEGES » = 3.3 pour les trajets domicile-travail (sous réserve du Lot 1 ; sinon vide sans erreur).
8. Le résultat figé n'est jamais recalculé : `CO₂ (kgCO2e)` exporté == `presence_trip.co2_kg` (R9). Un profil passé de 2025 à 2026 affiche toujours le facteur 2025 pour ses anciens trajets.
9. Aucune nouvelle migration SQL n'est introduite par ce lot.

## 7. Tests à écrire / mettre à jour

**Rust (cargo) :**
- `ods_writer.rs` (`mod tests`) — étendre `sample_data()`/`sample_labels()` avec `TripAudit` et les nouveaux labels, puis :
  - `fn trips_sheet_exposes_factor_value_unit_and_source()` : ré-ouvre l'ODS (calamine), vérifie en-têtes et cellules « Facteur », « Unité du facteur », « Source + version ».
  - `fn trips_sheet_result_equals_activity_times_factor()` : vérifie AC1 sur une ligne `passenger.km` et une ligne `veh.km` avec covoiturage.
  - `fn avoided_emissions_column_present_and_empty()` : colonne présente, cellule vide.
  - `fn fallback_mode_marked_average_data()` : `factor = None` → méthode « Average-data ».
  - Conserver et adapter `writes_a_readable_multi_sheet_ods()` (les 4 feuilles restent).
- `export_profile_data.rs` — test unitaire du use case avec dépôts mockés (`EmissionFactorRepository`, `ProfileSettingsRepository`, `PresenceRepository`, `WorkEntryRepository`) :
  - `fn loads_factor_per_year_without_n_plus_one()` : 2 millésimes, vérifie que `list(year)` est appelé une fois par année.
  - `fn maps_trip_to_audit_with_effective_factor()` : variante réseau BE appliquée à `car_ev`.
- `src-tauri/src/integration_tests.rs` — étendre le parcours export pour asserter la présence des colonnes d'audit dans le fichier produit.

**Frontend (vitest) :**
- `build-export-labels.test.ts` (colocalisé) : vérifier que `buildExportLabels(t)` renseigne tous les nouveaux champs (`factorValue`, `factorUnit`, `factorSource`, `method`, `ghgScope`, `begesPoste`, `esrsDatapoint`, `avoidedEmissions`, `co2T`).

**Commandes :**
```bash
cargo test -p basic-presence            # ou: (cd src-tauri && cargo test)
cargo fmt --check && cargo clippy --all-targets -- -D warnings
pnpm test
```

## 8. Points d'attention & pièges

- **R9 — figeage des jours** : ne JAMAIS recalculer `co2_kg`. Toujours exporter `trip.co2_kg`/`trip.factor_year` tels quels et résoudre le facteur **dans le millésime du trajet** (`maps[trip.factor_year]`), pas dans le millésime courant du profil. Un trajet 2025 doit afficher source et valeur 2025.
- **N+1** : charger les facteurs **par année rencontrée**, une fois, en `HashMap` (cf. `factor_maps.rs`). Ne pas requêter par trajet (risque perf sur gros profils).
- **Couches DDD** : `TripAudit`/`EmissionFactorAudit`/`TripMethod` vivent dans `domain/services/spreadsheet_exporter.rs` (port) ; la résolution facteur (use case) reste dans `application/` ; l'écriture cellule reste dans `infrastructure/export/`. Pas de `libsql`/`spreadsheet_ods` qui remonte dans le domaine.
- **i18n — clés FR ET EN obligatoires** : toute clé `export.sheet.cols.*` ajoutée en FR doit l'être en EN (sinon export vide/cassé). Le backend ne hardcode jamais de texte ; tout passe par `ExportLabels`.
- **Dépendance Lot 1** : si la classification (`ghg_scope`, `beges_poste`, `esrs_datapoint`) n'est pas encore en base, dégrader proprement (colonnes présentes, cellules vides). Ne pas bloquer le lot.
- **Dépendance Lot 2** : la colonne tCO2e suppose le helper `kg → t` (× 0,001). Si le Lot 2 a déjà ajouté `co2_t`/label, réutiliser ; sinon ajouter `co2_t = co2_kg / 1000.0` ici sans modifier le calcul interne (toujours en kgCO2e).
- **Émissions évitées** : interdiction formelle de les soustraire du total (GHG Protocol). Colonne isolée, jamais agrégée.
- **Bundle JSON (T6)** : bumper `version` à 2, garder la lecture v1 tolérante (champs neufs ⇒ `Option`), ne pas casser `import_profile_bundle` (validation untrusted input du commit 0bd3220 doit continuer à passer).
- **Non-régression** : les 4 feuilles existantes et leurs options (`include_*`) restent identiques ; seules les colonnes Trajets s'enrichissent. Vérifier que `writes_a_readable_multi_sheet_ods()` passe toujours.
- **Mouvance réglementaire** : la source doit porter sa **version/année** (ex. « DEFRA/DESNZ 2024 ») — c'est exactement ce qu'exige une assurance limitée. Ne pas afficher une source sans millésime.

## 9. Références réglementaires

- **GHG Protocol — Corporate Value Chain (Scope 3) Standard** et **Technical Guidance for Calculating Scope 3 Emissions**, catégorie 7 « Employee commuting » : méthodes *distance-based* et *average-data*, et règle des *avoided emissions* (reportées séparément, jamais déduites). https://ghgprotocol.org/scope-3-technical-calculation-guidance
- **BEGES (France)** — art. L.229-25 et R.229-46 et suivants du Code de l'environnement ; méthode réglementaire et obligation de traçabilité des données d'activité et facteurs d'émission (poste 3.3 « Déplacements domicile-travail »). https://www.legifrance.gouv.fr/codes/section_lc/LEGITEXT000006074220/LEGISCTA000022493825/
- **ADEME — Base Empreinte / Base Carbone** (facteurs FR, versionnés) : nécessité de citer la version du référentiel. https://base-empreinte.ademe.fr/
- **CSRD / ESRS E1 « Climate change »** (Règlement délégué (UE) 2023/2772) — exigences de transparence méthodologique et d'auditabilité (assurance limitée) ; datapoints E1-6 (émissions brutes Scope 1/2/3). https://eur-lex.europa.eu/eli/reg_del/2023/2772/oj
- **DEFRA/DESNZ — UK Government GHG Conversion Factors** (facteurs aérien + homeworking, version annuelle citée dans `0011_add_co2_2026.sql`). https://www.gov.uk/government/collections/government-conversion-factors-for-company-reporting
- **facteursdemissionco2.be** (référentiel belge, à intégrer au Lot 7) — pour les trajets sourcés Belgique. https://www.facteursdemissionco2.be/
