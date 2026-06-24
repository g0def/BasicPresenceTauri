# Lot 6 — Référence du cadre normatif (en-tête d'export + page méthodologie)

> **Statut :** À faire  ·  **Priorité :** P2  ·  **Effort :** faible  ·  **Dépend de :** aucun (léger) ; **alimente** le Lot 3 (export auditable) et **complète** la page méthodologie existante (commit `fc2c18c`).
> **Gap couvert :** (f) Aucune référence au cadre normatif dans l'export (méthode, version de la base de facteurs, PRG 100 ans GIEC, année de reporting / de référence, référentiels BEGES / ESRS / GHG Protocol / ISO 14064-1).
> **Débloque :** Exigence de publication CSRD (ESRS E1-6 — décrire méthodologie, facteurs, sources, PRG) et BEGES V5 (ch. 9 — méthode et facteurs documentés). Sans cette traçabilité normative, un OTI / CAC refuse l'export comme non démontrable.

---

## 1. Contexte & objectif

Les exports `.ods` et la page méthodologie in-app présentent aujourd'hui des chiffres CO₂ **sans déclarer le cadre normatif qui les produit**. Or BEGES V5 (ch. 9) et ESRS E1-6 exigent que tout bilan publié précise la **méthode** (distance-based / average-data, GHG Protocol Corporate Standard 2004), la **version de la base de facteurs** (Base Empreinte / DEFRA + millésime app), le **PRG sur 100 ans** (valeurs GIEC AR6), les **années de reporting et de référence**, et le **référentiel** suivi. L'objectif de ce lot est purement déclaratif : injecter ces métadonnées (1) dans une **nouvelle feuille « Méthodologie »** en tête du classeur d'export, et (2) dans une **nouvelle Card « Cadre normatif »** de la page méthodologie. Aucun changement de calcul, aucune migration de schéma : on documente l'existant, on ne le modifie pas.

## 2. État actuel du code (point de départ)

Faits vérifiés dans la base :

