# Lot 1 — Mapping Scope GHG / poste BEGES / catégorie ESRS

> **Statut :** À faire  ·  **Priorité :** P1  ·  **Effort :** moyen  ·  **Dépend de :** aucun (fondation)
> **Gap couvert :** (a) attacher à chaque facteur une classification réglementaire normalisée (Scope GHG, poste BEGES, catégorie ESRS) et lever l'ambiguïté du champ `scope` (périmètre cycle de vie ≠ Scope GHG Protocol).
> **Débloque :** BEGES V5 (Poste 3.3 « Déplacements domicile-travail », Catégorie 3 « Émissions indirectes associées au transport ») · GHG Protocol Corporate Standard + Scope 3 Standard (catégorie 7 « Employee commuting ») · CSRD/ESRS E1-6 (émissions brutes Scope 1/2/3). Fondation des lots 3 (export auditable), 4 (consolidation par poste) et 8 (Scope 2 location/market-based).

---

> ⚠️ **Numérotation de migration — lire avant de coder.** Ce lot crée la migration vault réservée **`0012_add_ghg_mapping.sql`** (cf. [README §5 — Numérotation des migrations](./README.md#numérotation-des-migrations)). Le Lot 1 étant la fondation exécutée en premier, `0012` est normalement libre ; **vérifier malgré tout** : `ls src-tauri/migrations/vault/ | sort | tail -1` — si une migration > `0011` existe déjà, prendre le numéro libre suivant et aligner le champ `version:` dans `migrations.rs`.

## 1. Contexte & objectif

Aujourd'hui BasicPresence calcule une empreinte carbone par jour et par trajet (kgCO2e figé dans `presence_trip`), mais **aucune classification réglementaire** n'est attachée au résultat. Le seul champ « scope » existant (`emission_factor.scope`) désigne en réalité le **périmètre cycle de vie** (well-to-wheel, ex. `usage(WtW)+fabrication`) et **non** le Scope GHG Protocol (1/2/3). Cette homonymie est un risque de déclaration erronée pour un usage entreprise (BEGES, CSRD).

L'objectif du lot est d'**ancrer chaque facteur du référentiel dans une taxonomie réglementaire normalisée** : Scope GHG (1/2/3), catégorie GHG Protocol Scope 3 (cat. 7 pour le domicile-travail), poste BEGES V5 (3.3), correspondance ESRS E1-6 — et de **renommer** le champ ambigu pour le réserver à sa sémantique cycle de vie. C'est la **fondation** : sans cette classification, l'export auditable (lot 3), la consolidation par poste (lot 4) et la distinction Scope 2 location/market-based (lot 8) ne peuvent pas s'appuyer sur des données fiables.

Résultat attendu : chaque ligne de `emission_factor` (millésimes 2025 **et** 2026) porte un `ghg_scope`, un `ghg_category`, un `beges_poste`, un `esrs_datapoint` ; le champ `scope` est renommé `lifecycle_boundary` ; ces métadonnées remontent jusqu'au DTO méthodologie et sont gelées par snapshot lors de l'encodage (R9).

---

## 2. État actuel du code (point de départ)

Faits précis tirés du code (chemins `fichier:ligne`) :

- **Entité domaine** `EmissionFactor` (`src-tauri/src/domain/entities/emission_factor.rs:82-96`) :
  ```
  pub struct EmissionFactor {
      pub id: String,            // mode_id, ex. "car_petrol"
      pub label: String,
      pub value: f64,
      pub unit: EmissionUnit,    // VehKm | PassengerKm | Km | Day
      pub category: EmissionCategory, // Car|Active|PublicTransport|Rail|Air|Building
      pub is_param: bool,
      pub scope: Option<String>, // PÉRIMÈTRE cycle de vie (well-to-wheel), PAS Scope GHG
      pub source: Option<String>,
  }
  ```
  Le doc-comment ligne 92 dit explicitement : `Perimeter of the factor (e.g. "usage+fabrication"), for traceability.` → c'est bien un périmètre ACV.

- **Enums** `EmissionUnit` (`emission_factor.rs:5-38`) et `EmissionCategory` (`emission_factor.rs:42-77`) suivent un patron `as_str()` / `parse()` renvoyant `DomainError::Validation` sur valeur inconnue. **Tout nouvel enum réglementaire doit suivre ce même patron.**

- **Schéma SQL** `emission_factor` (`src-tauri/migrations/vault/0004_add_co2.sql:15-28`) :
  ```sql
  CREATE TABLE IF NOT EXISTS emission_factor (
      id TEXT NOT NULL, year INTEGER NOT NULL, label TEXT NOT NULL,
      value REAL NOT NULL,
      unit TEXT NOT NULL CHECK (unit IN ('kgCO2e/veh.km',...,'kgCO2e/day')),
      category TEXT NOT NULL CHECK (category IN ('car','active','public_transport','rail','air','building')),
      scope TEXT,                       -- traceability (usage/amont/fabrication…)
      is_param INTEGER NOT NULL DEFAULT 0,
      source TEXT,
      PRIMARY KEY (id, year)
  );
  ```
  Clé primaire composite `(id, year)`. Les colonnes `scope`/`source` sont du texte libre, **sans CHECK**.

- **Seed 2025** : 26 facteurs `INSERT OR IGNORE` (`0004_add_co2.sql:94-120`) + 4 variantes réseau `car_ev` (`0004_add_co2.sql:125-129`). **Migration livrée, jamais éditée en place (R9).**

- **Seed 2026** : mêmes 26 mode_id, valeurs corrigées (`0011_add_co2_2026.sql:30-56`) + variantes `car_ev` 2026 (`0011_add_co2_2026.sql:60-64`) + bascule du défaut profil `UPDATE profile_settings SET factor_year = 2026 WHERE factor_year = 2025` (`0011_add_co2_2026.sql:72`).

- **Repository** `LibsqlEmissionFactorRepository` (`src-tauri/src/infrastructure/persistence/emission_factor_repository.rs`) : la constante `SELECT_COLUMNS` (ligne 14) liste **explicitement** `"id, label, value, unit, category, is_param, scope, source"` ; `row_to_factor()` (ligne 31-45) lit par **index positionnel** (`row.get(0)` … `row.get(7)`). **Tout ajout de colonne au SELECT décale les index — point de vigilance majeur.**

- **DTO frontend** `EmissionFactorDto` (`src-tauri/src/application/dto/emission_factor_dto.rs:15-27`) : `serde rename_all = "camelCase"`, champs `mode_id, label, value, unit, category, is_param, grid_variants, scope, source`. Conteneur `Co2ReferentialDto` (ligne 32-38) : `factor_year, radiative_forcing, factors`.

- **Use case** `ListCo2ReferentialUseCase::execute()` (`src-tauri/src/application/use_cases/list_co2_referential.rs:26-62`) : charge `factors.list(factor_year)` + `list_grid_variants(factor_year)`, mappe `EmissionFactor` → `EmissionFactorDto` (ligne 39-55, recopie champ à champ `scope: f.scope, source: f.source`).

- **Entité front** `Co2ReferentialFactor` (`src/features/methodology/domain/entities/co2-referential.ts:9-22`) : `modeId, label, value, unit, category, isParam, scope, source, gridVariants`.

- **Page méthodologie** (`src/features/methodology/presentation/pages/methodology-page.tsx`) : tableau par catégorie ; l'en-tête colonne « Périmètre » utilise la clé i18n `methodology.table.scope` (ligne 223-225) et rend `f.scope ?? "—"` (ligne 258-260). Clés i18n méthodologie existantes (`src/core/i18n/locales/fr/translation.json`) : `methodology.{menuItem,title,back,intro,introGeneric,loading,error,noProfile,formula,params,table,examples,sources,transparency}`.

- **Snapshot** `Trip` (`src-tauri/src/domain/entities/trip.rs:16-27`) et table `presence_trip` (`0004_add_co2.sql:73-86`) figent `co2_kg`, `is_estimated`, `factor_year` par trajet. **La classification est résoluble a posteriori via `(mode_id, factor_year)`** : elle vit sur le facteur versionné, donc le snapshot n'a pas besoin de la dupliquer (voir §5, décision d'architecture).

- **Calculateur** `Co2Calculator` (`src-tauri/src/domain/services/co2_calculator.rs`) : pur, ne touche pas à la classification. Building energy résolu via `OFFICE_DAY_MODE_ID = "office_day"` / `HOME_DAY_MODE_ID = "home_day"` (ligne 26-27, 87-98). **Aucune modification du calculateur dans ce lot** (la classification est métadonnée, pas arithmétique).

- **Migrations enregistrées** (`src-tauri/src/infrastructure/persistence/migrations.rs:22-67`) : `VAULT_MIGRATIONS` va jusqu'à la **version 11** (`0011_add_co2_2026.sql`). La prochaine est la **version 12**.

- **Dossier cible** `documentation/conformite/` : existe, vide.

---

## 3. Travail à réaliser

Tâches ordonnées. Chaque tâche cite le(s) fichier(s) exact(s) et l'action concrète.

### T1 — Définir la migration 0012 (renommage + 4 colonnes + valorisation 2025 & 2026)

Créer `src-tauri/migrations/vault/0012_add_ghg_mapping.sql` (nouveau fichier, version 12). Contenu :
1. Renommer la colonne ambiguë : `ALTER TABLE emission_factor RENAME COLUMN scope TO lifecycle_boundary;` (SQLite ≥ 3.25 supporte `RENAME COLUMN` ; libSQL l'embarque).
2. Ajouter 4 colonnes nullable : `ghg_scope`, `ghg_category`, `beges_poste`, `esrs_datapoint` (toutes `TEXT`, `DEFAULT NULL`). Pas de CHECK SQL (la validation se fait au parse domaine, comme `unit`/`category` — cohérence avec l'existant et tolérance aux futurs millésimes).
3. Valoriser **les deux millésimes existants** (2025 et 2026) par des `UPDATE … WHERE year IN (2025, 2026)` selon le mapping du §5. **Important :** valoriser des colonnes de métadonnées de classification n'altère **pas** les `co2_kg` figés dans `presence_trip` → conforme R9 (voir §8).

DDL complet en §4.

### T2 — Enregistrer la migration

Éditer `src-tauri/src/infrastructure/persistence/migrations.rs` : ajouter dans `VAULT_MIGRATIONS` (après l'entrée version 11, ligne 63-66) :
```rust
    Migration {
        version: 12,
        sql: include_str!("../../../migrations/vault/0012_add_ghg_mapping.sql"),
    },
```

### T3 — Créer les enums domaine réglementaires

Dans `src-tauri/src/domain/entities/emission_factor.rs`, ajouter trois enums (ou un module) suivant le patron `as_str()`/`parse()` déjà utilisé par `EmissionUnit`/`EmissionCategory` :
- `GhgScope { Scope1, Scope2, Scope3 }` → `as_str()` rend `"1"|"2"|"3"`.
- `GhgScope3Category` (catégorie GHG Protocol Scope 3) → au minimum la variante `Cat7` (`"7"`, Employee commuting). Prévoir l'extensibilité, mais ne pas sur-modéliser : ce lot ne couvre que le domicile-travail + énergie bâtiment.
- `BegesPoste` → au minimum `P3_3` (`"3.3"`). Le bâtiment relève de postes Scope 1/2 distincts (voir §5) ; modéliser `P1_x` / `P2_x` selon le tableau §5.

Pour `esrs_datapoint` : une simple `Option<String>` (valeur libre `"E1-6"`) suffit — pas besoin d'enum, c'est un identifiant de point de donnée ESRS qui peut évoluer. Justifié pour ne pas figer une nomenclature ESRS encore mouvante.

### T4 — Étendre la struct `EmissionFactor`

Dans `src-tauri/src/domain/entities/emission_factor.rs:82-96` :
- **Renommer** le champ `pub scope: Option<String>` → `pub lifecycle_boundary: Option<String>` et mettre à jour son doc-comment (ligne 92) pour dire clairement « périmètre cycle de vie / ACV (well-to-wheel) — N'EST PAS le Scope GHG Protocol ».
- **Ajouter** : `pub ghg_scope: Option<GhgScope>`, `pub ghg_category: Option<GhgScope3Category>`, `pub beges_poste: Option<BegesPoste>`, `pub esrs_datapoint: Option<String>`.

Mettre à jour les usages : le fixture de test `ef(...)` (`co2_calculator.rs:170-182`) construit un `EmissionFactor` avec `scope: None` → renommer en `lifecycle_boundary: None` et ajouter les 4 nouveaux champs à `None`.

### T5 — Adapter le repository (lecture)

Dans `src-tauri/src/infrastructure/persistence/emission_factor_repository.rs` :
- Étendre `SELECT_COLUMNS` (ligne 14) :
  `"id, label, value, unit, category, is_param, lifecycle_boundary, ghg_scope, ghg_category, beges_poste, esrs_datapoint, source"`.
  (Note : `scope` devient `lifecycle_boundary` ; on regroupe les 4 nouvelles colonnes juste après pour limiter le décalage, et on garde `source` en dernier.)
- Adapter `row_to_factor()` (ligne 31-45) aux **nouveaux index positionnels** : `lifecycle_boundary = row.get(6)`, `ghg_scope = row.get(7)` (parser via `GhgScope::parse` après lecture en `Option<String>`), `ghg_category = row.get(8)`, `beges_poste = row.get(9)`, `esrs_datapoint = row.get(10)`, `source = row.get(11)`. Pour les enums optionnels : lire `Option<String>` puis `.map(|s| GhgScope::parse(&s)).transpose()?`.

### T6 — Étendre le DTO méthodologie

Dans `src-tauri/src/application/dto/emission_factor_dto.rs:15-27` :
- Renommer `pub scope: Option<String>` → `pub lifecycle_boundary: Option<String>` (sérialisé `lifecycleBoundary`).
- Ajouter `pub ghg_scope: Option<String>` (sérialisé `ghgScope`), `pub ghg_category: Option<String>` (`ghgCategory`), `pub beges_poste: Option<String>` (`begesPoste`), `pub esrs_datapoint: Option<String>` (`esrsDatapoint`).

### T7 — Adapter le use case `ListCo2Referential`

Dans `src-tauri/src/application/use_cases/list_co2_referential.rs:39-55`, dans le `.map(|f| …)` : remplacer `scope: f.scope` par `lifecycle_boundary: f.lifecycle_boundary` et ajouter le mapping des 4 nouveaux champs en convertissant les enums en `String` via `as_str()` (`f.ghg_scope.map(|s| s.as_str().to_string())`, etc.) et `esrs_datapoint: f.esrs_datapoint`.

### T8 — Étendre l'entité frontend & l'affichage méthodologie

- `src/features/methodology/domain/entities/co2-referential.ts:9-22` : renommer `scope` → `lifecycleBoundary` ; ajouter `ghgScope: string | null`, `ghgCategory: string | null`, `begesPoste: string | null`, `esrsDatapoint: string | null`.
- Mettre à jour le mapper data front (la repository Tauri qui désérialise `Co2ReferentialDto`) pour porter les nouveaux champs.
- `methodology-page.tsx` : la colonne actuelle « Périmètre » (en-tête `methodology.table.scope`, ligne 223-225, rendu `f.scope`, ligne 258-260) doit **rester** mais lire `f.lifecycleBoundary`. **Ajouter** une colonne « Scope GHG » affichant `f.ghgScope` (et optionnellement poste BEGES). Cela matérialise la levée d'ambiguïté pour l'utilisateur : deux colonnes distinctes (« Périmètre (ACV) » vs « Scope GHG »).

### T9 — i18n

Dans `src/core/i18n/locales/fr/translation.json` et `…/en/translation.json`, sous `methodology.table` : renommer/clarifier la clé `scope` (libellé FR « Périmètre (ACV) ») et ajouter `ghgScope` (FR « Scope GHG »), éventuellement `begesPoste` (FR « Poste BEGES »). **Les deux langues doivent être à jour** (sinon export/rendu vide).

### T10 — Documentation

Dans `documentation/calcul-impact-co2.md`, ajouter une section (ex. §6 bis ou §11) avec le **tableau de mapping** `mode_id → {Scope GHG, cat. GHG, poste BEGES, ESRS}` (le tableau du §5 ci-dessous) et expliciter que `scope` a été renommé `lifecycle_boundary`. Justifier le rattachement obligatoire du domicile-travail au **Poste 3.3 / Scope 3 cat. 7** (et non cat. 4).

---

## 4. Modèle de données / migrations

**Nouvelle migration :** `src-tauri/migrations/vault/0012_add_ghg_mapping.sql` (version 12).

```sql
-- Mapping réglementaire des facteurs d'émission (gap a).
--
-- POURQUOI : le champ `scope` actuel désigne le PÉRIMÈTRE cycle de vie
-- (well-to-wheel, ex. "usage(WtW)+fabrication") et NON le Scope GHG Protocol.
-- On le renomme en `lifecycle_boundary` pour lever l'ambiguïté, puis on ajoute
-- la classification réglementaire normalisée par facteur.
--
-- R9 : on ne touche QUE des métadonnées de classification. Les co2_kg figés
-- dans presence_trip ne sont pas modifiés ; valoriser ces colonnes sur les
-- millésimes 2025 ET 2026 ne casse aucun footprint passé (les valeurs et
-- factor_year restent intacts).

-- 1. Lever l'ambiguïté : scope (périmètre ACV) -> lifecycle_boundary.
ALTER TABLE emission_factor RENAME COLUMN scope TO lifecycle_boundary;

-- 2. Classification réglementaire (nullable, validée au parse domaine).
ALTER TABLE emission_factor ADD COLUMN ghg_scope      TEXT DEFAULT NULL; -- '1' | '2' | '3'
ALTER TABLE emission_factor ADD COLUMN ghg_category   TEXT DEFAULT NULL; -- cat. Scope 3 GHG Protocol, ex. '7'
ALTER TABLE emission_factor ADD COLUMN beges_poste    TEXT DEFAULT NULL; -- poste BEGES V5, ex. '3.3'
ALTER TABLE emission_factor ADD COLUMN esrs_datapoint TEXT DEFAULT NULL; -- ex. 'E1-6'

-- 3. Valorisation — TRANSPORT (Scope 3, cat. 7, Poste 3.3, ESRS E1-6).
--    Tous les modes de déplacement domicile-travail (voiture, actif, transports
--    en commun, rail, aérien) relèvent OBLIGATOIREMENT du Poste 3.3 BEGES V5
--    « Déplacements domicile-travail » / Scope 3 cat. 7 « Employee commuting ».
UPDATE emission_factor
   SET ghg_scope = '3', ghg_category = '7', beges_poste = '3.3', esrs_datapoint = 'E1-6'
 WHERE year IN (2025, 2026)
   AND category IN ('car','active','public_transport','rail','air');

-- 4. Valorisation — ÉNERGIE BÂTIMENT.
--    office_day / home_day = énergie bâtiment/domicile. À ce stade NON ventilé
--    Scope 1 (combustion sur site : gaz/fioul) vs Scope 2 (élec/chaleur/vapeur
--    achetées) : le facteur agrégé est un proxy. On le classe par défaut Scope 2
--    (électricité/chaleur, location-based implicite), poste BEGES 2.x ;
--    la ventilation fine 1 vs 2 est traitée au lot 8 (location/market-based).
UPDATE emission_factor
   SET ghg_scope = '2', ghg_category = NULL, beges_poste = '2.1', esrs_datapoint = 'E1-6'
 WHERE year IN (2025, 2026)
   AND category = 'building';
```

> Remarque migration : SQLite/libSQL exécute chaque `ALTER TABLE` séparément ; `execute_batch` (utilisé par `migrations::run`, `migrations.rs:95`) découpe le fichier en instructions. `RENAME COLUMN` est supporté par SQLite ≥ 3.25. Si une contrainte d'environnement l'interdisait, le repli serait : ajouter `lifecycle_boundary`, `UPDATE … SET lifecycle_boundary = scope`, laisser `scope` (déprécié, non lu). **Préférer le `RENAME COLUMN`** car le SELECT positionnel rend la coexistence des deux colonnes fragile.

---

## 5. Spécification détaillée

### 5.1 Décision d'architecture : colonnes sur `emission_factor` (et non table de mapping séparée)

**Décision : enrichir `emission_factor` de 4 colonnes**, plutôt qu'une table `ghg_mapping(mode_id, …)` séparée.

Justification :
- La classification est **intrinsèquement versionnée par millésime** : un même `mode_id` pourrait, en théorie, changer de poste BEGES si la nomenclature évolue (BEGES V5 → V6). La clé `(id, year)` de `emission_factor` porte déjà ce versionnement. Une table séparée devrait répliquer `(mode_id, year)` → duplication de la clé et risque d'incohérence.
- Le référentiel est déjà la **source de vérité unique** chargée par `EmissionFactorRepository.list(year)` ; ajouter des colonnes évite un second JOIN/chargement dans le use case et le calculateur en aval (lots 3/4).
- Le snapshot `presence_trip` n'a **pas** besoin de dupliquer la classification : un trajet figé connaît son `(mode_id, factor_year)`, donc sa classification est **résoluble de façon déterministe et reproductible** en relisant `emission_factor`. La classification d'un mode pour une année donnée est **immuable** (mêmes garanties R9 que `value`). → On ne stocke la classification **qu'une fois**, sur le facteur.

### 5.2 Sémantique des champs

| Champ (DB / domaine) | Sérialisé (DTO) | Rôle | Exemples de valeurs |
|---|---|---|---|
| `lifecycle_boundary` (ex-`scope`) | `lifecycleBoundary` | Périmètre **cycle de vie** (ACV / well-to-wheel). Inchangé sémantiquement, juste renommé. | `usage(WtW)+fabrication`, `usage+forçage radiatif (1,7)`, `convention` |
| `ghg_scope` | `ghgScope` | **Scope GHG Protocol** | `1`, `2`, `3` |
| `ghg_category` | `ghgCategory` | Catégorie GHG Protocol **Scope 3** (NULL hors Scope 3) | `7` (Employee commuting) |
| `beges_poste` | `begesPoste` | **Poste BEGES V5** | `3.3`, `2.1` |
| `esrs_datapoint` | `esrsDatapoint` | Point de donnée **ESRS E1** | `E1-6` |

### 5.3 Mapping `mode_id → classification` (millésimes 2025 et 2026)

Tous les modes **transport** (domicile-travail) :

| mode_id (catégorie) | Scope GHG | cat. GHG (Scope 3) | Poste BEGES V5 | ESRS |
|---|---|---|---|---|
| `car_petrol`, `car_diesel`, `car_average`, `car_phev`, `car_ev`, `taxi` (`car`) | **3** | **7** | **3.3** | E1-6 |
| `walk`, `bike`, `ebike`, `escooter`, `scooter_elec` (`active`) | **3** | **7** | **3.3** | E1-6 |
| `public_transport`, `bus`, `coach`, `metro_tram` (`public_transport`) | **3** | **7** | **3.3** | E1-6 |
| `train_sncb`, `train_ter`, `train_hs_fr`, `train_eurostar`, `train_thalys` (`rail`) | **3** | **7** | **3.3** | E1-6 |
| `plane_domestic`, `plane_short`, `plane_medium`, `plane_long` (`air`) | **3** | **7** | **3.3** | E1-6 |

Modes **énergie bâtiment** :

| mode_id (catégorie) | Scope GHG | cat. GHG | Poste BEGES V5 | ESRS |
|---|---|---|---|---|
| `office_day` (`building`) | **2** (par défaut, voir note) | — | **2.1** | E1-6 |
| `home_day` (`building`) | **2** (par défaut, voir note) | — | **2.1** | E1-6 |

### 5.4 Règle réglementaire clé (domicile-travail)

Le **domicile-travail relève obligatoirement** du **Poste 3.3 BEGES V5** (« Déplacements domicile-travail », au sein de la **Catégorie 3** « Émissions indirectes associées au transport ») et de la **catégorie 7 du Scope 3 GHG Protocol** (« Employee commuting »). **Ce n'est PAS la catégorie 4** (« Upstream transportation and distribution »), contrairement à ce qu'indiquent à tort certaines sources commerciales. Cette règle est **non négociable** et doit être documentée dans `calcul-impact-co2.md` (T10).

### 5.5 Énergie bâtiment : Scope 1 vs Scope 2 (limite du lot)

`office_day` / `home_day` agrègent l'énergie du bâtiment/domicile sans distinguer la **combustion sur site** (gaz, fioul → **Scope 1**) de l'**énergie achetée** (électricité, chaleur, vapeur → **Scope 2**). Comme le facteur actuel est **agrégé** et que la ventilation fine exige de **scinder les facteurs** (et d'introduire location-based vs market-based), ce lot **classe par défaut le bâtiment en Scope 2 / poste 2.1** et **renvoie la ventilation 1↔2 et location/market-based au lot 8**. La décision est documentée dans la migration (commentaire SQL §4, point 4) et dans `calcul-impact-co2.md`.

### 5.6 Edge cases

- **Fallback `car_average` (R6)** : un trajet de `mode_id` inconnu retombe sur `car_average` (catégorie `car`) → classification Scope 3 / cat. 7 / poste 3.3. Cohérent (c'est un déplacement). Le flag `is_estimated` reste porté par le snapshot.
- **Jours vacation/holiday** : `co2_kg = NULL`, aucun trajet → pas de classification à exposer (pas d'impact).
- **Anciens jours encodés avant le lot** : leurs `presence_trip` gardent `co2_kg`/`factor_year` ; la classification est résolue à la lecture via `(mode_id, factor_year)` sur `emission_factor` désormais valorisé. **Aucune réécriture des snapshots.**
- **Millésime futur (2027+)** : toute nouvelle migration de seed devra renseigner les 5 colonnes (`lifecycle_boundary` + 4 classification) dans son `INSERT`. À documenter comme convention.

---

## 6. Critères d'acceptation

- **CA1** — Après migration 0012, la colonne `emission_factor.scope` n'existe plus ; `emission_factor.lifecycle_boundary` la remplace avec les mêmes valeurs (2025 et 2026).
- **CA2** — Pour les millésimes 2025 **et** 2026, **tout** facteur de catégorie `car|active|public_transport|rail|air` a `ghg_scope='3'`, `ghg_category='7'`, `beges_poste='3.3'`, `esrs_datapoint='E1-6'`.
- **CA3** — Pour les millésimes 2025 et 2026, `office_day` et `home_day` ont `ghg_scope='2'`, `beges_poste='2.1'`, `esrs_datapoint='E1-6'`.
- **CA4** — `Co2ReferentialDto` (commande `list_co2_referential`) expose `lifecycleBoundary`, `ghgScope`, `ghgCategory`, `begesPoste`, `esrsDatapoint` pour chaque facteur ; les valeurs reflètent CA2/CA3.
- **CA5** — La page méthodologie affiche **deux colonnes distinctes** : « Périmètre (ACV) » (= `lifecycleBoundary`) et « Scope GHG » (= `ghgScope`), prouvant la levée d'ambiguïté.
- **CA6 (R9)** — Un jour encodé **avant** le lot conserve exactement le même `co2_kg` dans `presence_trip` après application de 0012 (la migration ne modifie aucun `co2_kg`/`factor_year`).
- **CA7** — `GhgScope::parse("4")` (valeur invalide) renvoie `DomainError::Validation`, comme `EmissionUnit::parse`/`EmissionCategory::parse`.
- **CA8 (conformité)** — Le domicile-travail est rattaché au Poste 3.3 / Scope 3 cat. 7 (jamais cat. 4) ; vérifiable via CA2 et la doc.
- **CA9** — `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` et `cargo test` passent ; les clés i18n FR et EN sont synchronisées.

---

## 7. Tests à écrire / mettre à jour

**Rust — unitaires (mod `tests` dans `emission_factor.rs`)** :
- `ghg_scope_parse_roundtrip` : `GhgScope::parse(GhgScope::Scope3.as_str()) == Ok(Scope3)` pour les 3 variantes.
- `ghg_scope_parse_rejects_unknown` : `GhgScope::parse("4")` → `Err(DomainError::Validation(_))` (couvre CA7). Idem pour `BegesPoste`/`GhgScope3Category` si modélisés en enum.

**Rust — repository / intégration** (`src-tauri/src/integration_tests.rs`, parcours migrations + lecture) :
- `emission_factor_carries_ghg_mapping` : après migrations, charger le référentiel 2026 et 2025 via `LibsqlEmissionFactorRepository::list(year)` ; asserter qu'un mode transport (ex. `car_petrol`) a `ghg_scope=Some(Scope3)`, `beges_poste=Some(P3_3)`, et qu'un mode bâtiment (`office_day`) a `ghg_scope=Some(Scope2)` (couvre CA2/CA3 sur les deux millésimes).
- `lifecycle_boundary_preserved_after_rename` : la valeur de `lifecycle_boundary` pour `car_petrol` 2025 vaut bien l'ancienne valeur `scope` (`"usage+amont+fabrication"`) (couvre CA1).
- `presence_trip_co2_unchanged_by_0012` (si un harnais d'intégration encode un jour) : encoder un jour, appliquer 0012, relire `presence_trip.co2_kg` → inchangé (couvre CA6/R9). À défaut d'un tel harnais, documenter la garantie par revue (la migration ne contient aucun `UPDATE presence_trip`).

**Rust — calculateur (non-régression)** : `co2_calculator.rs` tests `ac1..ac10` + autres doivent **continuer à passer sans changement de valeurs** (la classification n'entre pas dans l'arithmétique). Seule la construction du fixture `ef(...)` change (renommage de champ + 4 champs `None`).

**Frontend (vitest)** : mettre à jour le test du mapper de `tauri-co2-referential.repository` pour vérifier la désérialisation de `ghgScope`/`lifecycleBoundary`/`begesPoste` ; rendu méthodologie inchangé fonctionnellement (deux colonnes au lieu d'une).

**Commandes** :
```
cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
pnpm test && pnpm typecheck && pnpm lint
```

---

## 8. Points d'attention & pièges

- **R9 / figeage des jours** : la migration ne fait que renommer une colonne et valoriser des métadonnées de classification. Elle **ne touche aucun `co2_kg` ni `factor_year`** de `presence_trip`/`presence`. Toute future correction du *mapping* d'un mode pour une année **livrée** doit suivre la même règle que les facteurs : **nouveau millésime**, jamais d'`UPDATE` rétroactif d'une valeur de calcul. (Le mapping étant immuable par `(mode_id, year)`, le re-lire reste reproductible.)
- **SELECT positionnel** (`emission_factor_repository.rs:14,31-45`) : le décalage d'index est la principale source de bug. Mettre à jour **ensemble** `SELECT_COLUMNS` et les `row.get(N)` ; vérifier chaque index après ajout des 4 colonnes.
- **Renommage = breaking pour le DTO/front** : `scope` disparaît du DTO (devient `lifecycleBoundary`). Mettre à jour **dans le même PR** : DTO Rust, entité TS, mapper data front, page méthodologie, et le bundle import/export s'il référençait `scope` (vérifier `json_bundle_codec.rs` — le bundle ne sérialise normalement pas le référentiel, mais confirmer).
- **i18n FR + EN** : ne jamais ajouter une clé dans une seule langue (rendu vide / export cassé). Renommer le libellé `methodology.table.scope` en « Périmètre (ACV) » et ajouter « Scope GHG ».
- **Couches DDD** : enums réglementaires dans `domain/entities` (zéro dépendance tierce) ; conversion enum→String **uniquement** dans le use case/DTO (frontière application), jamais dans le domaine.
- **Ne pas modifier le calculateur** : la classification est métadonnée pure. Toute modification de `co2_calculator.rs` au-delà du renommage du fixture de test est hors-périmètre (risque de régression `ac1..ac10`).
- **Mouvance réglementaire** : `esrs_datapoint` laissé en texte libre exprès (ESRS encore en consolidation). Le poste BEGES et le Scope GHG sont stables → enums.
- **Bâtiment Scope 1 vs 2** : choix par défaut Scope 2 documenté ; ne **pas** sur-anticiper le lot 8 (pas de scission de facteur ici). Bien noter cette limite dans la doc pour qu'un auditeur comprenne le proxy.
- **Convention futurs millésimes** : documenter que tout nouveau seed `emission_factor` doit renseigner `lifecycle_boundary` + les 4 colonnes de classification.

---

## 9. Références réglementaires

- **BEGES réglementaire V5** (Bilan d'émissions de gaz à effet de serre, art. L229-25 Code de l'environnement) — méthode et nomenclature des postes, Catégorie 3 « Émissions indirectes associées au transport », **Poste 3.3 « Déplacements domicile-travail »** : ADEME / Ministère de la Transition écologique — Bilans GES (https://bilans-ges.ademe.fr) et guide méthodologique BEGES réglementaire V5.
- **GHG Protocol Corporate Standard** et **Corporate Value Chain (Scope 3) Standard** — définition des Scopes 1/2/3 et des 15 catégories Scope 3, dont **Category 7 « Employee commuting »** (https://ghgprotocol.org/standards).
- **GHG Protocol Scope 2 Guidance** — distinction **location-based / market-based** (cadre pour le lot 8, classification bâtiment Scope 2 ici).
- **CSRD / ESRS E1 « Climate change »**, **datapoint E1-6 « Gross Scopes 1, 2, 3 and Total GHG emissions »** — EFRAG, ESRS Set 1 (Règlement délégué (UE) 2023/2772).
- **Cadre belge** (contexte usage entreprise BE) : facteurs et obligations PDE/diagnostic fédéral — référence amont des lots 7 (facteurs belges) ; pour ce lot, la classification Scope/poste s'appuie sur le GHG Protocol (référentiel international commun FR/BE).
- **Documentation interne** : `documentation/calcul-impact-co2.md` (règles R1–R9, formules, sources des facteurs) — à enrichir du tableau de mapping (T10).
