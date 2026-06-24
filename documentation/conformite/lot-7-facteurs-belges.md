# Lot 7 — Facteurs d'émission belges (facteursdemissionco2.be)

> **Statut :** À faire  ·  **Priorité :** P2  ·  **Effort :** faible-moyen  ·  **Dépend de :** Lot 1 (classification Scope GHG / périmètre — dépendance légère, voir §3) ; suit le pattern d'ajout de millésime (`0011_add_co2_2026.sql`).
> **Gap couvert :** (g) Intégrer le référentiel belge facteursdemissionco2.be comme source d'émission versionnée, en plus du facteur SNCB déjà présent.
> **Débloque :** Diagnostic fédéral mobilité belge (entreprises >100 travailleurs), Plan de Déplacements d'Entreprise (PDE) bruxellois / COBRACE, et clients belges exigeant une source nationale plutôt qu'ADEME (France).

---

> ⚠️ **Numérotation de migration — lire avant de coder.** Ce document écrit `0012` en supposant un dépôt à `0011`, mais **ce lot ne peut pas garder ce numéro** : il entre en collision avec le Lot 1. Migration **réservée par le plan : `0015_add_be_factors.sql`** (cf. [README §5 — Numérotation des migrations](./README.md#numérotation-des-migrations)). **À l'exécution :** `ls src-tauri/migrations/vault/ | sort | tail -1` → prendre le numéro libre suivant, renommer le fichier `.sql`, corriger les références `00XX` de ce document, et aligner le champ `version:` dans `migrations.rs`.

## 1. Contexte & objectif

BasicPresence calcule l'empreinte des déplacements domicile-travail à partir d'un référentiel ADEME (France) / DEFRA (UK) / SNCF, complété d'une seule ligne belge (`train_sncb`). Or les clients belges, et a fortiori le **diagnostic fédéral mobilité** (obligatoire pour les unités d'établissement de plus de 100 travailleurs) et le **Plan de Déplacements d'Entreprise bruxellois** (ordonnance COBRACE), s'appuient sur le référentiel national de facto **facteursdemissionco2.be** (porté par CO2logic et EnergieID — il n'existe pas de base d'État belge équivalente à la Base Carbone française). L'objectif de ce lot est d'intégrer la catégorie « transport de personnes » de facteursdemissionco2.be comme **nouveau millésime versionné** (donc sans toucher aux millésimes 2025/2026 existants, R9), de l'exposer comme choix de référentiel par profil, et de documenter l'arbre de décision (facteurs belges en priorité pour clients BE, sinon Base Carbone/GHG Protocol).

---

## 2. État actuel du code (point de départ)

Faits précis tirés du code :

- **Table référentiel** : `emission_factor (id TEXT, year INTEGER, label, value REAL, unit, category, scope, is_param, source, PRIMARY KEY(id, year))` — `src-tauri/migrations/vault/0004_add_co2.sql:15-28`. **La clé est `(id, year)` uniquement : il n'existe AUCUNE colonne `country` ni `region` sur `emission_factor`.** Le seul axe de versionnement disponible est `year`.
- **Variantes réseau** : `emission_factor_grid_variant (mode_id, year, country, value, PRIMARY KEY(mode_id, country, year))` — `0004_add_co2.sql:32-39`. C'est le SEUL endroit où une dimension `country` existe, et elle ne sert qu'à surcharger la valeur de base d'un mode électrique (`car_ev`) selon le pays de recharge (FR/BE/DE/EU). Ce n'est PAS un mécanisme de référentiel national complet.
- **Seed 2025** : `0004_add_co2.sql:94-129` (26 facteurs + 4 variantes `car_ev`). Verrouillé, jamais réédité (R9).
- **Seed 2026** : `0011_add_co2_2026.sql:30-64` (mêmes 26 ids, valeurs corrigées + variantes). Le fichier 0011 est le **patron exact** à suivre : `INSERT OR IGNORE` du référentiel d'un nouveau millésime, puis `UPDATE profile_settings SET factor_year = … WHERE …` pour basculer le défaut.
- **Ligne belge déjà présente** : `train_sncb` (`'Train Intercity (SNCB)'`, `0.021 kgCO2e/passenger.km`, source `'SNCB'`) en 2025 (`0004:110`) et 2026 (`0011:46`). C'est la seule ligne d'origine belge.
- **Chargement** : `load_factor_maps(factors, year)` (`src-tauri/src/application/use_cases/factor_maps.rs:13-30`) charge `factors.list(year)` en `HashMap<mode_id, EmissionFactor>` et `factors.list_grid_variants(year)` en `HashMap<(mode_id, country), value>`. Le calcul (`Co2Calculator::compute_day/compute_trip`) ne connaît qu'un `factor_year` et un `grid_country` ; il ne sait rien d'un « pays de référentiel ».
- **Repository** : `LibsqlEmissionFactorRepository::list(year)` et `list_grid_variants(year)` — `src-tauri/src/infrastructure/persistence/emission_factor_repository.rs:55-89`. Colonnes lues : `id, label, value, unit, category, is_param, scope, source` (constante `SELECT_COLUMNS:14`). Filtre exclusivement sur `year`.
- **Sélection du millésime** : `Co2Settings.factor_year` (`src-tauri/src/domain/entities/co2_settings.rs:18`, défaut `2026:29`), persisté dans `profile_settings.factor_year` (`0010_add_profile_settings.sql:7-33`, CHECK absent sur la valeur — un entier libre `>= 2025` par convention). `grid_country` ∈ {FR, BE, DE, EU} (CHECK dans 0010).
- **Méthodologie (UI)** : `list_co2_referential.rs:26-62` expose le référentiel du `factor_year` par défaut (`Co2Settings::default().factor_year`) avec `scope`/`source`/`gridVariants`. Côté front : `Co2ReferentialFactor` (`src/features/methodology/domain/entities/co2-referential.ts:9-22`) porte déjà `source: string | null`. Les liens de sources sont en dur dans `src/features/methodology/presentation/sources.ts` (`SOURCE_GROUPS`), avec un groupe `SNCB` mais **aucun lien facteursdemissionco2.be**.
- **Migrations enregistrées** : `src-tauri/src/infrastructure/persistence/migrations.rs:22-67`. Dernière version vault = **11** (`0011_add_co2_2026.sql`). Une nouvelle migration s'ajoute via `Migration { version: 12, sql: include_str!(".../0012_….sql") }`.
- **Documentation** : `documentation/calcul-impact-co2.md` (§8 millésimes, §10 sources, règles R1-R9).

**Conséquence de conception** : il n'y a pas de colonne `country` sur le référentiel principal. Plutôt que d'introduire une telle colonne (changement de schéma plus lourd, qui casserait la clé `(id, year)` et le chargement par `year` seul), **on réutilise l'axe `year` existant pour porter un référentiel belge versionné** : un millésime dédié dont les valeurs proviennent de facteursdemissionco2.be. C'est la solution la moins intrusive, R9-safe, et alignée sur le pattern 0011.

---

## 3. Travail à réaliser

> Ordre impératif. T1 est la pièce maîtresse ; T2-T7 l'exposent et la documentent.

**T1 — Créer la migration `src-tauri/migrations/vault/0012_add_be_factors.sql`** (nouveau fichier).
Seeder un référentiel belge complet sous un **nouveau `year` dédié = `2026` + offset belge**. Convention retenue : **`year = 12026`** (préfixe `1` = « variante nationale belge » du millésime 2026 ; voir §5 pour la justification du choix « année synthétique » vs « année calendaire »). Le seed reprend **tous les `id` (mode_id) du référentiel 2026** (pour que tout trajet reste calculable), en remplaçant les valeurs par les facteurs belges quand facteursdemissionco2.be en publie un, et en **conservant la valeur 2026 (ADEME/DEFRA/SNCF) comme repli documenté** pour les modes sans équivalent belge publié. Inclure les variantes `car_ev` pour `year = 12026` (au minimum BE ; FR/DE/EU recopiés pour cohérence). Voir le DDL/seed complet en §4 et la table de valeurs en §5. **Ne JAMAIS modifier** 0004 ni 0011 (R9).

**T2 — Enregistrer la migration** dans `src-tauri/src/infrastructure/persistence/migrations.rs`.
Ajouter, après l'entrée `version: 11`, un bloc :
```rust
Migration {
    version: 12,
    sql: include_str!("../../../migrations/vault/0012_add_be_factors.sql"),
},
```

**T3 — Permettre la sélection du référentiel belge par profil.**
Aucun changement de structure n'est requis : `Co2Settings.factor_year` (`co2_settings.rs:18`) et `profile_settings.factor_year` (`0010`) sont des entiers libres. Il suffit qu'un profil belge stocke `factor_year = 12026`. **Toutefois**, pour que ce choix soit lisible côté UI, ajouter dans `src/features/settings/presentation/pages/settings-page.tsx` un sélecteur « Référentiel de facteurs » dont les options sont libellées (« France / international — Base Carbone 2026 » → 12026 « Belgique — facteursdemissionco2.be » selon `factorYear`). Le wire format (`profile-settings.dto.ts`, `FileSettings` dans `json_bundle_codec.rs`) transporte déjà `factorYear: i32` : aucune migration de DTO. **Vérifier** qu'aucune validation existante (import bundle) ne rejette un `factorYear` hors {2025, 2026} ; si une borne `<= 2026` existe, l'élargir.

**T4 — Exposer la méthodologie belge.**
Dans `src/features/methodology/presentation/sources.ts`, ajouter un `SourceGroup` :
```ts
{
  org: "facteursdemissionco2.be",
  links: [
    { label: "Transport de personnes (CO2logic / EnergieID)",
      url: "https://www.facteursdemissionco2.be/" },
  ],
},
```
La page méthodologie (`methodology-page.tsx`) consomme déjà `source` par facteur via `list_co2_referential.rs` ; comme chaque ligne belge porte `source = 'facteursdemissionco2.be'`, elle s'affichera automatiquement quand le profil est sur le millésime belge. Aucune nouvelle clé i18n n'est requise pour la table (les `source` sont des chaînes backend) ; ajouter seulement les libellés du sélecteur T3 dans `src/core/i18n/locales/{fr,en}/translation.json`.

**T5 — Documenter l'arbre de décision** dans `documentation/calcul-impact-co2.md`.
Ajouter une sous-section (§8.x « Référentiel belge » ou §10) reprenant l'arbre de §5 ci-dessous : client/établissement belge ⇒ `factor_year = 12026` (facteursdemissionco2.be en priorité, repli 2026 documenté pour les modes non couverts) ; sinon ⇒ référentiel 2026 (Base Carbone / DEFRA, aligné GHG Protocol). Mentionner que facteursdemissionco2.be est la référence de facto (pas de base d'État belge), et lister le périmètre couvert.

