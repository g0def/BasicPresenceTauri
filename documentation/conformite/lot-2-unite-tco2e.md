# Lot 2 — Unité tCO2e en parallèle du kgCO2e

> **Statut :** À faire  ·  **Priorité :** P1  ·  **Effort :** faible  ·  **Dépend de :** aucun (indépendant ; le Lot 3 « Consolidation » le consomme)
> **Gap couvert :** (c) Restitution en tonnes équivalent CO2 (tCO2e) en parallèle du kgCO2e existant, exigée par BEGES ch.9, ESRS E1-6 et VSME B3.
> **Débloque :** Format de restitution BEGES (chapitre 9), ESRS E1-6 (publication en tCO2e), VSME module B3, et l'export consolidé du Lot 3 (qui additionne des totaux entité en tonnes).

---

## 1. Contexte & objectif

Aujourd'hui BasicPresence ne produit que des valeurs en **kgCO2e** : en base, à l'écran (résumé mensuel, calendrier, charts stats), et à l'export ODS. Or les cadres réglementaires de restitution exigent la **tonne équivalent CO2 (tCO2e)** : le BEGES (chapitre 9 « format de restitution »), l'ESRS E1-6 (déclaration des émissions brutes Scope 1/2/3 en tCO2e) et le standard VSME (module B3). L'objectif de ce lot est d'ajouter une sortie **tCO2e en parallèle** du kgCO2e (division par 1000), sans jamais altérer le calcul interne qui reste en kg. Résultat attendu : l'utilisateur peut afficher et exporter ses empreintes en tCO2e (avec les arrondis réglementaires adéquats), tout en conservant l'affichage kgCO2e pour le suivi quotidien.

## 2. État actuel du code (point de départ)

Faits précis tirés de la cartographie et vérifiés dans le code :

- **Unité unique kgCO2e partout.** Le calcul (`Co2Calculator`) et le stockage sont en kgCO2e. Aucune conversion tCO2e n'existe.
- **Stockage figé (R9).** `presence.co2_kg REAL NULL` (dénormalisé) et `presence_trip.co2_kg REAL` (snapshot par segment, avec `factor_year`). Voir `src-tauri/migrations/vault/0004_add_co2.sql:73-90`. Ces colonnes sont **figées au moment de l'encodage** et ne doivent jamais être recalculées rétroactivement.
- **DTO frontend.** `PresenceDto` expose `co2_kg: Option<f64>` (wire `co2Kg`) — `src-tauri/src/application/dto/presence_dto.rs:10-21`. Pas de champ tonnes.
- **Formatage frontend.** `formatCo2(kg, lang)` formate en kg avec `maximumFractionDigits: 1` et suffixe `" kg"` — `src/features/commute/presentation/commute-format.ts:11-17`. Aucune fonction tonnes.
  Usages de `formatCo2` (8 points) :
  - `src/routes/_authenticated.tsx:102` (résumé mensuel en-tête)
  - `src/features/stats/presentation/components/period-bar-chart.tsx:51` (chart barres CO₂, axe Y + tooltip + ligne moyenne)
  - `src/features/presence/presentation/components/presence-calendar.tsx:258` (cellule jour)
  - `src/features/presence/presentation/components/presence-day-dialog.tsx:288` (dialogue jour)
  - `src/features/commute/presentation/pages/commute-list-page.tsx:69` (liste des trajets enregistrés)
- **Agrégation stats.** `PeriodBucket.co2Kg: number` sommé dans `bucketPresences()` — `src/features/stats/domain/aggregate.ts:20-32, 107`. Pas de champ tonnes (la conversion peut se faire au formatage, voir §5).
- **Export ODS.** Deux colonnes CO₂ en kg :
  - Feuille « Présences » : header `labels.co2`, valeur `p.co2_kg` (kg) — `src-tauri/src/infrastructure/export/ods_writer.rs:115, 144-145`.
  - Feuille « Trajets » : header `labels.co2`, valeur `trip.co2_kg` (kg) — `ods_writer.rs:221, 239`.
  Le label `co2` vaut « CO₂ (kg) » (`ExportLabels`, `spreadsheet_exporter.rs:59`).
