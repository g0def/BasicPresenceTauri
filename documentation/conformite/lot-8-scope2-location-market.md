# Lot 8 — Scope 2 location-based / market-based (énergie bâtiment)

> **Statut :** À faire  ·  **Priorité :** P3  ·  **Effort :** moyen  ·  **Dépend de :** Lot 1 (classification Scope GHG 1/2/3 + champ `ghg_scope` sur `emission_factor`)
> **Gap couvert :** (h) Pas de distinction Scope 2 *location-based* / *market-based* pour l'énergie bâtiment.
> **Débloque :** ESRS E1-6 (double publication Scope 2 location-based ET market-based, exigence stricte CSRD) ; alignement GHG Protocol Scope 2 Guidance (dual reporting) ; préparation BEGES (poste énergie scope 2).

---

> ⚠️ **Numérotation de migration — lire avant de coder.** Ce document écrit `0012` en supposant un dépôt à `0011`, mais **ce lot ne peut pas garder ce numéro** : il entre en collision avec le Lot 1. Migration **réservée par le plan : `0016_add_scope2_building.sql`** (cf. [README §5 — Numérotation des migrations](./README.md#numérotation-des-migrations)). **À l'exécution :** `ls src-tauri/migrations/vault/ | sort | tail -1` → prendre le numéro libre suivant, renommer le fichier `.sql`, corriger les références `00XX` de ce document, et aligner le champ `version:` dans `migrations.rs`.

## 1. Contexte & objectif

Aujourd'hui l'énergie du bâtiment (jours `office`/`remote`) est modélisée par deux forfaits uniques : `office_day` (3,5 kgCO2e/jour) et `home_day` (2,7 kgCO2e/jour), seedés dans `0011_add_co2_2026.sql` et appliqués par `Co2Calculator::compute_day` quand `count_building_energy` est activé. Ces forfaits sont *location-based* implicites (mix réseau national/par défaut) et n'admettent **aucune** variante *market-based* (électricité achetée via contrat fournisseur, ex. garanties d'origine / électricité verte).

L'ESRS E1-6 (et le *GHG Protocol Scope 2 Guidance*) impose une **double publication** de l'électricité du Scope 2 : une valeur *location-based* (mix physique du réseau) ET une valeur *market-based* (instruments contractuels). Ce lot ajoute, **sans casser le figeage R9 des jours existants**, (1) un facteur électricité bâtiment *market-based* paramétrable par profil, (2) la double sortie dans l'export ODS, et (3) le paramétrage UI côté réglages. Le calcul interne reste en kgCO2e (la conversion tCO2e relève du Lot 3/gap c). Le résultat attendu : un export montrant, pour chaque jour `office`/`remote`, la composante énergie bâtiment en location-based **et** en market-based, qualifiée par sa méthode Scope 2.

---

## 2. État actuel du code (point de départ)

Faits précis tirés du code en place :

- **Forfaits bâtiment uniques.** `0011_add_co2_2026.sql:55-56` seede deux facteurs `building` :
  - `office_day` = `3.5` `kgCO2e/day`, scope (périmètre cycle de vie) `'usage (bâtiment, hors trajet)'`, source `'CIBSE TM46 / Circular Ecology (Univ. Exeter)'`.
  - `home_day` = `2.7` `kgCO2e/day`, scope `'usage (Homeworking, hors trajet)'`, source `'DEFRA/DESNZ (Homeworking)'`.
- **Application des forfaits.** `co2_calculator.rs:87-98` : si `settings.count_building_energy`, le jour `Office` ajoute le facteur `office_day`, le jour `Remote` ajoute `home_day` ; sinon `0.0`. Constantes `OFFICE_DAY_MODE_ID = "office_day"` et `HOME_DAY_MODE_ID = "home_day"` (`co2_calculator.rs:26-27`). Le total jour est `(trips_total + building).max(0.0)` (`co2_calculator.rs:100-104`).
- **Pas de notion de méthode Scope 2.** L'entité `Co2Settings` (`co2_settings.rs:5-19`) ne contient que `grid_country`, `default_car_occupancy`, `include_radiative_forcing`, `count_building_energy`, `working_days_per_year`, `factor_year`. Aucun champ `building_scope2_method`.
- **Pas de variante market-based.** La seule table de variantes est `emission_factor_grid_variant(mode_id, year, country, value)` (`0004_add_co2.sql:32-39`), utilisée **uniquement** pour `car_ev` (remplacement de la valeur de base par pays, `co2_calculator.rs:130-133`). Rien pour les modes `building`.
- **Entité facteur.** `EmissionFactor` (`emission_factor.rs:82-96`) : `{ id, label, value, unit, category, is_param, scope: Option<String>, source: Option<String> }`. Pas de colonne méthode Scope 2.
- **Settings persistés.** Table `profile_settings` (`0010_add_profile_settings.sql:7-33`), colonnes CO2 : `grid_country` (CHECK `FR|BE|DE|EU`), `default_car_occupancy`, `include_radiative_forcing`, `count_building_energy`, `working_days_per_year`, `factor_year`. Le mapping ligne→entité est dans `profile_settings_repository.rs:34-52` (constante `COLUMNS`, `profile_settings_repository.rs:13-16`), et le `INSERT … ON CONFLICT` dans `profile_settings_repository.rs:69-103`.
- **Settings front.** `Co2Config` (`src/features/profile-settings/domain/entities/profile-settings.ts:5-12`) et `DEFAULT_PROFILE_SETTINGS` (lignes 33-45) reflètent ces colonnes en camelCase. UI dans `src/features/settings/presentation/pages/settings-page.tsx`.
- **Export ODS.** `ods_writer.rs` produit 4 feuilles. La feuille « Présences » (`build_presences_sheet`, `ods_writer.rs:100-170`) écrit une seule colonne CO₂ (`p.co2_kg`, ligne 144) — total jour dénormalisé, qui **inclut déjà** la part bâtiment. La part bâtiment n'est jamais isolée. `ExportData`/`ExportOptions`/`ExportLabels` sont définis dans `spreadsheet_exporter.rs` ; l'exporter ne touche pas les repositories (les données sont pré-rassemblées par `export_profile_data.rs`).
- **Dernière migration vault connue : `0011`** (confirmé par `ls migrations/vault/` → `0001`..`0011`). La prochaine est `0012`.

Conséquence réglementaire : impossible de produire le double affichage Scope 2 exigé par l'ESRS E1-6. La seule valeur disponible est location-based implicite, non qualifiée.

---

## 3. Travail à réaliser

Tâches ordonnées. Le pré-requis Lot 1 fournit le champ `ghg_scope` sur `emission_factor` : ce lot suppose que `office_day`/`home_day` sont déjà classés Scope 2 (électricité) par le Lot 1. Si le Lot 1 ajoute aussi une notion Scope 1 (combustion sur site, ex. chaudière gaz), ce lot reste compatible (cf. §5, note Scope 1).

- **T1 — Migration `0012` (nouveau millésime `building` market-based, R9).**
  Créer `src-tauri/migrations/vault/0012_add_scope2_building.sql`. Cette migration :
  1. Ajoute la colonne `scope2_method` à `emission_factor` (TEXT NULL, CHECK ∈ `'location'|'market'`).
  2. Ajoute la colonne `building_scope2_method` à `profile_settings` (TEXT, défaut `'location'`, CHECK ∈ `'location'|'market'`), avec backfill implicite via le défaut (rétrocompat : tous les profils existants restent location-based).
  3. **Sans toucher 2025/2026 existants**, seede les variantes *market-based* du bâtiment pour le millésime **2026** : nouveaux `id` distincts (`office_day_market`, `home_day_market`) avec `scope2_method='market'`, et **tag** les facteurs `office_day`/`home_day` 2026 existants comme `scope2_method='location'` via un `UPDATE … WHERE year=2026 AND id IN ('office_day','home_day')` (mise à jour d'un **champ de classification**, pas de la valeur `value` — donc R9 préservé, cf. §8).
  Voir le DDL exact en §4.
- **T2 — Entité `EmissionFactor` : champ `scope2_method`.**
  Modifier `src-tauri/src/domain/entities/emission_factor.rs` : ajouter `pub scope2_method: Option<BuildingScope2Method>` à la struct (lignes 82-96) et définir l'enum `BuildingScope2Method { Location, Market }` avec `as_str()`/`parse()` (mêmes conventions que `EmissionUnit`/`EmissionCategory`). Mettre à jour le helper de test `ef(...)` (`co2_calculator.rs:170-182`) pour fournir `scope2_method: None`.
- **T3 — Entité `Co2Settings` : champ `building_scope2_method`.**
  Modifier `src-tauri/src/domain/entities/co2_settings.rs` : ajouter `pub building_scope2_method: BuildingScope2Method` (réutiliser l'enum de T2) ; défaut `Location` dans l'impl `Default` (`co2_settings.rs:21-32`).
- **T4 — Calculateur : sélection du facteur bâtiment selon la méthode.**
  Modifier `src-tauri/src/domain/services/co2_calculator.rs` (`compute_day`, lignes 87-98). Quand `count_building_energy`, choisir le `mode_id` bâtiment selon `settings.building_scope2_method` :
  - `Location` → `office_day` / `home_day` (comportement actuel inchangé — défaut).
  - `Market` → `office_day_market` / `home_day_market` ; **fallback** sur la variante location si la variante market est absente du référentiel (pas de panique, jour reste calculable). Ajouter constantes `OFFICE_DAY_MARKET_MODE_ID = "office_day_market"` et `HOME_DAY_MARKET_MODE_ID = "home_day_market"`.
  La part « trajets » est inchangée. **Important :** la double publication (location ET market simultanément) est portée par **l'export** (T7), pas par le total jour figé : `presence.co2_kg` reste un scalaire unique (la méthode active du profil). L'export recalcule l'autre méthode pour affichage (cf. §5, §8).
- **T5 — Persistance settings.**
  Modifier `src-tauri/src/infrastructure/persistence/profile_settings_repository.rs` : ajouter `building_scope2_method` à la constante `COLUMNS` (ligne 14-16), au mapping `row_to_settings` (lignes 39-51), et au `INSERT … ON CONFLICT … DO UPDATE` (lignes 72-98). Sérialiser via `as_str()` (TEXT).
- **T6 — Settings front (DTO + UI).**
  - `src/features/profile-settings/domain/entities/profile-settings.ts` : ajouter `buildingScope2Method: "location" | "market"` à `Co2Config` (lignes 5-12) et à `DEFAULT_PROFILE_SETTINGS.co2` (défaut `"location"`, lignes 37-44).
  - `src/features/profile-settings/data/dto/profile-settings.dto.ts` : ajouter le champ au wire shape (camelCase).
  - `src/features/settings/presentation/pages/settings-page.tsx` : ajouter un sélecteur « Méthode Scope 2 (électricité bâtiment) » : *Localisation réseau (location-based)* / *Contractuel fournisseur (market-based)*. **Gated** par `countBuildingEnergy` (sinon le réglage est inerte).
  - i18n : ajouter les clés `settings.co2.scope2Method.*` dans `src/core/i18n/locales/fr/translation.json` ET `…/en/translation.json` (sinon export/UI cassés, cf. §8).
- **T7 — Export ODS : double colonne Scope 2 bâtiment.**
  - `spreadsheet_exporter.rs` : enrichir `ExportData`/`DayExport` (lignes 29-42) pour porter, **par jour office/remote**, la part bâtiment location ET market (deux `Option<f64>`), résolues à l'export. Ajouter à `ExportLabels` (lignes 48-90) : `building_energy_location` (« Énergie bâtiment — Scope 2 location-based (kg) »), `building_energy_market` (« Énergie bâtiment — Scope 2 market-based (kg) »), `scope2_method` (« Méthode Scope 2 »). Ajouter à `ExportOptions` un flag `include_scope2_building: bool`.
  - `export_profile_data.rs` : charger `Co2Settings` du profil + les facteurs du `factor_year` (via `load_factor_maps`, `factor_maps.rs:13-30`), et pour chaque jour `office`/`remote` calculer les deux composantes (`office_day`/`office_day_market` ou `home_day`/`home_day_market`) en réutilisant la même logique de fallback que T4. N+1 évité : charger les maps une seule fois par année (cf. §8 perf).
  - `ods_writer.rs` : dans `build_presences_sheet` (lignes 100-170), si `options.include_scope2_building`, ajouter deux colonnes (location, market) après la colonne CO₂.
- **T8 — Documentation méthodologie.**
  Mettre à jour `documentation/calcul-impact-co2.md` : nouvelle sous-section « Scope 2 location-based vs market-based » (rappel ESRS E1-6, défaut location, fallback, R9). Mettre à jour la page in-app méthodologie (`src/features/methodology/presentation/pages/methodology-page.tsx`) pour distinguer les deux variantes bâtiment si le millésime les expose (optionnel mais recommandé pour la traçabilité gap f/Lot 5).

---

## 4. Modèle de données / migrations

Nouvelle migration : `src-tauri/migrations/vault/0012_add_scope2_building.sql`.

> **Note R9 :** SQLite (libSQL) ne supporte pas `ALTER TABLE … ADD CONSTRAINT`. On ajoute donc les colonnes avec leur CHECK directement dans `ADD COLUMN`. Les nouveaux `id` `office_day_market`/`home_day_market` sont seedés pour **2026 uniquement** (le millésime par défaut). Aucun `value` du référentiel 2025/2026 existant n'est modifié.

```sql
-- 0012_add_scope2_building.sql
-- Scope 2 location-based / market-based pour l'énergie bâtiment (gap h, ESRS E1-6).
-- POURQUOI : l'ESRS E1-6 exige la double publication Scope 2 (mix réseau location-based
-- ET instruments contractuels market-based). Les forfaits office_day/home_day actuels
-- sont location-based implicites. On ajoute des variantes market-based en NOUVEAUX id
-- pour le millésime 2026, on tague la METHODE des forfaits existants (champ de
-- classification, jamais leur `value`), et on ajoute le réglage profil. R9 préservé :
-- aucune valeur d'un millésime livré n'est modifiée ; les jours figés restent intacts.

-- 1) Classification de méthode Scope 2 sur le référentiel.
ALTER TABLE emission_factor
  ADD COLUMN scope2_method TEXT DEFAULT NULL
    CHECK (scope2_method IS NULL OR scope2_method IN ('location','market'));

-- 2) Réglage par profil (défaut location-based = rétrocompat).
ALTER TABLE profile_settings
  ADD COLUMN building_scope2_method TEXT NOT NULL DEFAULT 'location'
    CHECK (building_scope2_method IN ('location','market'));

-- 3) Tag des forfaits bâtiment EXISTANTS 2026 comme location-based.
--    (Met à jour la CLASSIFICATION, pas `value` -> R9 OK.)
UPDATE emission_factor
   SET scope2_method = 'location'
 WHERE year = 2026 AND id IN ('office_day','home_day');

-- 4) Seed des variantes market-based 2026 (nouveaux id distincts).
--    `value` par défaut = facteur résiduel "électricité contractuelle". Mettre une
--    valeur conservatrice tant que le profil n'a pas saisi son propre facteur
--    fournisseur ; documenter clairement la source. Exemple ci-dessous : électricité
--    verte ~0 hors énergie non-élec du bâtiment, à ajuster selon la source retenue.
INSERT OR IGNORE INTO emission_factor
  (id, year, label, value, unit, category, scope, is_param, source, scope2_method) VALUES
 ('office_day_market', 2026, 'Jour au bureau (énergie bâtiment, market-based)', 3.5,
    'kgCO2e/day', 'building',
    'usage (bâtiment, market-based, hors trajet)', 1,
    'Facteur contractuel fournisseur (à paramétrer par profil)', 'market'),
 ('home_day_market', 2026, 'Jour télétravail (énergie domicile, market-based)', 2.7,
    'kgCO2e/day', 'building',
    'usage (Homeworking, market-based, hors trajet)', 1,
    'Facteur contractuel fournisseur (à paramétrer par profil)', 'market');
```

> **Sur la valeur market-based seedée :** les variantes market-based démarrent **égales** aux forfaits location (3,5 / 2,7) afin de ne pas sous-estimer par défaut (un profil sans contrat vert ne doit pas afficher 0). `is_param = 1` signale que ces facteurs sont surchargables (cf. Phase 2 : facteur fournisseur saisi par le profil — hors périmètre strict de ce lot, mais l'infra est en place). Si le client cible CSRD veut un facteur fournisseur réel, il passera par un futur millésime ou une variante paramétrée — **jamais** par un UPDATE d'un `value` livré.

Colonnes ajoutées (récap) :

| Table | Colonne | Type | Défaut | CHECK |
|---|---|---|---|---|
| `emission_factor` | `scope2_method` | TEXT | `NULL` | `NULL` ou `'location'`/`'market'` |
| `profile_settings` | `building_scope2_method` | TEXT | `'location'` | `'location'`/`'market'` |

Nouveaux `id` référentiel (année 2026) : `office_day_market`, `home_day_market`.

---

## 5. Spécification détaillée

### 5.1 Méthode Scope 2 active (calcul figé du jour)

Le `building_scope2_method` du profil sélectionne **quel** forfait bâtiment alimente `presence.co2_kg` au moment du `set_presence` :

| `building_scope2_method` | Jour `Office` → mode_id | Jour `Remote` → mode_id |
|---|---|---|
| `location` (défaut) | `office_day` | `home_day` |
| `market` | `office_day_market` (fallback `office_day`) | `home_day_market` (fallback `home_day`) |

Règle de fallback (T4) : si le mode market-based est absent du référentiel du `factor_year` choisi (ex. profil sur `factor_year=2025`, qui n'a pas de variante market), on retombe sur le mode location correspondant. Le calcul ne panique jamais et le jour reste calculable (cohérent avec R6/R8).

Le total jour reste : `total_kg = (Σ co2_trajet + building).max(0.0)` (`co2_calculator.rs:100-104`), inchangé hormis la sélection du `building`.

### 5.2 Double publication (export)

L'export calcule **les deux** composantes bâtiment pour chaque jour `office`/`remote` éligible (building energy activé), indépendamment de la méthode active du profil :

- `building_location = factor(office_day|home_day).value` (selon le type de jour)
- `building_market   = factor(office_day_market|home_day_market).value`, fallback `building_location` si absent.

Edge cases :

| Cas | `building_location` | `building_market` |
|---|---|---|
| `count_building_energy = false` | vide (—) | vide (—) |
| Jour `vacation`/`holiday` | vide (R7 : pas d'énergie bâtiment) | vide |
| `factor_year` sans variante market | valeur location | = valeur location (fallback) |
| Profil en méthode `market` | valeur location (recalculée) | valeur market (figée dans le total) |

Ainsi un auditeur ESRS lit, sur la feuille « Présences », la composante Scope 2 du bâtiment dans **les deux** conventions, plus la colonne « Méthode Scope 2 » indiquant laquelle est intégrée au total `co2_kg` du jour.

### 5.3 Note Scope 1 (combustion sur site)

Le brief mentionne de distinguer le Scope 1 « si pertinent ». Dans le modèle actuel l'énergie bâtiment est un forfait électricité (Scope 2). Une combustion **directe** sur site (chaudière gaz/fioul) relèverait du Scope 1. Ce lot **ne crée pas** de mode Scope 1 dédié (hors périmètre, dépend du découpage énergétique amené par le Lot 1) : il se limite à classer `office_day`/`home_day` en Scope 2. Si un facteur de combustion directe est introduit plus tard, il portera `ghg_scope='1'` (Lot 1) et `scope2_method=NULL` (non concerné). Documenter ce choix en §8 / `calcul-impact-co2.md`.

### 5.4 Formats de colonnes export (feuille « Présences »)

Ordre des colonnes après ajout (avec `include_carbon` ET `include_scope2_building`) :

`Date | Type | CO₂ (kg) | Estimé | Énergie bâtiment — S2 location (kg) | Énergie bâtiment — S2 market (kg) | Méthode Scope 2 | [Heures] | [Créé le | Modifié le]`

Valeurs numériques en kgCO2e (réels, pas de conversion tCO2e ici). « Méthode Scope 2 » = libellé i18n de `location`/`market`.

---

## 6. Critères d'acceptation

- **CA1 — Migration appliquée.** Après migration `0012`, `emission_factor` possède `scope2_method` et `profile_settings` possède `building_scope2_method` (défaut `'location'`). Les `value` de tous les facteurs 2025 et 2026 préexistants sont **inchangés**.
- **CA2 — Variantes market seedées.** `office_day_market` et `home_day_market` existent pour `year=2026` avec `value=3.5`/`2.7`, `category='building'`, `scope2_method='market'`. `office_day`/`home_day` 2026 portent `scope2_method='location'`.
- **CA3 — Rétrocompat calcul.** Quand `building_scope2_method='location'` (défaut), `Co2Calculator::compute_day` produit exactement les mêmes totaux qu'avant ce lot (tous les tests `ac1`..`ac10` et `building_energy_only_when_enabled_and_office_or_remote` passent sans modification de valeur attendue).
- **CA4 — Bascule market.** Quand `building_scope2_method='market'` et `count_building_energy=true`, un jour `Office` sans trajet utilise `office_day_market` ; un jour `Remote` sans trajet utilise `home_day_market`. Si la variante market est absente du référentiel, le fallback location s'applique sans erreur.
- **CA5 — Figeage R9.** Un jour encodé avant migration (location) conserve son `presence.co2_kg`/`presence_trip.co2_kg` figé et son `factor_year` ; aucune ré-écriture rétroactive lors de la migration `0012`.
- **CA6 — Double sortie export.** Quand `include_scope2_building=true` et `count_building_energy=true`, la feuille « Présences » montre, pour chaque jour `office`/`remote`, la part bâtiment **location** ET **market** et la « Méthode Scope 2 » active. Jours `vacation`/`holiday` : cellules vides.
- **CA7 — Persistance settings.** Sauver puis recharger un profil avec `building_scope2_method='market'` restitue bien `'market'` (round-trip repo).
- **CA8 — i18n.** Les clés `settings.co2.scope2Method.*` et les libellés d'export existent en `fr` ET `en` ; aucun libellé vide à l'export.
- **CA9 — Conformité ESRS E1-6.** L'export permet de lire la valeur Scope 2 électricité bâtiment dans les deux conventions (location-based et market-based) sur la même feuille — exigence de double publication satisfaite.

---

## 7. Tests à écrire / mettre à jour

**Rust — `src-tauri/src/domain/services/co2_calculator.rs` (module `tests`)**

- `building_energy_market_method_uses_market_factor` : seeder `office_day_market`/`home_day_market` dans `seed_factors()`, régler `Co2Settings { count_building_energy: true, building_scope2_method: Market, .. }`, vérifier que `Office`/`Remote` prennent les valeurs market.
- `building_energy_market_falls_back_to_location_when_missing` : ne PAS seeder les variantes market, méthode `Market` → le total doit égaler le forfait location (3,5 / 2,7).
- Mettre à jour le helper `ef(...)` (`co2_calculator.rs:170-182`) → ajouter `scope2_method: None`.
- Mettre à jour `Co2Settings::default()` usages dans les tests (le champ a un défaut, donc `..Default::default()` reste valide).
- Vérifier que `building_energy_only_when_enabled_and_office_or_remote` (lignes 440-466) et `ac8_week_with_building_energy` (lignes 337-355) restent verts **sans changer les valeurs attendues** (CA3).

**Rust — `src-tauri/src/infrastructure/persistence/profile_settings_repository.rs` (ou `integration_tests.rs`)**

- `profile_settings_roundtrip_scope2_method` : `save` avec `building_scope2_method = Market`, `load`, assert `Market` (CA7).

**Rust — `src-tauri/src/infrastructure/export/ods_writer.rs` (module `tests`)**

- Étendre `sample_data()`/`sample_labels()`/`all_options()` (lignes 285-368) pour porter les nouveaux champs.
- `scope2_building_columns_written` : avec `include_scope2_building=true`, ré-ouvrir l'ODS via `calamine` et assert la présence des en-têtes location/market et de la colonne « Méthode Scope 2 ».

**Frontend — vitest**

- Test du mapper `profile-settings.dto.ts` ↔ entité : round-trip de `buildingScope2Method`.
- Test settings-page (si couvert) : le sélecteur Scope 2 n'est visible/actif que si `countBuildingEnergy` est coché.

**Commandes**

```bash
# Backend
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
# Front
pnpm typecheck && pnpm test
```

---

## 8. Points d'attention & pièges

- **R9 — figeage des jours (CRITIQUE).** La migration `0012` ne modifie **jamais** un `value` d'un millésime livré. Le `UPDATE … SET scope2_method='location'` ne touche qu'un champ de **classification** : les footprints figés (`presence_trip.co2_kg` + `factor_year`) restent reproductibles. Les variantes market sont des **nouveaux `id`** pour 2026. Ne jamais rétro-affecter les jours existants.
- **Double publication ≠ double comptage.** L'export montre location ET market côte à côte, mais le **total jour** (`presence.co2_kg`) n'intègre qu'**une** méthode (celle du profil). Ne pas sommer les deux dans un total — ce serait un double comptage. La colonne « Méthode Scope 2 » lève l'ambiguïté pour l'auditeur.
- **Fallback silencieux.** Si `factor_year` ne contient pas la variante market (ex. profils restés sur 2025), le calcul retombe sur location. Documenter ce comportement dans `calcul-impact-co2.md` pour ne pas faire croire à un market réel. Idéalement signaler (UI/export) que la valeur market = location par défaut.
- **Couches DDD.** Respecter : enum `BuildingScope2Method` et champs dans `domain/entities` ; sélection dans `domain/services/co2_calculator.rs` (pur, sans I/O) ; chargement/seed dans `infrastructure` + `migrations` ; sérialisation DTO au boundary. Le calculateur ne lit jamais la DB (les maps lui sont passées, cf. `factor_maps.rs`).
- **Perf / N+1.** À l'export, charger les facteurs **une seule fois** par `factor_year` (réutiliser `load_factor_maps`, `factor_maps.rs:13-30`) et résoudre les composantes bâtiment en mémoire. Ne pas requêter par jour.
- **i18n obligatoire.** L'exporter n'a aucune chaîne en dur : les nouveaux libellés (location/market, méthode) viennent de `ExportLabels` (front, déjà traduits). Ajouter les clés en `fr` ET `en` ; une clé manquante casse l'export (libellé vide). Idem pour le sélecteur settings.
- **Non-régression défaut.** Le défaut `location` garantit que tout profil existant et tout test actuel produisent des totaux identiques. Vérifier `ac8` et `building_energy_only_when_enabled_and_office_or_remote` (valeurs 3,5 / 2,7 inchangées).
- **CHECK SQLite.** libSQL n'accepte pas `ADD CONSTRAINT` ; mettre le CHECK dans le `ADD COLUMN`. `INSERT OR IGNORE` pour idempotence du seed (convention migrations).
- **Mouvance réglementaire / priorité.** Lot P3 par défaut, **à remonter en P1** si le client cible est assujetti CSRD : l'ESRS E1-6 rend la double publication Scope 2 obligatoire, pas optionnelle. Le facteur market-based réel (garanties d'origine, mix résiduel fournisseur) devra être paramétré sérieusement à ce moment-là — via un nouveau millésime ou une variante paramétrée, jamais par édition d'un facteur livré.
- **Scope 1 hors périmètre.** Ne pas confondre ce lot (Scope 2 électricité bâtiment) avec une éventuelle combustion directe (Scope 1). Laisser `scope2_method=NULL` sur les facteurs non électriques.

---

## 9. Références réglementaires

- **ESRS E1-6 (CSRD)** — *Gross Scopes 1, 2, 3 and Total GHG emissions* : exige la publication du **Scope 2 location-based ET market-based** (double reporting). Règlement délégué (UE) 2023/2772, annexe (ESRS E1), datapoint E1-6.
  https://eur-lex.europa.eu/legal-content/FR/TXT/?uri=OJ:L_202302772
- **GHG Protocol — Scope 2 Guidance (2015)** : définit les deux méthodes (*location-based* = mix moyen du réseau ; *market-based* = instruments contractuels, garanties d'origine), et impose le dual reporting de l'électricité achetée.
  https://ghgprotocol.org/scope-2-guidance
- **BEGES / Bilan GES réglementaire (France, art. L229-25 Code de l'environnement)** — postes énergie : cadre pour la classification Scope 1/2/3 vers laquelle s'aligne le champ `ghg_scope` (Lot 1) ; le présent lot prépare la composante Scope 2 électricité.
  https://www.bilans-ges.ademe.fr/
- **Garanties d'origine / mix résiduel** (pour le facteur market-based, paramétrage Phase 2) — AIB European Residual Mix ; pour la Belgique : Bruxelles-Environnement / SPW Énergie.
  https://www.aib-net.org/facts/european-residual-mix
- **ADEME Base Carbone / Impact CO2** (source des forfaits location-based actuels `office_day`, et cohérence générale) — https://base-empreinte.ademe.fr/