**T6 — Tests** (voir §7) : test d'intégration Rust vérifiant que `list(12026)` charge un référentiel complet et que les valeurs belges sont distinctes des 2026, et test du calcul sur un profil `factor_year = 12026`.

**T7 — Lot 1 (dépendance légère)** : si le Lot 1 (classification Scope GHG / renommage `scope` → périmètre) est déjà livré au moment d'écrire 0012, **reporter les colonnes de classification** (ex. `ghg_scope`, `beges_category`, etc.) dans le seed belge avec les mêmes valeurs que 2026 par `mode_id` (la classification ne dépend pas du pays source du facteur). Si le Lot 1 n'est pas livré, ce point est sans objet et 0012 reste sur le schéma 0004 actuel.

---

## 4. Modèle de données / migrations

**Nouvelle migration : `src-tauri/migrations/vault/0012_add_be_factors.sql`.** Aucun changement de schéma (DDL) : la table `emission_factor` et `emission_factor_grid_variant` existent déjà (0004). On **seede uniquement** un nouveau millésime. Squelette (valeurs détaillées en §5) :

```sql
-- Référentiel belge facteursdemissionco2.be (catégorie « transport de personnes »).
-- Porté sous le year synthétique 12026 (= variante belge du millésime 2026) afin de
-- NE PAS modifier les millésimes 2025/2026 livrés (reproductibilité R9). Sélectionné
-- par profil via profile_settings.factor_year = 12026.
--
-- POURQUOI un nouveau millésime et pas une colonne country : la table emission_factor
-- est clé (id, year) sans dimension pays ; le chargement (factor_maps.rs) filtre sur
-- year seul. Réutiliser l'axe year est le mécanisme R9-safe déjà éprouvé par 0011.
--
-- ARBRE DE DÉCISION (cf. documentation/calcul-impact-co2.md) :
--   * client / établissement belge (diagnostic fédéral, PDE bruxellois) -> 12026 ;
--   * sinon -> 2026 (Base Carbone / DEFRA, aligné GHG Protocol).
-- facteursdemissionco2.be est la référence de facto (CO2logic + EnergieID) : il
-- n'existe pas de base d'État belge équivalente à la Base Carbone française.
--
-- REPLI DOCUMENTÉ : pour les modes sans facteur belge publié, on recopie la valeur
-- 2026 (ADEME/DEFRA/SNCF) en marquant la source « facteursdemissionco2.be (repli
-- Base Carbone) » pour la traçabilité de l'audit.

INSERT OR IGNORE INTO emission_factor (id, year, label, value, unit, category, scope, is_param, source) VALUES
 -- … 26 lignes, year = 12026, voir §5 pour les valeurs et sources exactes …
 ;

-- Variantes réseau électrique pour le millésime belge (car_ev).
INSERT OR IGNORE INTO emission_factor_grid_variant (mode_id, year, country, value) VALUES
 ('car_ev', 12026, 'FR', 0.09),
 ('car_ev', 12026, 'BE', 0.11),
 ('car_ev', 12026, 'DE', 0.18),
 ('car_ev', 12026, 'EU', 0.13);

-- PAS de UPDATE profile_settings ici : le millésime belge n'est PAS le défaut global.
-- Il est opt-in par profil (sélecteur UI, T3). On ne bascule donc aucun profil
-- existant automatiquement (contrairement à 0011 qui bascule 2025 -> 2026).
```