- **Options d'export.** `ExportOptions` (Rust) et `ExportOptionsDto` n'ont **pas** de champ de choix d'unité — `src-tauri/src/domain/services/spreadsheet_exporter.rs:10-25`, `src-tauri/src/application/dto/export_dto.rs:7-29`. Côté frontend, `ExportOptions` (`src/features/export/domain/entities/export.ts:4-28`) et `OPTION_KEYS` (`src/features/export/presentation/components/export-dialog.tsx:33-40`) listent les flags `includeCarbon`, `includeHours`, etc.
- **Labels export.** `ExportLabels` / `ExportLabelsDto` ont un seul champ `co2` ; pas de `co2_tonne` — `spreadsheet_exporter.rs:49-90`, `export_dto.rs:34-65`, `src/features/export/presentation/build-export-labels.ts:19`.
- **i18n.** Clés sous `export.sheet.cols.co2` = « CO₂ (kg) » (fr) / « CO₂ (kg) » (en), `summary.co2Emitted` (résumé mensuel), `stats.co2Title`. Pas de clé tonnes. Fichiers : `src/core/i18n/locales/{fr,en}/translation.json`.
- **Convention forte (cartographie).** « CO2 : toujours en kgCO2e en interne, jamais en tCO2e ou unités mixtes ». La conversion doit donc rester **en couche présentation/export**, jamais dans le `Co2Calculator`.
- **Dernière migration vault connue : `0011`.** Prochain numéro disponible : `0012`.

## 3. Travail à réaliser

Décision d'architecture (justifiée en §5) : **valeur dérivée à la lecture/affichage/export**, par défaut. Pas de colonne `co2_t` stockée (évite la désynchronisation et reste fidèle à R9 : le snapshot kg reste l'unique source de vérité). Voir §4 pour la justification du « Sans objet » migration.

Tâches ordonnées :

### Backend (Rust)

**T1 — Helper de conversion kg→t (domaine, pur).**
Fichier : `src-tauri/src/domain/services/co2_calculator.rs`.
Ajouter une constante et une fonction publiques, testables, sans I/O :
```rust
/// Conversion kgCO2e → tCO2e (BEGES ch.9 / ESRS E1-6 / VSME B3 exigent la tonne).
/// La conversion vit en présentation/export : le calcul interne reste en kg (R9).
pub const KG_TO_T: f64 = 0.001;

/// kgCO2e → tCO2e sans arrondi (l'arrondi d'affichage relève de la couche présentation).
pub fn kg_to_tonnes(kg: f64) -> f64 {
    kg * KG_TO_T
}
```
Justification de l'emplacement : le calculateur est le module pur du domaine ; on y centralise la constante de conversion comme on y centralise déjà `radiative_forcing_factor` (`co2_calculator.rs:14-20`).

**T2 — Option d'export « unité tonnes » (port + DTO).**
- `src-tauri/src/domain/services/spreadsheet_exporter.rs` : ajouter à `ExportOptions` le champ `pub co2_unit_tonnes: bool` (par défaut le wire le fixe ; `false` = kg, rétrocompat).
- `src-tauri/src/application/dto/export_dto.rs` : ajouter `pub co2_unit_tonnes: bool` à `ExportOptionsDto` et le mapper dans `From<ExportOptionsDto>`.
- Ajouter à `ExportLabels` (et `ExportLabelsDto` + son `From`) le champ `pub co2_tonne: String` (libellé colonne tonnes, ex. « CO₂ (t) »).