- **Page méthodologie** : `src/features/methodology/presentation/pages/methodology-page.tsx`. Elle affiche déjà : intro (`methodology.intro`, avec `factorYear` + `radiativeForcing`), Card Formules, Card Paramètres, Card Tableau des facteurs (colonnes Mode / Valeur+unité / **Périmètre** = `methodology.table.scope`, ligne 223-225 / **Source** = `methodology.table.source`), Card Exemples, Card Sources (`SOURCE_GROUPS` de `sources.ts`), et un `<details>` Transparence (5 items). Le référentiel vient du hook `useCo2Referential()` (ligne 27) qui expose `referential.factorYear` et `referential.radiativeForcing`.
- **DTO référentiel exposé au front** : `src-tauri/src/application/dto/emission_factor_dto.rs:32-38` → `Co2ReferentialDto { factor_year: i32, radiative_forcing: f64, factors: Vec<EmissionFactorDto> }`. **Aucun champ de cadre normatif.** Construit par `src-tauri/src/application/use_cases/list_co2_referential.rs:26-62` ; `factor_year = Co2Settings::default().factor_year` (= 2026, voir `co2_settings.rs:29`).
- **Export — port** : `src-tauri/src/domain/services/spreadsheet_exporter.rs`. `ExportData { days: Vec<DayExport> }` (ligne 38-42) ne porte **aucune métadonnée d'en-tête**. `ExportLabels` (ligne 48-90) liste 27 libellés traduits, **tous fournis par le front**, jamais codés en dur côté Rust. Le trait `SpreadsheetExporter::write_workbook` (ligne 116-124) reçoit `data, options, labels, path`.
- **Export — implémentation ODS** : `src-tauri/src/infrastructure/export/ods_writer.rs`. `write_workbook` (ligne 71-96) pousse `build_presences_sheet` puis, sous condition, `build_tasks_sheet`, `build_trips_sheet`, `build_notes_sheet`. **Aucune feuille de métadonnées.** Styles enregistrés via `register_styles` (ligne 50-69 : `header` bold, `date`, `datetime`).
- **Export — use case** : `src-tauri/src/application/use_cases/export_profile_data.rs:31-80`. Rassemble les jours et appelle l'exporteur ligne 71-72. **Ne charge ni `Co2Settings` ni le référentiel** ; n'a donc aujourd'hui aucune source pour `factor_year`, RF, grid_country, etc.
- **Export — DTO** : `src-tauri/src/application/dto/export_dto.rs`. `ExportOptionsDto` (6 flags, ligne 9-16), `ExportLabelsDto` (27 champs, ligne 36-65) avec `From` mapping vers le domaine, `ExportSummaryDto` (ligne 105-114).
- **Export — labels front** : `src/features/export/presentation/build-export-labels.ts:10-41` construit `ExportLabels` depuis les clés `export.sheet.*` et `presence.types.*`. L'interface TS miroir est `src/features/export/domain/entities/export.ts:33-62`.
- **Export — commande Tauri** : `src-tauri/src/presentation/commands/export.rs:12-25` (`export_profile_data`, session-gated).
- **i18n** : `src/core/i18n/locales/fr/translation.json` et `en/translation.json`. Sous `methodology.*` : clés `menuItem, title, back, intro, introGeneric, loading, error, noProfile, formula, params, table, examples, sources, transparency`. Sous `export.sheet.*` : `presences, tasks, trips, notes, yes, no, cols.{date,type,co2,estimated,hours,created,updated,title,description,minutes,color,order,mode,distance,roundTrip,occupants,factorYear,note}`. **Aucune clé `methodology.framework.*` ni `export.sheet.cols.methodology*`.**
- **Forçage radiatif** : `radiative_forcing_factor(factor_year)` dans `co2_calculator.rs:14-20` → `1.7` si `year >= 2026`, sinon `1.9`. Déjà exposé via `Co2ReferentialDto.radiative_forcing`.
- **Sources des facteurs (seed 2026)** : `src-tauri/migrations/vault/0011_add_co2_2026.sql:30-56`. Attributions présentes : `ADEME Base Carbone`, `DEFRA/DESNZ 2024`, `SNCF Open Data (périmètre complet 2024)`, `SNCB`, `CIBSE TM46 / Circular Ecology`, `moyenne pondérée interne`, etc. Le champ SQL `scope` désigne le **périmètre cycle de vie** (ex. `usage(WtW)+fabrication`), **PAS** le Scope GHG — la page méthodologie l'affiche correctement sous le libellé « Périmètre ».
- **Convention forte** : le backend **n'i18n jamais** ; les libellés viennent du front. Les facteurs ne sont **jamais** codés en dur, ils vivent en SQL versionnés par `year`.

## 3. Travail à réaliser

Tâches ordonnées. Ce lot est **additif** : nouveaux champs DTO, nouvelle feuille, nouvelle Card. Aucune suppression, aucune migration.

### Backend (Rust)

**T1 — Définir une struct de métadonnées normatives (domaine).**
Fichier : `src-tauri/src/domain/services/spreadsheet_exporter.rs`.
Ajouter une struct `ExportMetadata` (valeurs textuelles déjà résolues côté front pour rester i18n-neutre) :

```rust
/// Cadre normatif déclaré dans l'en-tête d'export. Toutes les valeurs sont des
/// chaînes déjà localisées par le frontend (le backend n'i18n jamais).
#[derive(Debug, Clone)]
pub struct ExportMetadata {
    /// Année du millésime de facteurs résolu (ex. "2026"). Donnée objective.
    pub factor_year: i32,
    /// Multiplicateur de forçage radiatif aérien en vigueur (ex. 1.7).
    pub radiative_forcing: f64,
    /// Pays grille électrique du profil (ex. "BE").
    pub grid_country: String,
    /// Couples clé/valeur du bloc « Méthodologie », déjà traduits par le front
    /// (méthode, base de facteurs, PRG, référentiels, années, etc.).
    pub rows: Vec<(String, String)>,
    /// Titre de la feuille (ex. "Méthodologie"), traduit par le front.
    pub sheet_title: String,
    /// En-têtes des deux colonnes (ex. "Élément", "Valeur"), traduits.
    pub key_header: String,
    pub value_header: String,
}
```

Étendre `ExportData` pour porter ce bloc, **optionnel** (rétro-compat des tests existants) :

