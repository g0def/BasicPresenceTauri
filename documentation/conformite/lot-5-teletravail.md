# Lot 5 — Modélisation du télétravail (réduction poste 3.3 + WFH incrémental optionnel)

> **Statut :** À faire  ·  **Priorité :** P2  ·  **Effort :** moyen  ·  **Dépend de :** Lot 1 (mapping Scope GHG / poste BEGES 3.3 / GHG Protocol cat.7)
> **Gap couvert :** (e) Le télétravail n'est pas modélisé comme réduction du Scope 3 cat.7 / poste BEGES 3.3, ni comme émissions incrémentales WFH optionnelles.
> **Débloque :** BEGES (poste 3.3 « Déplacements domicile-travail » — modulé par les jours télétravaillés), GHG Protocol Scope 3 catégorie 7 « Employee commuting » (le télétravail y est une *donnée d'activité*, avec une option « may include » pour les émissions WFH), ESRS E1-6 (cohérence du périmètre et des hypothèses du Scope 3).

---

> ⚠️ **Numérotation de migration — lire avant de coder.** Ce document écrit `0012` en supposant un dépôt à `0011`, mais **ce lot ne peut pas garder ce numéro** : il entre en collision avec le Lot 1. Migration **réservée par le plan : `0014_add_wfh_setting.sql`** (cf. [README §5 — Numérotation des migrations](./README.md#numérotation-des-migrations)). **À l'exécution :** `ls src-tauri/migrations/vault/ | sort | tail -1` → prendre le numéro libre suivant, renommer le fichier `.sql`, corriger les références `00XX` de ce document, et aligner le champ `version:` dans `migrations.rs`.

## 1. Contexte & objectif

Dans la nomenclature réglementaire, **le télétravail n'est pas un poste d'émission distinct** : c'est une **donnée d'activité** qui *module* le poste BEGES 3.3 / GHG Protocol Scope 3 catégorie 7 (« Déplacements domicile-travail »). Un jour télétravaillé est un jour de trajet domicile-travail **évité** : il réduit mécaniquement le nombre/la distance des trajets comptabilisés. Le GHG Protocol précise par ailleurs que les émissions incrémentales du travail à domicile (chauffage/électricité du logement) **peuvent** (« may include ») être ajoutées au Scope 3 cat.7 — c'est une option, jamais une obligation, et seules les émissions **incrémentales vs ligne de base** doivent être comptées.

Aujourd'hui, BasicPresence traite déjà correctement le « trajet évité » (un jour `remote` n'a simplement pas de trajet, donc pas d'émission de déplacement). Mais l'application **ne rend ce mécanisme ni explicite ni traçable** : aucune statistique « jours télétravaillés », aucune ligne « émissions de trajet évitées », aucune distinction de l'énergie domicile (`home_day` = 2,7 kgCO₂e) comme « émissions incrémentales WFH ». L'objectif de ce lot est de **rendre le télétravail explicitement lisible comme donnée d'activité du poste 3.3** dans l'export et la méthodologie, et d'**exposer (optionnellement) l'énergie domicile comme émissions incrémentales WFH du Scope 3 cat.7**, sans rien double-compter et sans toucher au moteur de calcul figé (R9).

---

## 2. État actuel du code (point de départ)

### 2.1 Le télétravail est déjà un type de présence, sans sémantique « réduction »

- `PresenceType` (`src-tauri/src/domain/entities/presence.rs:6-11`) : enum `Office | Remote | Vacation | Holiday`. `Remote` = télétravail.
- Le moteur `Co2Calculator::compute_day` (`src-tauri/src/domain/services/co2_calculator.rs:62-105`) traite `Office` et `Remote` **à l'identique** côté trajets : il somme les `TripInput` fournis. Un jour `remote` typique n'a pas de trajet → total des trajets = 0. **Le « trajet évité » est donc implicite, jamais matérialisé.**
- `set_presence.rs:89-96` : seuls les jours `Office | Remote` portent des trajets ; les autres types vident la liste. Donc un jour `remote` *peut* porter un trajet (cas réel : télétravail le matin, déplacement client l'après-midi).

### 2.2 L'énergie bâtiment existe déjà mais n'est pas étiquetée « WFH incrémental »

- `compute_day` (`co2_calculator.rs:87-98`) ajoute un forfait bâtiment **uniquement si** `settings.count_building_energy == true` :
  - `PresenceType::Office` → facteur `office_day` (`OFFICE_DAY_MODE_ID`, `co2_calculator.rs:26`), valeur **3,5 kgCO₂e/jour**.
  - `PresenceType::Remote` → facteur `home_day` (`HOME_DAY_MODE_ID`, `co2_calculator.rs:27`), valeur **2,7 kgCO₂e/jour**.
  - Autres → rien.
- Ces deux facteurs sont seedés dans le référentiel (`migrations/vault/0004_add_co2.sql:119-120` pour 2025, `0011_add_co2_2026.sql:55-56` pour 2026), catégorie `building`, unité `kgCO2e/day`.
- Documentation existante (`documentation/calcul-impact-co2.md` §4.4 lignes 136-146, §7.6 lignes 395-403) : `home_day` = DEFRA/DESNZ « Homeworking » (0,33378 kgCO₂e/h × 8 h ≈ 2,67 → 2,7) ; `office_day` = CIBSE TM46 / Circular Ecology. **Le `home_day` est déjà un facteur « incrémental Homeworking »** par construction (équipement + chauffage attribuable au travail) — mais le code ne l'expose jamais comme tel.

### 2.3 Configuration profil

- `Co2Settings` (`src-tauri/src/domain/entities/co2_settings.rs:5-19`) : `grid_country`, `default_car_occupancy`, `include_radiative_forcing`, `count_building_energy`, `working_days_per_year`, `factor_year`. **Aucun champ propre au télétravail.**
- Stockage : table `profile_settings` (`migrations/vault/0010_add_profile_settings.sql:7-33`), colonnes typées miroir de `Co2Settings`. `factor_year` défaut 2026 (basculé par `0011_add_co2_2026.sql:72`). Repo : `profile_settings_repository.rs` (`COLUMNS` ligne 14-16, `row_to_settings` ligne 34-52, `save` ligne 69-103).
- Miroir frontend : `Co2Config` (`src/features/profile-settings/domain/entities/profile-settings.ts:5-12`) + `DEFAULT_PROFILE_SETTINGS` (lignes 33-45). UI : `src/features/settings/presentation/pages/settings-page.tsx` (toggle forçage radiatif lignes 273-285 comme modèle de checkbox).

### 2.4 Export (point de référence pour les colonnes WFH)

- Port d'export : `SpreadsheetExporter` (`src-tauri/src/domain/services/spreadsheet_exporter.rs:116-124`), avec `ExportOptions` (lignes 11-25), `ExportData`/`DayExport` (lignes 29-42), `ExportLabels` (lignes 49-90).
- Implémentation ODS : `src-tauri/src/infrastructure/export/ods_writer.rs` (feuilles Présences, Tâches, Trajets, Notes ; `factor_year` sur Trajets aux lignes 222 et 240).
- DTO frontière : `ExportOptionsDto` / `ExportLabelsDto` (`src-tauri/src/application/dto/export_dto.rs:7-65`), wire camelCase. **Aucune notion de « jours télétravaillés » ni « WFH incrémental ».**
- L'export est **par ligne** (jour/trajet) ; aucune agrégation. Les jours `remote` apparaissent dans la feuille Présences via `ExportLabels::type_label` → `type_remote`, mais aucun total/compteur.

### 2.5 Règles métier figées (rappel)

- **R7** : `Vacation | Holiday` → total = 0 (`co2_calculator.rs:71-77`).
- **R9** : un jour déjà encodé garde son `co2_kg` figé (`presence_trip.co2_kg` + `factor_year`, `0004_add_co2.sql:73-86`). Toute évolution du référentiel passe par un **nouveau millésime** (`0011` n'altère pas `0004`).
- Tests d'acceptation `ac1..ac10` + cas dédiés dans `co2_calculator.rs:166-479`. En particulier `ac7_week_commute_only` (ligne 329-335) et `ac8_week_with_building_energy` (ligne 337-355) couvrent déjà le mix bureau/télétravail.

---

## 3. Travail à réaliser

Le lot se décompose en deux volets **indépendamment activables** :

- **Volet A — Donnée d'activité « jours télétravaillés » (cœur du gap e)** : matérialiser, agréger et exporter le nombre de jours télétravaillés et les « trajets évités » comme modulateur du poste 3.3. Aucune nouvelle colonne SQL, aucun changement de calcul.
- **Volet B — WFH incrémental optionnel** : réétiqueter clairement le forfait `home_day` comme « émissions incrémentales WFH (Scope 3 cat.7, option GHG Protocol) » et le distinguer dans l'export, sans double-compter avec `count_building_energy`.

> **Dépendance Lot 1 :** le Volet A consomme le mapping `mode_id → poste BEGES 3.3 / Scope 3 cat.7` posé par le Lot 1 (champ/colonne `ghg_scope`, `beges_poste`, `ghg_category` ou équivalent). Si le Lot 1 n'est pas encore livré, implémenter d'abord les tâches **T1, T2, T6, T7, T9** (purement front + agrégation + libellés), et différer **T3, T4, T5, T8** (qui réfèrent au mapping poste) jusqu'à disponibilité du Lot 1.

Tâches ordonnées :

### T1 — Helper d'agrégation « jours télétravaillés » (frontend, sans backend)
**Fichier :** `src/features/stats/domain/aggregate.ts` (à étendre).
**Action :** ajouter une fonction pure `teleworkSummary(presences: Presence[]): TeleworkSummary` qui, sur l'ensemble (ou un sous-ensemble filtré) de présences, calcule :
- `officeDays` = nombre de jours `office`,
- `remoteDays` = nombre de jours `remote`,
- `workDays` = `officeDays + remoteDays` (jours travaillés),
- `teleworkRate` = `remoteDays / workDays` (0 si `workDays == 0`),
- `remoteDaysWithTrip` = nombre de jours `remote` portant ≥ 1 trajet (cas mixte télétravail + déplacement client),
- `remoteDaysCommuteAvoided` = `remoteDays - remoteDaysWithTrip` (jours de pur évitement de trajet).

Définir l'interface `TeleworkSummary` à côté de `PeriodBucket`/`BreakdownSlice` (`aggregate.ts:20-38`). Ne dépend d'aucune entité backend nouvelle. Réutilise `PresenceType` et la convention `co2Kg: number | null` déjà en place (`src/features/presence/domain/entities/presence.ts:11-27`).

### T2 — Affichage « télétravail » sur la page Stats (frontend)
**Fichier :** `src/features/stats/presentation/pages/stats-page.tsx` (à étendre) + un petit composant `src/features/stats/presentation/components/telework-card.tsx` (à créer).
**Action :** afficher une carte « Télétravail » : `remoteDays` / `workDays` jours, taux de télétravail (`teleworkRate` en %), et la mention « N jours de trajet domicile-travail évités ». Brancher sur `teleworkSummary` (T1) avec le même filtre/range que les charts existants. Libellés via i18n (clés ajoutées en T7).

### T3 — Étiquetage Scope/poste du poste 3.3 « modulé » dans la méthodologie (frontend)
**Fichiers :** `src/features/methodology/presentation/pages/methodology-page.tsx`, i18n (T7), `documentation/calcul-impact-co2.md`.
**Action :** ajouter dans la page Méthodologie une sous-section « Télétravail » expliquant que (1) un jour télétravaillé = un trajet domicile-travail évité = **réduction de la donnée d'activité** du poste BEGES 3.3 / GHG Protocol Scope 3 cat.7 ; (2) l'énergie domicile (`home_day`) est une **option** « émissions incrémentales WFH » (cf. Volet B). Réutiliser le mapping poste du Lot 1 (le `home_day` reste catégorie `building` mais est rattaché, en reporting, au Scope 3 cat.7 quand l'option WFH est active — voir §5.3).

### T4 — Agrégation backend « activité 3.3 » (use case, dépend Lot 1)
**Fichier :** `src-tauri/src/application/use_cases/export_profile_data.rs` (à étendre) — ou un nouveau helper d'agrégation dans la couche application.
**Action :** lors du rassemblement des jours pour l'export, calculer une synthèse « poste 3.3 » incluant : `office_days`, `remote_days`, `commute_avoided_days`, et — si le mapping Lot 1 est disponible — le total `co2_kg` des trajets rattachés au poste 3.3 (Scope 3 cat.7). Cette synthèse alimente la feuille de méthodologie/synthèse (T5). **Aucune persistance** : agrégation en mémoire à l'export uniquement (cohérent avec l'architecture actuelle, cf. `export_profile_data.rs`).

### T5 — Colonnes/feuille d'export « Télétravail » (backend ODS)
**Fichiers :** `src-tauri/src/domain/services/spreadsheet_exporter.rs`, `src-tauri/src/infrastructure/export/ods_writer.rs`, `src-tauri/src/application/dto/export_dto.rs`.
**Action :**
1. Étendre `ExportOptions` (et `ExportOptionsDto`) avec `include_telework: bool` (défaut `false` pour rétrocompat — voir §8).
2. Étendre `ExportLabels` (et `ExportLabelsDto`) avec les libellés télétravail (`telework_section`, `telework_days`, `office_days`, `work_days`, `telework_rate`, `commute_avoided_days`, `wfh_incremental`, `wfh_incremental_tco2e`, `wfh_incremental_kg`).
3. Dans `ods_writer.rs`, écrire (si `include_telework`) un bloc/feuille « Télétravail » : `Jours télétravaillés`, `Jours bureau`, `Jours travaillés`, `Taux de télétravail (%)`, `Trajets domicile-travail évités (jours)`. Quand l'option WFH incrémental est active (Volet B, T8), ajouter `WFH incrémental (kgCO₂e)` et — si Lot 3 « tCO₂e » est dispo — `WFH incrémental (tCO₂e)`.

### T6 — Champ profil `wfh_incremental_energy` (Volet B, settings)
**Fichiers :** `src-tauri/src/domain/entities/co2_settings.rs`, `migrations/vault/00NN_*.sql` (nouvelle migration — voir §4), `src-tauri/src/infrastructure/persistence/profile_settings_repository.rs`, `src/features/profile-settings/domain/entities/profile-settings.ts`, `src/features/settings/presentation/pages/settings-page.tsx`.
**Action :** ajouter un paramètre booléen **`wfh_incremental_energy`** (défaut `false`) à `Co2Settings` + colonne `profile_settings.wfh_incremental_energy`. Sa **seule** fonction : décider si, dans l'export et la méthodologie, le forfait `home_day` des jours `remote` est **étiqueté** « émissions incrémentales WFH du Scope 3 cat.7 ». **Il ne change PAS le calcul** : le calcul du forfait reste piloté par `count_building_energy` (voir §5.2 « anti-double-comptage »). Miroir frontend dans `Co2Config` + `DEFAULT_PROFILE_SETTINGS`, et toggle dans la page Settings (sur le modèle du toggle `includeRadiativeForcing`, `settings-page.tsx:273-285`).

### T7 — Clés i18n FR + EN
**Fichiers :** `src/core/i18n/locales/fr/translation.json`, `src/core/i18n/locales/en/translation.json`.
**Action :** ajouter les clés sous `stats.telework.*`, `methodology.telework.*`, `settings.co2.wfhIncremental*`, et `export.telework.*` (libellés passés au backend via `ExportLabelsDto`). **Les deux langues obligatoirement** (un manque FR/EN casse l'export, cf. §8).

### T8 — Branchement WFH incrémental dans l'export (backend)
**Fichiers :** `export_profile_data.rs`, `ods_writer.rs`.
**Action :** quand `wfh_incremental_energy == true` **et** `count_building_energy == true`, calculer et exporter le total WFH incrémental = Σ (`home_day` des jours `remote`), exposé sous le label « émissions incrémentales WFH (Scope 3 cat.7) ». Quand `count_building_energy == false`, le total WFH incrémental est `0` (les jours `remote` n'ont alors aucun forfait domicile dans le calcul figé) : afficher `0` avec une note explicite « l'énergie bâtiment n'est pas comptée » plutôt que d'inventer une valeur (anti-double-comptage / cohérence avec les `co2_kg` figés, R9).

### T9 — Documentation
**Fichier :** `documentation/calcul-impact-co2.md` (nouvelle sous-section §4.5 « Télétravail »).
**Action :** documenter la sémantique « jour télétravaillé = trajet évité = réduction du poste 3.3 », l'option WFH incrémental, et la règle anti-double-comptage. Renvoyer au présent document de lot.

---

## 4. Modèle de données / migrations

**Volet A : sans objet** (aucune colonne SQL ; agrégation pure en mémoire / frontend).

**Volet B : une migration vault nécessaire** pour le flag profil `wfh_incremental_energy`. La dernière migration connue est `0011_add_co2_2026.sql` → la prochaine est **`0012`**. Si le Lot 1 introduit déjà un `0012` (mapping Scope/poste), ce lot utilisera le **numéro suivant disponible** (`0013`, etc.) — la règle est : numérotation croissante stricte, un fichier par changement logique, jamais d'édition in-place d'une migration livrée.

Fichier à créer : `src-tauri/migrations/vault/0012_add_wfh_setting.sql` (ou numéro suivant libre) :

```sql
-- Option de reporting « émissions incrémentales WFH » (télétravail) — gap (e).
-- N'INFLUE PAS sur le calcul figé : c'est un drapeau de PRÉSENTATION (export /
-- méthodologie) qui étiquette le forfait `home_day` des jours `remote` comme
-- « émissions incrémentales WFH du Scope 3 cat.7 » (option GHG Protocol « may
-- include »). Le calcul du forfait domicile reste piloté par
-- `count_building_energy`. Défaut 0 (off) → rétrocompatibilité totale.
ALTER TABLE profile_settings
    ADD COLUMN wfh_incremental_energy INTEGER NOT NULL DEFAULT 0
        CHECK (wfh_incremental_energy IN (0, 1));

-- Backfill explicite : tout profil existant reste à 0 (comportement actuel).
-- No-op sur une install neuve. ADD COLUMN ... DEFAULT 0 le garantit déjà ;
-- la ligne ci-dessous est conservée pour la lisibilité et l'idempotence.
UPDATE profile_settings SET wfh_incremental_energy = 0
    WHERE wfh_incremental_energy IS NULL;
```

> **Important — pas de nouveau facteur, pas de nouveau millésime.** Ce lot **ne modifie aucun facteur** (`home_day` reste 2,7 ; `office_day` reste 3,5) et **ne crée aucun nouveau `mode_id`**. Il n'y a donc **pas** de seed `emission_factor` ici et **pas** de violation de R9. Le rattachement reporting `home_day → Scope 3 cat.7 (option WFH)` est porté par le mapping du Lot 1 + le flag de présentation, pas par une nouvelle valeur figée.

Mise à jour du repo `profile_settings_repository.rs` :
- `COLUMNS` (ligne 14-16) : ajouter `wfh_incremental_energy` en fin.
- `row_to_settings` (ligne 34-52) : lire l'index correspondant (i64 → bool, comme `include_radiative_forcing`/`count_building_energy`).
- `save` (ligne 69-103) : ajouter la colonne à l'INSERT + au `ON CONFLICT DO UPDATE`.

---

## 5. Spécification détaillée

### 5.1 Sémantique réglementaire du télétravail

| Concept | Place dans la nomenclature | Traitement dans BasicPresence |
|---|---|---|
| Jour télétravaillé (`remote` sans trajet) | **Donnée d'activité** du poste BEGES 3.3 / GHG Protocol Scope 3 cat.7 : trajet domicile-travail **évité** | Total trajets du jour = 0 (déjà le cas). Compté comme `commute_avoided_days`. |
| Jour télétravaillé mixte (`remote` avec trajet) | Donnée d'activité 3.3 : trajet **réel** ce jour-là (ex. déplacement client) | Trajets comptés normalement (déjà le cas). Compté dans `remoteDaysWithTrip`, **exclu** de `commute_avoided_days`. |
| Énergie domicile (`home_day` = 2,7 kgCO₂e/jour) | **Option** « WFH incremental emissions » du Scope 3 cat.7 (« may include », GHG Protocol) | Ajoutée au total **uniquement si `count_building_energy == true`** (inchangé). Étiquetée « WFH incrémental » dans l'export **uniquement si `wfh_incremental_energy == true`**. |

**Le télétravail n'est jamais un poste séparé.** Aucune colonne « poste télétravail », aucun nouveau Scope. C'est un modulateur du 3.3.

### 5.2 Anti-double-comptage (règle centrale du lot)

Trois grandeurs ne doivent **jamais** se chevaucher :

1. **Trajet évité** (Volet A) — c'est une **absence** d'émission, exprimée comme *information* (compteur de jours), **pas** comme une valeur kgCO₂e négative ni un crédit. On ne soustrait rien d'un total : on **affiche** « N jours évités ». Aucun risque de double-comptage car aucune valeur n'est manipulée.
2. **Émission de trajet réelle** d'un jour `remote` mixte — comptée **une fois**, via les trajets normaux (`presence_trip`), exactement comme aujourd'hui.
3. **WFH incrémental** (`home_day`, Volet B) — comptée **une fois**, via le forfait bâtiment existant, **et seulement si `count_building_energy == true`**.

| `count_building_energy` | `wfh_incremental_energy` | `home_day` ajouté au `co2_kg` ? | « WFH incrémental » affiché dans l'export ? |
|---|---|---|---|
| `false` | `false` | Non | Non (section masquée ou « 0, énergie bâtiment non comptée ») |
| `false` | `true` | **Non** | « 0 — l'énergie bâtiment n'est pas comptée » (note explicite, T8) |
| `true` | `false` | Oui (comportement actuel) | Non (le forfait reste « énergie bâtiment » générique) |
| `true` | `true` | Oui (**inchangé**) | Oui : le `home_day` des jours `remote` est **étiqueté** « WFH incrémental (Scope 3 cat.7) » |

> Le flag `wfh_incremental_energy` est **purement présentationnel**. Il ne déclenche aucun ajout d'émission : il **réétiquette** une émission déjà calculée et déjà figée. Ainsi, activer/désactiver ce flag **ne modifie aucun `co2_kg` figé** (R9 respecté), et il est **impossible** de compter `home_day` deux fois.

### 5.3 Rattachement reporting du `home_day` au Scope 3 cat.7

Aujourd'hui `home_day` est catégorie `building` (Scope 2 « énergie bâtiment » dans l'esprit du Lot 1). Quand l'option WFH est active, en **reporting uniquement**, le `home_day` des jours `remote` est présenté sous **Scope 3 catégorie 7** (émissions incrémentales du télétravail), conformément au GHG Protocol. C'est un choix de **présentation** porté par le flag, pas une réécriture du mapping de base (le `office_day`, lui, reste building/Scope 2). Documenter ce point explicitement (T3, T9) pour qu'un OTI/CAC comprenne le rattachement.

### 5.4 Formules d'agrégation (Volet A, T1)

Sur un ensemble de présences `P` (filtré comme les charts) :

```
officeDays               = |{ p ∈ P : p.type == office }|
remoteDays               = |{ p ∈ P : p.type == remote }|
workDays                 = officeDays + remoteDays
teleworkRate             = workDays == 0 ? 0 : remoteDays / workDays
remoteDaysWithTrip       = |{ p ∈ P : p.type == remote ∧ a au moins 1 trajet }|
remoteDaysCommuteAvoided = remoteDays − remoteDaysWithTrip
```

Note : la connaissance « a au moins 1 trajet » côté frontend dépend de ce qui est chargé. Si la liste de présences front ne porte pas le détail des trajets (cf. `PresenceDto` qui n'inclut pas les trajets), approximer `remoteDaysWithTrip` par « jour `remote` dont `co2Kg > 0` **et** `count_building_energy == false` » est ambigu (le forfait `home_day` rend `co2Kg > 0` sans trajet). **Recommandation :** côté frontend, considérer un jour `remote` comme « trajet évité » par défaut, et n'exposer `remoteDaysWithTrip` que si le détail trajets est disponible ; côté backend (T4), utiliser directement `DayExport.trips` (`spreadsheet_exporter.rs:33`) qui porte les trajets réels — c'est la source fiable. Documenter cette nuance.

### 5.5 WFH incrémental total (Volet B, T8)

```
wfhIncrementalKg = (count_building_energy ∧ wfh_incremental_energy)
                   ? Σ_{ jours remote } home_day_value(factor_year_du_jour)
                   : 0
```

`home_day_value` doit être lue **par millésime** : un jour `remote` figé en 2025 a pu utiliser `home_day` 2025 (même valeur 2,7, mais le principe par-millésime est obligatoire pour rester cohérent avec R9). En pratique, comme le forfait n'est pas snapshoté séparément du total du jour, le plus robuste est de **recomputer** la contribution `home_day` à partir du `factor_year` du jour et du référentiel de cette année (lecture seule du facteur `home_day` pour `year = presence.factor_year`). Si `count_building_energy == false`, le total est `0` (aucun forfait n'a été ajouté au calcul figé).

### 5.6 Edge cases

| Cas | Comportement attendu |
|---|---|
| Aucun jour travaillé (`workDays == 0`) | `teleworkRate = 0`, pas de division par zéro. |
| Jour `remote` mixte (télétravail + déplacement client) | Trajet réel compté ; jour **non** compté comme « trajet évité ». |
| `vacation` / `holiday` | Exclus de tous les compteurs télétravail (ce ne sont pas des jours travaillés). R7 inchangé. |
| `wfh_incremental_energy = true` mais `count_building_energy = false` | WFH incrémental affiché = `0` + note « énergie bâtiment non comptée » (jamais de valeur inventée). |
| Jours pré-CO₂ importés (`co2_kg = NULL`) | Comptés dans `officeDays`/`remoteDays` selon leur type ; ignorés pour les totaux kgCO₂e. |
| Export avec `include_telework = false` | Aucune feuille/colonne télétravail (rétrocompat). |

---

## 6. Critères d'acceptation

1. **Donnée d'activité 3.3 visible.** Quand l'utilisateur ouvre la page Stats, alors une carte « Télétravail » affiche `remoteDays`/`workDays`, le taux de télétravail (%), et le nombre de « jours de trajet domicile-travail évités ».
2. **Taux correct.** Pour un profil avec 3 jours `office` et 2 jours `remote` (sans trajet), alors `teleworkRate = 40 %` et `commuteAvoidedDays = 2`.
3. **Jour mixte non compté comme évité.** Pour un jour `remote` portant un trajet réel, alors ce jour n'est **pas** dans `commuteAvoidedDays` mais bien dans `remoteDays`.
4. **Pas de division par zéro.** Pour un profil sans aucun jour travaillé, alors `teleworkRate = 0` et aucune erreur.
5. **Export télétravail opt-in.** Quand `include_telework = true`, alors l'ODS contient une section/feuille « Télétravail » avec les compteurs ; quand `false`, alors l'export est **identique** à l'export actuel (aucune feuille/colonne ajoutée).
6. **WFH incrémental étiqueté, jamais double-compté.** Quand `count_building_energy = true` **et** `wfh_incremental_energy = true`, alors l'export affiche un total « émissions incrémentales WFH (Scope 3 cat.7) » égal à Σ `home_day` des jours `remote`, et ce total **n'ajoute rien** au total général déjà figé (il pointe sur la même émission, étiquetée).
7. **WFH off → 0 explicite.** Quand `count_building_energy = false`, alors le total WFH incrémental affiché est `0` avec la note « l'énergie bâtiment n'est pas comptée », quelle que soit la valeur de `wfh_incremental_energy`.
8. **R9 intact.** Activer/désactiver `wfh_incremental_energy` ne modifie **aucun** `co2_kg` de `presence` ni de `presence_trip` (vérifiable : re-export d'un même profil produit les mêmes totaux généraux).
9. **Migration sûre.** Après application de la nouvelle migration, tout profil existant a `wfh_incremental_energy = 0` et son comportement de calcul est inchangé.
10. **i18n complète.** Toutes les nouvelles chaînes existent en FR **et** EN ; l'export en FR et en EN produit des libellés cohérents (aucune clé manquante).

**Benchmark conformité :** un lecteur BEGES/OTI peut, à partir de l'export, (a) lire le nombre de jours télétravaillés comme donnée d'activité du poste 3.3, (b) constater que les trajets évités ne sont pas comptés comme émissions, et (c) si l'option est active, identifier les émissions incrémentales WFH rattachées au Scope 3 cat.7, sans risque de double-comptage.

---

## 7. Tests à écrire / mettre à jour

### 7.1 Rust — moteur de calcul (non-régression R9)
Fichier : `src-tauri/src/domain/services/co2_calculator.rs` (module `tests`, lignes 166-479).
- **Conserver** `ac7_week_commute_only` et `ac8_week_with_building_energy` **inchangés** : ils prouvent que le calcul du mix bureau/télétravail n'est pas touché par ce lot. Aucune valeur attendue ne doit changer (le lot ne modifie pas `compute_day`).
- Ajouter `wfh_setting_does_not_change_day_total()` : vérifier qu'aucun champ `wfh_incremental_energy` n'entre dans `Co2Settings` consommé par `compute_day` (le flag est de présentation ; s'assurer qu'il n'est **pas** lu par le calculateur). Si `Co2Settings` gagne le champ (T6), ce test garantit que `compute_day` l'ignore : même `DayEmission.total_kg` avec le flag à `true` ou `false`.

### 7.2 Rust — agrégation export (Volet A/B)
Fichier : `src-tauri/src/application/use_cases/export_profile_data.rs` (module `tests` à ajouter ou compléter) ou un module dédié.
- `telework_summary_counts_office_remote()` : 3 office + 2 remote (sans trajet) → `office_days = 3`, `remote_days = 2`, `commute_avoided_days = 2`.
- `mixed_remote_day_not_counted_as_avoided()` : 1 remote avec trajet → `commute_avoided_days = 0`.
- `wfh_incremental_zero_when_building_off()` : `count_building_energy = false` → total WFH incrémental = 0.
- `wfh_incremental_sums_home_day_when_on()` : `count_building_energy = true` + 2 jours remote → total = 2 × 2,7 = 5,4 kgCO₂e.

### 7.3 Rust — repository settings
Fichier : `src-tauri/src/infrastructure/persistence/profile_settings_repository.rs` (ou `integration_tests.rs`).
- Round-trip `save`/`load` de `wfh_incremental_energy` (true et false), et lazy default (`load` d'un profil sans ligne → `false`).

### 7.4 Vitest — agrégation frontend
Fichier : `src/features/stats/domain/aggregate.test.ts` (colocalisé).
- `teleworkSummary` : taux 40 % sur 3 office + 2 remote ; `teleworkRate = 0` sur 0 jour travaillé ; exclusion vacation/holiday ; `commuteAvoidedDays` correct avec/sans trajet.

### 7.5 Commandes
```bash
# Backend
cargo test --manifest-path src-tauri/Cargo.toml
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings
# Frontend
pnpm test
pnpm typecheck
pnpm lint
```

---

## 8. Points d'attention & pièges

- **R9 / figeage des jours.** Le lot **ne doit modifier ni `compute_day` ni les facteurs**. Le flag `wfh_incremental_energy` est de présentation : il ne doit **jamais** être lu par `Co2Calculator` (sinon ré-éditer un jour changerait son total → casse R9). Vérifié par `wfh_setting_does_not_change_day_total` (§7.1).
- **Anti-double-comptage.** Le « trajet évité » est une *information*, pas une valeur ; le WFH incrémental *étiquette* une émission existante. Ne **jamais** soustraire le trajet évité d'un total, ni ajouter `home_day` en plus du forfait déjà calculé (§5.2).
- **Dépendance Lot 1.** Le rattachement « `home_day` / trajets → poste 3.3 / Scope 3 cat.7 » s'appuie sur le mapping du Lot 1. Sans lui, livrer d'abord les tâches purement frontend/agrégation (T1, T2, T6, T7, T9) ; les libellés « poste 3.3 / cat.7 » dans l'export (T4, T5, T8) attendent le mapping.
- **i18n FR + EN obligatoires.** L'export reçoit les libellés du frontend (`ExportLabelsDto`) ; une clé manquante en FR ou EN casse l'export ou produit un libellé vide. Ajouter chaque clé dans les deux fichiers (`src/core/i18n/locales/{fr,en}/translation.json`).
- **Rétrocompatibilité de l'export.** `include_telework` et `wfh_incremental_energy` par défaut `false` → un export existant reste bit-à-bit identique. Le `ExportSummaryDto` (`export_dto.rs:103-114`) peut rester inchangé, ou gagner des compteurs ; si modifié, mettre à jour le mapper frontend des résultats.
- **Couches DDD.** `Co2Settings` (domaine) ← `profile_settings` (infra) ← `Co2Config` (front) : le nouveau champ doit être **aligné** dans les trois, avec le même défaut (`false`), exactement comme l'a été `count_building_energy`. Le `DEFAULT_PROFILE_SETTINGS` front (`profile-settings.ts:33-45`) **doit** refléter le défaut SQL/`Co2Settings::default()`.
- **Source `remoteDaysWithTrip`.** Côté frontend, le détail trajets n'est pas garanti chargé (`PresenceDto` ne porte pas les trajets) ; la source fiable du « jour mixte » est backend (`DayExport.trips`). Documenter et tester les deux chemins (§5.4).
- **Mouvance réglementaire.** L'inclusion des émissions WFH est *optionnelle* dans le GHG Protocol (« may include ») et son périmètre (incrémental vs total) peut évoluer. Le flag opt-in + la note explicite protègent contre une sur-déclaration ; ne pas l'activer par défaut.
- **Numéro de migration.** Vérifier le dernier `00NN_*.sql` au moment de l'implémentation (dernier connu : `0011`, le Lot 1 peut consommer `0012`). Prendre le **numéro libre suivant**, jamais réutiliser ni éditer une migration livrée.

---

## 9. Références réglementaires

- **GHG Protocol — Corporate Value Chain (Scope 3) Standard**, catégorie 7 « Employee commuting » : le télétravail y est une donnée d'activité du déplacement domicile-travail ; les émissions du travail à domicile **peuvent** (« companies may include emissions from teleworking ») être incluses dans la cat.7. https://ghgprotocol.org/standards/scope-3-standard
- **GHG Protocol — Technical Guidance for Calculating Scope 3 Emissions**, §7 (Employee commuting), méthodes distance-based et average-data, traitement du télétravail. https://ghgprotocol.org/scope-3-technical-calculation-guidance
- **BEGES (Bilan d'émissions de gaz à effet de serre réglementaire, art. L229-25 Code de l'environnement)** — méthode réglementaire, poste 3.3 « Déplacements domicile-travail » (donnée d'activité = jours/distances effectivement parcourus). Base : https://bilans-ges.ademe.fr/ et la méthode pour les bilans réglementaires (ADEME / Ministère de la Transition écologique).
- **ESRS E1-6 (CSRD)** — exigence de cohérence du périmètre Scope 3 et de transparence des hypothèses (dont le traitement du télétravail). EFRAG ESRS E1. https://www.efrag.org/
- **DEFRA/DESNZ — UK Government GHG Conversion Factors**, facteur « Homeworking » (base du `home_day` = 2,7 kgCO₂e/jour ≈ 0,33378 kgCO₂e/h × 8 h, méthodo EcoAct 2020). https://www.gov.uk/government/collections/government-conversion-factors-for-company-reporting — détail dans `documentation/calcul-impact-co2.md` §7.6.
- **Circular Ecology / EcoAct — Homeworking vs office emissions** (source des chiffres `home_day` 2,7 et `office_day` 3,5). https://circularecology.com/news/the-carbon-emissions-of-homeworking-and-office-working