**T3 — Émettre la colonne CO₂ selon l'unité dans l'ODS.**
Fichier : `src-tauri/src/infrastructure/export/ods_writer.rs`.
- Feuille « Présences » (`build_presences_sheet`, ~ligne 114-119 pour le header, 143-149 pour les valeurs) : si `options.co2_unit_tonnes`, écrire le header `labels.co2_tonne` et la valeur `kg_to_tonnes(co2)` arrondie à 3 décimales ; sinon comportement actuel (`labels.co2`, `co2` en kg).
- Feuille « Trajets » (`build_trips_sheet`, header ~ligne 221, valeur ~ligne 239) : même logique sur `trip.co2_kg`.
- Utiliser un helper local `round_t(x: f64) -> f64 { (x * 1000.0).round() / 1000.0 }` (3 décimales, voir §5) appliqué uniquement à la valeur tonnes ; le kg reste inchangé (`maximumFractionDigits` géré côté tableur via la valeur brute, pas d'arrondi destructif côté kg).

### Frontend (TypeScript/React)

**T4 — Helpers de formatage tonnes.**
Fichier : `src/features/commute/presentation/commute-format.ts`.
Ajouter, à côté de `formatCo2` :
```ts
/** Conversion kgCO2e → tCO2e (BEGES ch.9 / ESRS E1-6 / VSME B3). */
export const KG_TO_T = 0.001;

/** Format a footprint in tCO2e, locale-aware, 3 decimals (réglementaire). */
export function formatCo2AsTonnes(kg: number, lang: string): string {
  const locale = lang === "en" ? "en-US" : "fr-FR";
  const value = new Intl.NumberFormat(locale, {
    maximumFractionDigits: 3,
  }).format(kg * KG_TO_T);
  return `${value} t`;
}
```
Ne **pas** modifier `formatCo2` (rétrocompat : le suivi quotidien reste en kg).

**T5 — Choix d'unité à l'export (UI).**
- `src/features/export/domain/entities/export.ts` : ajouter `co2UnitTonnes: boolean` à `ExportOptions` (wire 1:1 avec `ExportOptionsDto.co2_unit_tonnes`) et `co2UnitTonnes: false` à `DEFAULT_EXPORT_OPTIONS`. Ajouter `co2Tonne: string` à `ExportLabels`.
- `src/features/export/presentation/build-export-labels.ts` : ajouter `co2Tonne: t("export.sheet.cols.co2Tonne")`.
- `src/features/export/presentation/components/export-dialog.tsx` : ajouter `"co2UnitTonnes"` dans `OPTION_KEYS` (rendu via la même boucle de cases à cocher, libellé `export.fields.co2UnitTonnes`). Optionnel UX : ne montrer la case que si `includeCarbon || includeTrips` est coché.
- Vérifier le passage du flag dans le hook `use-export.ts` et le repository `tauri-export.repository.ts` : ils transmettent l'objet `ExportOptions` tel quel (camelCase) ; aucun mapping manuel à faire au-delà de l'ajout du champ.

**T6 — Affichage tCO2e dans les vues agrégées (résumé mensuel + chart stats).**
Décision : afficher **les deux unités** là où la lecture réglementaire est utile, et garder le kg seul là où c'est du suivi quotidien (cellule jour, dialogue jour). Concrètement :
- `src/routes/_authenticated.tsx:102` (résumé mensuel) : afficher kg ET, en complément discret (tooltip ou ligne secondaire), `formatCo2AsTonnes(monthlyStats.co2, lang)`.
- `src/features/stats/presentation/components/period-bar-chart.tsx` : pour le chart `metric === "co2Kg"`, le tooltip peut afficher les deux unités (`formatCo2` + `formatCo2AsTonnes`) ; l'axe Y reste en kg (lisibilité, valeurs souvent < 1 t). Aucun changement à `PeriodBucket` ni à `bucketPresences()` : la conversion est dérivée au formatage à partir de `b.co2Kg`.
- Ne pas toucher `presence-calendar.tsx:258` ni `presence-day-dialog.tsx:288` (suivi quotidien, kg pertinent).

**T7 — i18n.**
Fichiers : `src/core/i18n/locales/fr/translation.json` et `.../en/translation.json`.
Ajouter sous le bloc `export.sheet.cols` la clé `co2Tonne` :
- fr : `"co2Tonne": "CO₂ (t)"`
- en : `"co2Tonne": "CO₂ (t)"`
Ajouter sous `export.fields` :
- fr : `"co2UnitTonnes": "Exporter le CO₂ en tonnes (tCO2e)"`
- en : `"co2UnitTonnes": "Export CO₂ in tonnes (tCO2e)"`
Optionnel (T6) : clés `summary.co2EmittedTonnes` / `stats.co2TitleTonnes` ou un suffixe d'unité réutilisable. Toute clé ajoutée en `fr` **doit** l'être en `en` (sinon export/rendu cassé — convention §3.6 checklist).

### Documentation

**T8 — Documenter la conversion et l'arrondi.**
Fichier : `documentation/calcul-impact-co2.md`.
Ajouter une sous-section (§ unités/restitution) précisant : interne kgCO2e (immuable, R9) ; restitution dérivée tCO2e = kg × 0,001 ; arrondi tCO2e à 3 décimales à l'affichage/export ; cadres exigeant la tonne (BEGES ch.9, ESRS E1-6, VSME B3).

## 4. Modèle de données / migrations

**Sans objet (recommandé).** La valeur tCO2e est **dérivée** (`co2_kg × 0,001`) à la lecture/affichage/export ; aucune colonne `co2_t` n'est ajoutée.

Justification :
- **Cohérence / R9.** Le snapshot kgCO2e dans `presence_trip.co2_kg` (figé avec `factor_year`) reste l'unique source de vérité. Une colonne `co2_t` stockée introduirait un risque de désynchronisation (deux nombres à garder cohérents) sans bénéfice : la division par 1000 est exacte et déterministe.
- **Pas de backfill.** Une colonne stockée imposerait un `UPDATE ... SET co2_t = co2_kg/1000` rétroactif, opération à éviter sur des lignes figées.

**Bascule éventuelle (si besoin d'arrondi réglementaire stocké).** Si un audit futur exige que la valeur tonnes arrondie soit **gelée** au moment de l'encodage (et non recalculée), créer alors `src-tauri/migrations/vault/0012_add_co2_tonnes.sql` :
```sql
-- Colonne dérivée GELÉE (uniquement si un arrondi réglementaire figé est requis).
-- Par défaut, ce lot N'AJOUTE PAS cette colonne (valeur dérivée à la lecture).
ALTER TABLE presence ADD COLUMN co2_t REAL;
ALTER TABLE presence_trip ADD COLUMN co2_t REAL;
UPDATE presence      SET co2_t = ROUND(co2_kg / 1000.0, 3) WHERE co2_kg IS NOT NULL;
UPDATE presence_trip SET co2_t = ROUND(co2_kg / 1000.0, 3);
```
Cette option n'est **pas** retenue par défaut ; elle est documentée pour traçabilité de la décision.

## 5. Spécification détaillée

### Formule de conversion
`tCO2e = kgCO2e × 0,001` (exact). Constante unique : `KG_TO_T = 0.001` (Rust et TS).

### Politique d'arrondi

| Unité  | Décimales | Où | Pourquoi |
|--------|-----------|----|----------|
| kgCO2e | 1 (existant) | affichage/résumé/charts | suivi quotidien, conservé tel quel (`formatCo2`) |
| tCO2e  | 3           | affichage tonnes + export ODS tonnes | les émissions individuelles sont faibles (souvent 0,001–0,5 t/mois) ; 3 décimales évitent de tout afficher « 0,0 t » et collent à la granularité usuelle BEGES/ESRS |

Règle : l'arrondi est **uniquement** d'affichage/export. Les valeurs en base et le calcul restent non arrondis (`presence.co2_kg`, `presence_trip.co2_kg`). La somme se fait **en kg** (haute précision flottante), la conversion tonnes intervient **après** la somme (évite l'accumulation d'erreurs d'arrondi).

### Affichage par surface (cohérence kg↔t)

| Surface | Fichier | kgCO2e | tCO2e |
|---------|---------|--------|-------|
| Résumé mensuel | `src/routes/_authenticated.tsx:102` | oui (principal) | oui (complément/tooltip) |
| Chart barres CO₂ | `period-bar-chart.tsx` | oui (axe Y + tooltip) | tooltip (complément) |
| Cellule calendrier | `presence-calendar.tsx:258` | oui | non (suivi quotidien) |
| Dialogue jour | `presence-day-dialog.tsx:288` | oui | non |
| Liste trajets enregistrés | `commute-list-page.tsx:69` | oui | non |
| Export ODS « Présences » | `ods_writer.rs:115,144` | si unité=kg | si unité=tonnes |
| Export ODS « Trajets » | `ods_writer.rs:221,239` | si unité=kg | si unité=tonnes |

### Comportement de l'option d'export
- `co2_unit_tonnes = false` (défaut) → comportement actuel inchangé (header « CO₂ (kg) », valeurs en kg). **Rétrocompatibilité totale.**
- `co2_unit_tonnes = true` → header « CO₂ (t) » (`labels.co2_tonne`), valeurs = `ROUND(kg × 0,001, 3)` sur les feuilles « Présences » et « Trajets ».
- L'option n'a d'effet que si `include_carbon` (Présences) et/ou `include_trips` (Trajets) sont actifs ; sinon elle est inerte.

### Edge cases
- `presence.co2_kg = NULL` (vacances/jour férié, jours pré-CO2) : reste vide dans l'export quelle que soit l'unité (la cellule n'est pas écrite — cf. `if let Some(co2) = p.co2_kg`).
- Valeur 0,0 kg (marche/vélo) : `0` t affiché « 0 t ».
- Très petites valeurs (< 0,0005 t) : arrondi 3 décimales → « 0 t » ; acceptable (le détail kg reste accessible via l'export en kg).
- Locale : séparateur décimal géré par `Intl.NumberFormat` (fr → virgule, en → point). Côté ODS, les nombres sont écrits comme nombres réels (pas de formatage texte), le tableur applique la locale du lecteur.

## 6. Critères d'acceptation

- **CA1.** Quand `co2_unit_tonnes = false`, l'export ODS est **identique octet-pour-octet en logique** au comportement actuel (header « CO₂ (kg) », valeurs kg). Rétrocompat prouvée par le test existant `writes_a_readable_multi_sheet_ods`.
- **CA2.** Quand `co2_unit_tonnes = true` et `include_carbon = true`, la feuille « Présences » a un header « CO₂ (t) » et, pour un jour à 3200 kgCO2e (impossible en pratique mais déterministe), affiche 3,2 ; pour 1,1 kg, affiche 0,001.
- **CA3.** Quand `co2_unit_tonnes = true` et `include_trips = true`, la feuille « Trajets » convertit `trip.co2_kg` en tonnes arrondies à 3 décimales.
- **CA4.** `kg_to_tonnes(1234.5) == 1.2345` et `formatCo2AsTonnes(1100, "fr") == "1,1 t"` (1100 kg = 1,1 t).
- **CA5.** Le calcul interne (`Co2Calculator::compute_day`) reste en kgCO2e : aucune valeur stockée ni snapshot n'est modifié. Les tests `ac1..ac10` passent sans changement.
- **CA6.** L'UI résumé mensuel affiche la valeur tonnes en complément (ex. tooltip), sans casser l'affichage kg principal.
- **CA7.** Les clés i18n `export.sheet.cols.co2Tonne` et `export.fields.co2UnitTonnes` existent en **fr ET en**.
- **CA8.** Conformité : un export en tonnes fournit une restitution exploitable telle quelle pour BEGES ch.9 / ESRS E1-6 / VSME B3 (unité tCO2e, 3 décimales).

## 7. Tests à écrire / mettre à jour

### Rust
- **`co2_calculator.rs` (mod tests).** Ajouter `fn kg_to_tonnes_divides_by_thousand()` : `assert_eq!(kg_to_tonnes(1234.5), 1.2345)` et `assert_eq!(kg_to_tonnes(0.0), 0.0)`.
- **`ods_writer.rs` (mod tests).** Mettre à jour `all_options()` pour le nouveau champ `co2_unit_tonnes` (mettre `false` afin que `writes_a_readable_multi_sheet_ods` reste vert — CA1). Mettre à jour `sample_labels()` avec `co2_tonne: "CO₂ (t)".to_string()`.
  Ajouter `fn exports_co2_in_tonnes_when_unit_is_tonnes()` : monter un `ExportData` avec `presence.co2_kg = Some(3200.0)` et `trip.co2_kg = 1.1`, options `co2_unit_tonnes = true`, ré-ouvrir avec `calamine` et asserter header « CO₂ (t) » et valeurs `3.2` (Présences) / `0.001` (Trajets, arrondi 3 déc.).
- Commande : `cargo test --manifest-path src-tauri/Cargo.toml`. Vérifier aussi `cargo fmt --check` et `cargo clippy --all-targets -- -D warnings`.

### TypeScript (vitest)
- **`src/features/commute/presentation/commute-format.test.ts`** (nouveau fichier colocalisé) :
  - `formatCo2AsTonnes(1100, "fr")` → `"1,1 t"` ; `formatCo2AsTonnes(1100, "en")` → `"1.1 t"`.
  - `formatCo2AsTonnes(0, "fr")` → `"0 t"`.
  - `formatCo2AsTonnes(2345, "fr")` → `"2,345 t"` (3 décimales).
  - (sanity) `formatCo2(4800, "fr")` inchangé → `"4 800 kg"`.
- Commande : `pnpm test` (ou `pnpm vitest run`). Vérifier `pnpm typecheck` et `pnpm lint` (champ `co2UnitTonnes` ajouté partout, pas de `any`).

## 8. Points d'attention & pièges

- **R9 / figeage des jours.** Ne jamais recalculer ni réécrire `presence.co2_kg` / `presence_trip.co2_kg`. La tonne est strictement dérivée. C'est la raison du choix « valeur dérivée » (§4).
- **Calcul interne intouchable.** Convention forte : kgCO2e en interne, jamais tCO2e. La conversion ne doit apparaître **que** en présentation (`commute-format.ts`, charts, résumé) et export (`ods_writer.rs`). Ne rien introduire dans `compute_trip`/`compute_day`.
- **Somme avant conversion.** Toujours sommer en kg puis convertir (précision flottante — cf. risque AC-10 / arrondi de la cartographie).
- **Rétrocompat de l'export.** Le défaut `co2_unit_tonnes = false` garantit qu'aucun export existant ne change. Le test ODS existant doit rester vert (CA1).
- **i18n fr+en obligatoire.** Une clé `fr` sans son équivalent `en` casse l'export ou rend un libellé vide (convention §3.6 checklist). Ajouter `co2Tonne` et `co2UnitTonnes` dans les deux fichiers.
- **Couches DDD.** `KG_TO_T`/`kg_to_tonnes` dans le domaine (`co2_calculator.rs`, pur) ; l'option d'unité traverse `ExportOptions` (domaine port) ↔ `ExportOptionsDto` (application) ↔ `ods_writer.rs` (infra) ↔ `export.ts`/`export-dialog.tsx` (frontend). Respecter le sens des dépendances (camelCase au wire, snake_case en Rust).
- **Non-régression UI.** Ne pas remplacer le kg par la tonne dans les vues de suivi quotidien (cellule, dialogue) : pour des valeurs < 1 t, « 0,003 t » est moins lisible que « 3 kg ».
- **Mouvance réglementaire.** La granularité d'arrondi (3 décimales) est un choix de restitution, pas une valeur figée par la loi ; documenter dans `calcul-impact-co2.md` pour pouvoir l'ajuster sans casser le calcul.

## 9. Références réglementaires

- **BEGES — Bilan d'émissions de gaz à effet de serre** (art. L229-25 Code de l'environnement ; méthode réglementaire ADEME). Chapitre 9 « Format de restitution » : restitution des émissions en **tCO2e**. Méthode et outils : https://bilans-ges.ademe.fr/
- **ESRS E1 — Climate change** (Règlement délégué (UE) 2023/2772, normes ESRS au titre de la directive CSRD 2022/2464). Datapoint **E1-6** « Gross Scopes 1, 2, 3 and Total GHG emissions » : publication en **tCO2eq**. https://eur-lex.europa.eu/eli/reg_del/2023/2772/oj
- **VSME — Voluntary SME standard** (EFRAG, standard volontaire pour PME non cotées). Module Basic, indicateur **B3 « Energy and GHG emissions »** : émissions en **tonnes de CO2 équivalent**. https://www.efrag.org/ (VSME Exposure Draft / Standard)
- **GHG Protocol Corporate Standard** (unité de reporting recommandée : tCO2e ; PRG/GWP 100 ans GIEC). https://ghgprotocol.org/corporate-standard