```rust
pub struct ExportData {
    pub days: Vec<DayExport>,
    /// Bloc « Méthodologie » écrit en première feuille quand présent.
    pub metadata: Option<ExportMetadata>,
}
```

**T2 — Écrire la feuille « Méthodologie » dans l'exporteur ODS.**
Fichier : `src-tauri/src/infrastructure/export/ods_writer.rs`.
Ajouter `fn build_methodology_sheet(meta: &ExportMetadata, styles: &Styles) -> Sheet` : feuille à 2 colonnes (Élément / Valeur), en-tête `styles.header`, une ligne par `(clé, valeur)` de `meta.rows`. Largeurs : col 0 `mm!(55.0)`, col 1 `mm!(95.0)`, `set_header_rows(0, 0)`.
Dans `write_workbook` (ligne 82, **avant** `build_presences_sheet`) : `if let Some(meta) = &data.metadata { wb.push_sheet(build_methodology_sheet(meta, &styles)); }`. La feuille Méthodologie doit être **la première** du classeur.

**T3 — Mettre à jour les sites de construction de `ExportData`.**
Tous les `ExportData { days }` deviennent `ExportData { days, metadata }`. Concerne : `export_profile_data.rs:70`, et les tests d'`ods_writer.rs` (`sample_data`, ligne 288 → ajouter `metadata: None` pour conserver le comportement existant prouvé par `writes_a_readable_multi_sheet_ods`).

**T4 — Étendre le DTO d'export pour recevoir les métadonnées du front.**
Fichier : `src-tauri/src/application/dto/export_dto.rs`.
Ajouter `ExportMetadataDto` (serde camelCase) miroir de `ExportMetadata`, avec `rows: Vec<MetaRowDto { key: String, value: String }>`, et un `From<ExportMetadataDto> for ExportMetadata`. Ajouter le champ au boundary : la commande recevra un argument `metadata: Option<ExportMetadataDto>`.

**T5 — Câbler le use case et la commande.**
Fichiers : `export_profile_data.rs`, `export.rs`.
`ExportProfileDataUseCase::execute` reçoit un paramètre `metadata: Option<ExportMetadataDto>`, le convertit (`.map(Into::into)`) et le pose dans `ExportData { days, metadata }`. La commande `export_profile_data` (export.rs) ajoute le paramètre `metadata` et le transmet. **Le backend ne calcule pas les libellés** : il relaie les couples déjà traduits ; les seules valeurs « dures » objectives (factor_year, radiative_forcing, grid_country) viennent aussi du front, qui les tient déjà du référentiel chargé.

### Backend — exposition du cadre normatif à la page méthodologie