Points clés du modèle :
- **`year = 12026`** : entier ; aucune contrainte CHECK ne l'interdit (vérifié : `profile_settings` 0010 n'a pas de CHECK sur `factor_year` ; `emission_factor` n'a pas de CHECK sur `year`). Les valeurs `unit`/`category` restent dans les CHECK existants (0004:20-23).
- **`is_param`** : conserver `is_param = 1` pour `car_ev`, `ebike`, `escooter`, `scooter_elec`, et les modes `air` (comme en 2026) pour préserver le mécanisme variante réseau / forçage radiatif.
- **Forçage radiatif aérien** : `radiative_forcing_factor(factor_year)` (`co2_calculator.rs:14-20`) retourne `1.7` pour `year >= 2026`. **`12026 >= 2026` ⇒ RF = 1,7**, cohérent avec 2026. Les facteurs `air` belges (s'il y en a) ou de repli doivent donc inclure le forçage 1,7 dans leur `value`, exactement comme 2026.

---

## 5. Spécification détaillée

### 5.1 Choix de l'identifiant de millésime

Deux options ont été pesées :

| Option | `year` | Avantage | Inconvénient |
|---|---|---|---|
| Année calendaire (ex. 2027) | `2027` | Lisible | Fausse l'idée de « millésime temporel » : 12026 n'est pas plus récent que 2026, c'est une variante géographique du même millésime. Et `radiative_forcing_factor` renverrait quand même 1,7 (OK). |
| **Année synthétique préfixée** | **`12026`** | Encode « variante belge de 2026 » sans collision avec un futur 2027 réel ; `12026 >= 2026` ⇒ RF 1,7 correct | Convention à documenter |

**Retenu : `12026`** (préfixe `1` = pays belge, suffixe `2026` = millésime de base). Convention extensible (ex. `12027` = belge millésime 2027). À documenter dans `calcul-impact-co2.md` §8.

### 5.2 Mapping des facteurs (référentiel belge 12026)

Périmètre couvert par facteursdemissionco2.be : **« transport de personnes »** (voiture, transports en commun, rail, modes actifs). Les valeurs ci-dessous suivent la structure des 26 `id` de 2026 ; **les valeurs numériques exactes doivent être relevées sur facteursdemissionco2.be au moment de l'implémentation** (la table ci-dessous donne la règle de remplissage, pas des chiffres à recopier aveuglément). Règle :

| `mode_id` | Origine de la valeur 12026 | `source` à inscrire | Notes |
|---|---|---|---|
| `train_sncb` | **facteursdemissionco2.be** (rail BE) | `facteursdemissionco2.be (SNCB)` | déjà belge ; aligner sur la valeur du portail belge |
| `bus` | **facteursdemissionco2.be** (bus BE : De Lijn / TEC / STIB-MIVB) | `facteursdemissionco2.be` | remplace la valeur ADEME 2026 (0,113) |
| `metro_tram` | **facteursdemissionco2.be** (métro/tram BE : STIB-MIVB) | `facteursdemissionco2.be` | mix électrique belge ⇒ valeur propre |
| `coach` | **facteursdemissionco2.be** si publié, sinon repli 2026 | `facteursdemissionco2.be` ou `… (repli Base Carbone)` | |
| `public_transport` (mix) | moyenne pondérée recalculée sur les modes BE ci-dessus | `moyenne pondérée interne (modes facteursdemissionco2.be)` | recalculer cohérent avec bus/metro_tram BE |
| `car_petrol`, `car_diesel`, `car_average`, `car_phev`, `taxi` | **facteursdemissionco2.be** (parc BE) si publié, sinon repli 2026 ADEME | `facteursdemissionco2.be` ou `… (repli Base Carbone)` | véhicule thermique ≈ FR ; vérifier |
| `car_ev` (base) + variante BE | **mix réseau belge** (≈0,11) — déjà aligné en 2026 | `facteursdemissionco2.be / mix réseau BE` | variante BE prioritaire via `grid_country` |
| `walk`, `bike` | 0,0 (convention) | `convention` | inchangé |
| `ebike`, `escooter`, `scooter_elec` | facteursdemissionco2.be si publié, sinon repli 2026 | `facteursdemissionco2.be` ou `… (repli Base Carbone)` | |
| `train_ter`, `train_hs_fr`, `train_eurostar`, `train_thalys` | **repli 2026** (modes FR/international, hors périmètre BE) | `… (repli Base Carbone)` / `DEFRA (repli)` | conservés pour les trajets transfrontaliers |
| `plane_domestic/short/medium/long` | **repli 2026 DEFRA** (forçage 1,7 inclus) | `DEFRA/DESNZ (repli)` | RF 1,7 car `12026 >= 2026` |
| `office_day`, `home_day` | facteursdemissionco2.be (énergie bâtiment BE) si publié, sinon repli 2026 | `facteursdemissionco2.be` ou `… (repli Base Carbone)` | mix électrique/chaleur belge |

**Invariant de complétude** : le seed 12026 DOIT contenir **les 26 `id`** présents en 2026. Sinon un trajet sur un mode non seedé déclencherait le fallback `car_average` (R6, `is_estimated = true`) — comportement dégradé non souhaité pour un client belge.

### 5.3 Arbre de décision (à documenter, T5)

```
Le client / l'établissement reporte-t-il en Belgique
(diagnostic fédéral mobilité, PDE bruxellois COBRACE,
 ou exigence client BE) ?
  ├─ OUI → factor_year = 12026 (facteursdemissionco2.be)
  │         · facteurs belges en priorité (CO2logic + EnergieID)
  │         · repli documenté Base Carbone/DEFRA pour modes non couverts
  └─ NON → factor_year = 2026 (Base Carbone ADEME / DEFRA / SNCF)
            · aligné GHG Protocol / BEGES (cadre FR/international)
```

### 5.4 Edge cases

- **Profil sans row `profile_settings`** : lazy default ⇒ `factor_year = 2026` (jamais belge par défaut). Le belge est strictement opt-in.
- **Jours déjà encodés** : leur `co2_kg` + `factor_year` sont figés dans `presence_trip` (R9). Basculer un profil vers 12026 n'affecte que les **futurs** jours ou les jours **ré-édités** ; les anciens restent reproductibles avec leur ancien `factor_year`.
- **Forçage radiatif** : un jour aérien encodé sous 12026 avec RF désactivé sera divisé par `radiative_forcing_factor(12026) = 1,7` — identique à 2026. Cohérent.
- **Variante `car_ev`** : avec `grid_country = 'BE'`, la valeur de la variante `(car_ev, 12026, 'BE')` prime sur la base (mécanisme inchangé, `factor_maps.rs` + `compute_trip`).

---

## 6. Critères d'acceptation

- Quand la migration 0012 est appliquée, **`emission_factor` contient 26 lignes avec `year = 12026`** (couverture complète des modes) ET **4 variantes `car_ev` avec `year = 12026`**.
- Quand on lit le référentiel via `list(2025)` et `list(2026)` après 0012, **les valeurs 2025/2026 sont strictement inchangées** (R9 : 0012 n'altère ni 0004 ni 0011).
- Quand un profil a `factor_year = 12026` et encode un trajet `bus`/`metro_tram`/`train_sncb`, **le `co2_kg` figé utilise la valeur belge** (distincte de la valeur 2026 correspondante) et **`is_estimated = false`** (pas de fallback).
- Quand un profil belge encode un trajet sur un mode au **repli** (ex. `train_eurostar`), le calcul réussit avec la valeur 2026 recopiée et la `source` indique explicitement le repli.
- Quand on ouvre la page méthodologie avec un profil sur 12026, **chaque ligne belge affiche `source = facteursdemissionco2.be`** (ou la mention de repli), et le groupe de sources « facteursdemissionco2.be » apparaît.
- Quand on **n'a pas** opté pour le belge, **aucun profil existant n'est basculé** (0012 ne contient pas de `UPDATE profile_settings`).
- **Conformité** : un diagnostic fédéral mobilité / PDE bruxellois peut être produit avec des facteurs belges traçables et une source nationale citée, sans recourir aux facteurs ADEME France par défaut.

---

## 7. Tests à écrire / mettre à jour

**Tests d'intégration Rust** (`src-tauri/src/integration_tests.rs`, base SQLite réelle migrée) :

- `be_referential_seeded_complete` : après migrations, `LibsqlEmissionFactorRepository::list(12026)` renvoie 26 facteurs ; tous les `id` de `list(2026)` sont présents dans `list(12026)`.
- `be_grid_variants_seeded` : `list_grid_variants(12026)` contient `(car_ev, BE)`.
- `be_does_not_mutate_2026` : les `value` de `list(2026)` sont identiques aux constantes attendues (snapshot des valeurs clés `bus = 0.113`, `metro_tram = 0.0044`, `car_ev = 0.11`) — garantit la non-régression R9.
- `be_factors_differ_from_2026` : au moins `bus` et `metro_tram` ont une `value` différente entre 2026 et 12026 (preuve que le belge est bien distinct).

**Tests calcul** (`src-tauri/src/domain/services/co2_calculator.rs`, style `ac1..ac10`) :

- `ac_be_bus_uses_belgian_factor` : `compute_trip` avec `Co2Settings { factor_year: 12026, .. }` et un `FactorMap` chargé du millésime belge ⇒ `co2_kg` = `distance × valeur_bus_BE`, `is_estimated = false`.
- `ac_be_radiative_forcing_is_1_7` : un trajet `air` sous 12026, `include_radiative_forcing = false`, divise bien par 1,7 (couvre `radiative_forcing_factor(12026)`).

**Tests front (vitest)** :

- Mapper/methodology : un `Co2Referential` avec `factorYear = 12026` et des `source` belges se rend sans erreur ; le `SourceGroup` facteursdemissionco2.be est présent dans `SOURCE_GROUPS`.
- Sélecteur settings : choisir « Belgique » écrit `factorYear = 12026` dans le DTO envoyé.

**Commandes** :
```bash
# Backend
cd src-tauri && cargo test
cargo fmt --check && cargo clippy --all-targets -- -D warnings
# Frontend
pnpm test && pnpm typecheck && pnpm lint
```

---

## 8. Points d'attention & pièges

- **R9 (figeage)** : ne JAMAIS éditer `0004_add_co2.sql` ni `0011_add_co2_2026.sql`. Le belge est un **nouveau** millésime (`year = 12026`) dans un **nouveau** fichier `0012`. Les jours encodés conservent leur `co2_kg`/`factor_year` figés dans `presence_trip`.
- **Pas de `UPDATE profile_settings` dans 0012** : contrairement à 0011 (qui bascule 2025→2026 par défaut), le millésime belge est **opt-in**. Le basculer globalement casserait la cohérence des clients non belges et altérerait la reproductibilité perçue.
- **Complétude obligatoire** : seeder les 26 `id`, sinon fallback `car_average` silencieux (R6) ⇒ sous-estimation et `is_estimated` non désiré.
- **`is_param` et variantes** : conserver `is_param = 1` sur `car_ev`/`ebike`/`escooter`/`scooter_elec` et les modes `air`, et seeder les 4 variantes `car_ev` pour 12026, sinon la variante réseau (`grid_country`) ne s'appliquera pas.
- **Forçage radiatif** : `12026 >= 2026` ⇒ RF résolu à **1,7** par `radiative_forcing_factor`. Les facteurs `air` (repli DEFRA) doivent intégrer 1,7 dans leur `value` (comme 2026). Ne pas seeder un facteur air « brut » sans RF.
- **Couches DDD** : T1/T2 = infrastructure (migration + enregistrement). T3 = front + (éventuelle) levée de borne de validation import. T4/T5 = présentation/doc. Aucun changement n'est nécessaire dans `domain/services/co2_calculator.rs` (le calcul est agnostique au pays : il consomme un `factor_year`).
- **Validation import bundle** : vérifier que `json_bundle_codec.rs` / `import_profile_bundle.rs` n'imposent pas `factor_year ∈ {2025, 2026}`. Le wire transporte `factorYear: i32`, mais une borne haute introduite par un commit de durcissement (cf. `0bd3220` sur la validation d'entrée non fiable) bloquerait `12026`. À tester explicitement.
- **i18n** : seuls les libellés du sélecteur (T3) sont à traduire (fr/en). Les `source` de la table méthodologie sont des chaînes backend (non i18n), conformément à la convention `sources.ts` (noms propres affichés tels quels).
- **Mouvance réglementaire** : facteursdemissionco2.be est mis à jour par CO2logic/EnergieID ; à chaque révision majeure, créer un **nouveau** millésime (ex. `12027`) plutôt que d'éditer `12026` — même discipline que les millésimes FR.
- **Ambiguïté pays vs millésime** : `12026` mélange une dimension géographique dans l'axe temporel `year`. C'est un compromis assumé (cf. §2) ; le documenter clairement évite la confusion d'un futur mainteneur. Si le Lot 1 ou un lot ultérieur introduit une vraie colonne `country`/`region` sur `emission_factor`, ce choix pourra être migré proprement à ce moment-là.

---

## 9. Références réglementaires

- **facteursdemissionco2.be** — référentiel belge de facteurs d'émission (porté par CO2logic et EnergieID), catégorie « transport de personnes ». Référence de facto en l'absence de base d'État belge équivalente à la Base Carbone.
- **Diagnostic fédéral mobilité** (SPF Mobilité et Transports) — obligation triennale de relevé des déplacements domicile-travail pour les unités d'établissement de plus de 100 travailleurs (loi-programme du 8 avril 2003 et arrêtés d'exécution).
- **Plan de Déplacements d'Entreprise (PDE) — Région de Bruxelles-Capitale** — ordonnance COBRACE (Code bruxellois de l'air, du climat et de la maîtrise de l'énergie, 2 mai 2013) ; obligation pour les entreprises occupant plus de 100 travailleurs sur un même site, suivi par Bruxelles Environnement.
- **SNCB / NMBS** — facteur ferroviaire belge (déjà intégré, `train_sncb`).
- **GHG Protocol — Corporate Standard & Scope 3 (catégorie 7, Employee commuting)** — cadre méthodologique de référence pour le déplacement domicile-travail (s'applique quel que soit le référentiel de facteurs choisi).
- **ADEME Base Carbone® / Base Empreinte®** et **DEFRA/DESNZ GHG conversion factors** — référentiels de repli (millésime 2026) pour les modes non couverts par facteursdemissionco2.be.
- Documentation interne : `documentation/calcul-impact-co2.md` (§8 millésimes, §10 sources, règles R1-R9).