**T6 — Enrichir `Co2ReferentialDto` avec un bloc cadre normatif structuré (données objectives uniquement).**
Fichier : `src-tauri/src/application/dto/emission_factor_dto.rs`.
Ajouter un champ `framework: NormativeFrameworkDto` à `Co2ReferentialDto` (ligne 34-38), où `NormativeFrameworkDto` (serde camelCase) porte des **constantes versionnées** non i18n (les libellés humains restent côté i18n front) :

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct NormativeFrameworkDto {
    pub calculation_method: String,   // ex. "distance-based / average-data"
    pub ghg_protocol: String,         // ex. "GHG Protocol Corporate Standard 2004"
    pub factor_database: String,      // ex. "Base Empreinte v23.6 · DEFRA 2024"
    pub gwp_basis: String,            // ex. "PRG 100 ans — GIEC AR6"
    pub referentials: Vec<String>,    // ex. ["BEGES V5", "ESRS E1-6", "ISO 14064-1"]
    pub reporting_year: i32,          // année de reporting (= année courante par défaut)
    pub reference_year: i32,          // année de référence (= factor_year)
}
```

Fichier : `src-tauri/src/application/use_cases/list_co2_referential.rs`.
Construire ce bloc dans `execute()` (à côté de `factor_year` et `radiative_forcing`, ligne 57-61). Les chaînes sont des **constantes Rust** (cf. §5), pas de l'i18n : ce sont des références normatives objectives (noms de standards, versions de bases). `reference_year = factor_year` ; `reporting_year` = année civile courante (à dériver d'un `Clock` injecté, cohérent avec le reste du code qui utilise `Arc<dyn Clock>` ; sinon constante documentée). Ce DTO alimente la page méthodologie ET sert de source unique au front pour bâtir les `rows` de l'en-tête d'export (T8).

### Frontend (React / TS)

**T7 — Card « Cadre normatif » dans la page méthodologie.**
Fichiers : `src/features/methodology/presentation/pages/methodology-page.tsx`, `src/features/methodology/domain/entities/co2-referential.ts`, mapper `src/features/methodology/data/mappers/co2-referential.mapper.ts` (+ son test) et DTO `…/data/dto/co2-referential.dto.ts`.
- Étendre l'entité front `Co2Referential` (et son DTO + mapper) avec `framework` (camelCase, miroir de `NormativeFrameworkDto`).
- Ajouter une `Card` (même structure que la Card Paramètres, après l'intro et avant/après les Formules) rendant un `<dl>` avec : Méthode, GHG Protocol, Base de facteurs, PRG, Référentiels, Année de reporting, Année de référence, Forçage radiatif (réutilise `referential.radiativeForcing`). Tous les **intitulés** via clés `methodology.framework.*` ; les **valeurs** via `referential.framework.*` (sauf RF déjà disponible).

**T8 — Construire les métadonnées d'export côté front et les transmettre.**
Fichiers : `src/features/export/presentation/build-export-labels.ts` (ou un nouveau `build-export-metadata.ts`), l'interface `src/features/export/domain/entities/export.ts`, le hook `use-export.ts`, le repository `tauri-export.repository.ts`, et le dialog `export-dialog.tsx`.
- Ajouter `ExportMetadata` à l'entité TS (camelCase, miroir du DTO backend) + `buildExportMetadata(t, referential)` qui assemble les `rows` traduites à partir de `referential.framework` (déjà chargé via `useCo2Referential`) et de `referential.factorYear` / `radiativeForcing`.
- Le dialog passe `metadata` à `invoke` (nouveau paramètre de la commande). Si le référentiel n'est pas chargé, passer `null` (l'export reste fonctionnel sans la feuille).

**T9 — Clés i18n (FR + EN).**
Fichiers : `src/core/i18n/locales/fr/translation.json`, `…/en/translation.json`.
Ajouter `methodology.framework.*` (titre + 8 intitulés) et `export.sheet.methodology.*` (titre feuille, en-têtes colonnes, et les 8 intitulés du bloc — réutiliser ou pointer vers les mêmes libellés que `methodology.framework.*` pour cohérence). FR et EN **obligatoirement synchronisés** (cf. §8).

## 4. Modèle de données / migrations

**Sans objet.** Ce lot n'ajoute **aucune** colonne ni table. Les références normatives sont des **constantes applicatives** (versionnées dans le code, datées par commit), pas des données utilisateur. Le millésime de facteurs (`factor_year`) et le multiplicateur RF existent déjà en base / dans le référentiel. La prochaine migration libre reste `0012_*.sql` (dernière connue : `0011_add_co2_2026.sql`) ; ce lot **ne la consomme pas** — il la laisse aux lots a/c/d/g qui en ont besoin.

> **R9 (figeage des jours)** : strictement préservé. Aucune valeur `co2_kg` ni `factor_year` n'est touchée. La feuille « Méthodologie » décrit le **millésime par défaut** du profil au moment de l'export ; les trajets exportés conservent leur `factor_year` figé par ligne (feuille Trajets existante). Si un classeur contient des jours de millésimes différents, la feuille Méthodologie l'indique (mention « millésime par défaut ; voir colonne *Année facteur* de la feuille Trajets pour le détail par jour », cf. §5).

## 5. Spécification détaillée

### 5.1 Contenu du bloc « Cadre normatif » / en-tête d'export

Tableau clé → valeur. Les **clés** (intitulés) sont traduites (i18n) ; les **valeurs** sont soit objectives (millésime, RF, pays, années), soit des constantes normatives versionnées dans le code.

| Clé i18n (`methodology.framework.*`) | Intitulé FR | Valeur (exemple millésime 2026) | Source de la valeur |
|---|---|---|---|
| `method` | Méthode de calcul | `distance-based / average-data` | constante (`NormativeFrameworkDto.calculationMethod`) |
| `ghgProtocol` | Norme comptable | `GHG Protocol Corporate Standard (2004)` | constante (`ghgProtocol`) |
| `factorDatabase` | Base de facteurs | `Base Empreinte (ADEME) · DEFRA/DESNZ 2024 · SNCF/SNCB Open Data` | constante (`factorDatabase`) |
| `appReferentialYear` | Millésime référentiel app | `2026` | objectif (`factorYear`) |
| `gwp` | Potentiel de réchauffement (PRG) | `PRG 100 ans — valeurs GIEC AR6` | constante (`gwpBasis`) |
| `radiativeForcing` | Forçage radiatif aérien | `inclus — multiplicateur 1,7` | objectif (`radiativeForcing`, déjà exposé) |
| `referentials` | Référentiels de reporting | `BEGES V5 · ESRS E1-6 (CSRD) · ISO 14064-1` | constante (`referentials`) |
| `reportingYear` | Année de reporting | `2026` | objectif (`reportingYear`, année courante) |
| `referenceYear` | Année de référence | `2026` | objectif (`referenceYear` = `factorYear`) |
| `gridCountry` | Pays grille électrique | `BE` | objectif (`gridCountry`, profil) |
| `unit` | Unité des résultats | `kgCO₂e` (mention : conversion tCO₂e — Lot 3 gap c) | constante |
| `caveat` | Mention | `Export non certifié — données déclaratives à des fins de pré-diagnostic` | constante (i18n) |

Valeurs constantes proposées (à figer dans `list_co2_referential.rs`, modifiables au prochain millésime) :

- `calculation_method = "distance-based / average-data"` (GHG Protocol Scope 3 Technical Guidance, cat. 7).
- `ghg_protocol = "GHG Protocol Corporate Standard (2004)"`.
- `factor_database = "Base Empreinte (ADEME) · DEFRA/DESNZ 2024 · SNCF/SNCB Open Data"` (refléter `0011_add_co2_2026.sql:30-56`).
- `gwp_basis = "PRG 100 ans — GIEC AR6"`.
- `referentials = ["BEGES V5", "ESRS E1-6 (CSRD)", "ISO 14064-1"]`.

### 5.2 Règles de cohérence

- **Une seule source de vérité** : les valeurs objectives (millésime, RF, pays) proviennent toutes du `Co2ReferentialDto` (chargé via `useCo2Referential`). Le front ne recopie pas de chiffres « en dur » : il les lit du référentiel et les place dans `rows` (T8).
- **i18n-neutralité du backend** : l'exporteur ODS ne traduit rien. Toute chaîne lisible (intitulés, titre de feuille, en-têtes colonnes, mention « non certifié ») arrive déjà traduite dans `ExportMetadata.rows` / `sheet_title` / `key_header` / `value_header`. Conforme à la convention `ExportLabels`.
- **Formatage des nombres** : le forçage radiatif est formaté côté front avec `Intl.NumberFormat` de la locale active (la page utilise déjà `nf.format(referential.radiativeForcing)`, ligne 109) avant d'être placé dans `rows`. Le backend reçoit la chaîne finale (« 1,7 » en FR, « 1.7 » en EN).
- **Ordre des feuilles** : Méthodologie (si présente) **en premier**, puis Présences, Tâches, Trajets, Notes (ordre inchangé).
- **Édge case « classeur multi-millésimes »** : le bloc affiche le millésime **par défaut** du profil et ajoute la ligne de renvoi vers la colonne *Année facteur* de la feuille Trajets. Aucune incohérence avec R9 : on documente, on ne recalcule pas.
- **Édge case « référentiel absent »** (`metadata = None` / `framework` non chargé) : l'export s'effectue **sans** la feuille Méthodologie (rétro-compatible) ; la page méthodologie masque la Card si `referential` est nul (même garde que les autres Cards, ligne 106/185).

## 6. Critères d'acceptation

1. **Feuille présente** : quand `metadata` est fourni, le `.ods` exporté contient une feuille en **première position** dont le titre = `sheet_title` reçu, avec deux colonnes (Élément / Valeur).
2. **Contenu objectif exact** : la feuille contient une ligne « Millésime référentiel app » = `2026`, une ligne « Forçage radiatif » contenant `1,7` (FR) ou `1.7` (EN), une ligne « Pays grille électrique » = `BE` (ou la valeur réelle du profil).
3. **Constantes normatives** : la feuille contient les lignes Méthode (`distance-based / average-data`), Norme (`GHG Protocol Corporate Standard (2004)`), PRG (`PRG 100 ans — GIEC AR6`), Référentiels (`BEGES V5 · ESRS E1-6 (CSRD) · ISO 14064-1`).
4. **Rétro-compat export** : quand `metadata` est absent (`None`), l'export produit exactement les mêmes feuilles qu'avant (Présences/Tâches/Trajets/Notes), test `writes_a_readable_multi_sheet_ods` toujours vert.
5. **Page méthodologie** : la Card « Cadre normatif » s'affiche, montre les 8+ intitulés via `methodology.framework.*`, et les valeurs issues de `referential.framework` ; elle est masquée si aucun profil/référentiel.
6. **i18n complète** : toutes les nouvelles clés existent en FR **et** EN ; aucune clé manquante (sinon l'export afficherait une chaîne vide / brute).
7. **Aucune migration** : `git diff` sur `src-tauri/migrations/` est vide pour ce lot.
8. **Conformité réglementaire** : un lecteur de l'export peut, sans connaissance externe, identifier la méthode, la base de facteurs + millésime, le PRG, l'année de reporting/référence et les référentiels visés — exigence BEGES ch. 9 / ESRS E1-6.

## 7. Tests à écrire / mettre à jour

### Rust

- **`ods_writer.rs` (module `tests`)** : nouveau test `methodology_sheet_written_first_when_metadata_present`. Construire un `ExportData` avec `metadata: Some(ExportMetadata { rows: vec![("Méthode".into(), "distance-based / average-data".into()), ("Forçage radiatif".into(), "1,7".into())], sheet_title: "Méthodologie".into(), key_header: "Élément".into(), value_header: "Valeur".into(), factor_year: 2026, radiative_forcing: 1.7, grid_country: "BE".into() })`. Réouvrir via `calamine`, assert que `sheet_names()[0] == "Méthodologie"` et que la cellule (0,0) = en-tête, qu'une cellule de valeur = « distance-based / average-data ».
- **`ods_writer.rs`** : mettre à jour `sample_data()` (ligne 288) avec `metadata: None` et garder `writes_a_readable_multi_sheet_ods` (vérifie la rétro-compat : pas de feuille « Méthodologie » quand `None`).
- **`export_profile_data.rs`** (si test d'intégration présent dans `integration_tests.rs`) : passer `metadata` (`None` ou `Some`) au nouvel argument et vérifier que `ExportSummaryDto` reste inchangé.
- **DTO** : test du `From<ExportMetadataDto> for ExportMetadata` (mapping 1:1 des `rows`).
- **`list_co2_referential.rs`** : test que `execute()` renvoie un `framework` non vide avec `reference_year == factor_year` et `referentials` contenant `"BEGES V5"`.
- Commande : `cargo test` (et `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings`).

### Frontend (vitest)

- **mapper** `co2-referential.mapper.test.ts` : étendre le cas existant pour vérifier que `framework` est correctement mappé du DTO (camelCase) vers l'entité.
- **`build-export-metadata`** (nouveau test) : `buildExportMetadata(t, referential)` produit des `rows` non vides, avec un intitulé traduit et la valeur de RF formatée par locale.
- Commande : `pnpm test`, `pnpm typecheck`, `pnpm lint`.

## 8. Points d'attention & pièges

- **R9 — figeage** : ne **jamais** dériver les valeurs du bloc à partir d'un recalcul. Le bloc décrit la configuration du référentiel par défaut ; le détail par jour reste dans la colonne `factor_year` de la feuille Trajets (figée par ligne). Ajouter la mention de renvoi pour les classeurs multi-millésimes.
- **i18n FR + EN synchronisés** : oublier une clé EN (ou FR) casse silencieusement l'affichage (chaîne vide). Vérifier que `methodology.framework.*` et `export.sheet.methodology.*` existent dans **les deux** fichiers. Diacritiques FR corrects (« Référentiel », « réchauffement », « Année de référence »).
- **Le backend n'i18n jamais** : respecter la convention établie par `ExportLabels` — toute chaîne lisible vient du front déjà traduite. Les seules constantes Rust admises (`list_co2_referential.rs`) sont des **noms propres de standards** (GHG Protocol, BEGES V5, ISO 14064-1, GIEC AR6) qui ne se traduisent pas ; tout intitulé descriptif passe par l'i18n.
- **Couches DDD** : `ExportMetadata` vit dans `domain/services/spreadsheet_exporter.rs` (port), `ExportMetadataDto` dans `application/dto`, le mapping `From` à la frontière, l'écriture dans `infrastructure/export`. Le domaine n'importe rien de tiers (pas de `spreadsheet_ods` hors infra).
- **Rétro-compat des appels** : `metadata` est **optionnel** partout (`Option<…>` Rust, `… | null` TS). Un appelant qui ne le fournit pas garde le comportement actuel — protège les tests et tout consommateur de la commande.
- **Ne pas confondre « scope » périmètre et Scope GHG** : ce lot **n'introduit pas** le Scope GHG (1/2/3) — c'est le gap (a). Le bloc cadre normatif **nomme** les référentiels (BEGES/ESRS/ISO) mais ne classe pas les facteurs par Scope ; rester strictement déclaratif pour ne pas empiéter sur le Lot du gap (a).
- **Mouvance réglementaire** : les constantes (versions de base, PRG, libellés de référentiels) sont datées et changeront au prochain millésime. Les centraliser dans `list_co2_referential.rs` (et i18n) permet une mise à jour en un point lors du passage à un futur référentiel (ex. millésime 2027, GIEC AR7), cohérent avec la stratégie « nouveau millésime, jamais d'édition en place » (R9).
- **Mention « non certifié »** : indispensable pour ne pas laisser croire à un bilan opposable. À inclure systématiquement dans la feuille et dans la Card.

## 9. Références réglementaires

- **GHG Protocol — Corporate Accounting and Reporting Standard (rév. 2004)** : exige la divulgation de la méthode et des facteurs. https://ghgprotocol.org/corporate-standard
- **GHG Protocol — Corporate Value Chain (Scope 3) Standard** et **Technical Guidance for Calculating Scope 3 Emissions** (catégorie 7 « Employee commuting », méthodes *distance-based* / *average-data*). https://ghgprotocol.org/scope-3-technical-calculation-guidance
- **BEGES (Bilan d'émissions de gaz à effet de serre), méthode réglementaire V5** — ADEME / Ministère de la Transition écologique (art. L229-25 du Code de l'environnement) : le bilan doit préciser la méthode et les facteurs ; restitution en tCO₂e. https://bilans-ges.ademe.fr/
- **Base Empreinte (ADEME)** — base officielle des facteurs d'émission (ex-Base Carbone). https://base-empreinte.ademe.fr/
- **CSRD / ESRS E1 « Changement climatique »**, dispositions E1-6 (émissions brutes Scope 1/2/3 + total) — Règlement délégué (UE) 2023/2772 : documentation de la méthodologie, des facteurs et du PRG. https://eur-lex.europa.eu/legal-content/FR/TXT/?uri=OJ:L_202302772
- **GIEC AR6 (Sixth Assessment Report, WG1, 2021)** — valeurs de PRG (GWP) sur 100 ans (GWP-100). https://www.ipcc.ch/report/ar6/wg1/
- **DEFRA / DESNZ — UK Government GHG Conversion Factors 2024** (facteurs aériens avec forçage radiatif, RF abaissé de 1,9 à 1,7 en juin 2023). https://www.gov.uk/government/collections/government-conversion-factors-for-company-reporting
- **ISO 14064-1:2018** — Spécification et lignes directrices pour la quantification et la déclaration des émissions de GES au niveau de l'organisme. https://www.iso.org/standard/66453.html
- **Belgique (gaps connexes g)** — facteursdemissionco2.be (référence fédérale / PDE bruxellois) ; cité ici pour cohérence du bloc « Base de facteurs » si des facteurs BE sont ajoutés ultérieurement (Lot gap g). https://www.facteursdemissionco2.be/
