# Plan d'implémentation — Conformité réglementaire de l'empreinte carbone domicile-travail

> **Objet.** Ce dossier découpe en **lots indépendants et exécutables** la mise en conformité de
> BasicPresence pour l'usage entreprise de son calcul d'empreinte carbone des déplacements
> domicile-travail, vis-à-vis du **BEGES réglementaire V5** (France), de la **CSRD / ESRS E1-6**
> (UE), du **GHG Protocol** (Scope 3 catégorie 7) et du **cadre belge** (diagnostic fédéral
> mobilité, PDE bruxellois).
>
> Chaque lot est **auto-suffisant** : une instance Claude (ou un développeur) qui ne lit que le
> fichier du lot doit pouvoir l'exécuter. Ce README porte uniquement ce qui est **transverse**
> (ordonnancement, conventions partagées, numérotation des migrations, critères de re-priorisation).
>
> **Dernière mise à jour :** 2026-06-24.

---

## 1. Origine du plan

Ce plan découle d'une *deep research* d'alignement réglementaire (analyse des écarts entre la
donnée physique déjà calculée par BasicPresence et la **couche réglementaire** manquante :
mapping Scope/poste, consolidation entreprise, unité tCO2e, export auditable). Le constat
fondateur : **BasicPresence calcule déjà la bonne donnée physique** (kgCO2e par trajet/mode/jour
avec facteurs versionnés), mais il lui manque la **structuration réglementaire**. Ce sont des
écarts de *structuration de données*, pas de *calcul* — donc relativement peu coûteux à combler.

Le moteur de calcul et les facteurs sont documentés dans
[`../calcul-impact-co2.md`](../calcul-impact-co2.md) (formules, règles R1–R9, sources vérifiées,
écarts §7–§8). **À lire avant tout lot touchant au référentiel** (lots 1, 7, 8).

---

## 2. Les 9 lots (un gap réglementaire = un lot)

| Lot | Gap | Titre | Priorité | Effort | Migration ? |
|---|---|---|---|---|---|
| [1](./lot-1-mapping-scope-poste.md) | a | Mapping Scope GHG / poste BEGES / catégorie ESRS | **P1** | moyen | **oui — `0012`** |
| [2](./lot-2-unite-tco2e.md) | c | Unité tCO2e en parallèle du kgCO2e | **P1** | faible | non (dérivé) |
| [3](./lot-3-export-auditable.md) | b | Export auditable (traçabilité source → résultat) | **P1** | faible-moyen | non |
| [4](./lot-4-consolidation-entreprise.md) | d | Consolidation individu → entreprise (par poste/site/entité) | **P1** | moyen | **oui — `0013`** |
| [5](./lot-5-teletravail.md) | e | Modélisation du télétravail (poste 3.3 + WFH incrémental) | P2 | moyen | **oui — `0014`** |
| [6](./lot-6-reference-cadre-normatif.md) | f | Référence du cadre normatif (en-tête export + méthodologie) | P2 | faible | non |
| [7](./lot-7-facteurs-belges.md) | g | Facteurs d'émission belges (facteursdemissionco2.be) | P2 | faible-moyen | **oui — `0015`** |
| [8](./lot-8-scope2-location-market.md) | h | Scope 2 location-based / market-based (énergie bâtiment) | P3 | moyen | **oui — `0016`** |
| [9](./lot-9-documentation-incertitude.md) | i | Documentation d'audit & incertitude | P3 | moyen | **oui — `0017`** |

> La colonne « Migration » indique le **numéro réservé** par ce plan (voir
> [§5 Numérotation des migrations](#numérotation-des-migrations)). Le numéro réel à
> l'exécution se calcule toujours dynamiquement (`max existant + 1`).

---

## 3. Dépendances & ordre d'exécution recommandé

```
                ┌─────────────────────────────┐
   Lot 1 (a) ───┤ FONDATION : classification   │
  mapping       │ Scope/poste + renommage       │
   scope/poste  │ scope → lifecycle_boundary    │
                └──┬───────┬───────┬───────┬────┘
                   │       │       │       │
        ┌──────────┘       │       │       └────────────┐
        ▼                  ▼       ▼                     ▼
   Lot 3 (b)          Lot 4 (d)  Lot 5 (e)          Lot 8 (h)
   export auditable   consolid.  télétravail        Scope 2 loc/market
        ▲                  ▲                              ▲
        │                  │                              │
   Lot 2 (c) ─────────────┘                          (énergie bâtiment)
   tCO2e (dérivé, sans dépendance)
        ▲
   Lot 6 (f) ── en-tête normatif (alimente l'export du lot 3)

   Lot 7 (g) ── facteurs belges : ~indépendant (suit le pattern de millésime 0011)
   Lot 9 (i) ── doc d'audit/incertitude : dépend de Lot 1 + Lot 3
```

**Sprint 1 (P1) — débloque l'essentiel.** Lots **1 → 2 → 3 → 4**.
Le Lot 1 est strictement préalable (il renomme `scope` et crée les colonnes de classification que
3, 4, 8 lisent). Le Lot 2 (tCO2e dérivé) est indépendant mais consommé par 3 et 4.
**Benchmark de réussite du sprint :** un OTI/CAC peut retracer un résultat tCO2e jusqu'à sa donnée
d'activité et son facteur **sans pièce externe**.

**Sprint 2 (P2).** Lots **5, 6, 7**. Modélisation télétravail, en-tête normatif, facteurs belges.
Débloque les clients belges et fiabilise la catégorie 7.

**Sprint 3 (P3).** Lots **8, 9**. Double facteur Scope 2 (location/market-based), journal
d'incertitude/échantillonnage. Conformité fine ESRS E1-6 et assurance limitée.

> **Parallélisation possible.** À l'intérieur d'un sprint, les lots sans dépendance mutuelle
> peuvent être menés par des instances distinctes — **à condition de respecter la numérotation des
> migrations** (§5) et de livrer le Lot 1 **avant** 3/4/5/8. Les lots 3 et 6 dégradent
> gracieusement si le Lot 1 n'est pas encore mergé (classification `None`/vide), voir leur §4.

---

## 4. Conventions partagées (à respecter par tous les lots)

- **R9 — figeage des jours (NON NÉGOCIABLE).** Un jour déjà encodé conserve son `co2_kg` figé
  (avec son `factor_year`) dans `presence_trip`. **Aucun lot ne réécrit `co2_kg` ni `factor_year`.**
  Toute évolution du référentiel de facteurs passe par un **nouveau millésime** (cf.
  `0011_add_co2_2026.sql`), jamais par un `UPDATE` rétroactif du seed livré. Les métadonnées de
  classification (lot 1) sont résolubles a posteriori via `(mode_id, factor_year)` : reproductibles,
  donc valorisables sur les millésimes existants sans violer R9.
- **Renommage `scope → lifecycle_boundary` (propriété du Lot 1).** Le champ `emission_factor.scope`
  désigne le **périmètre cycle de vie** (well-to-wheel), **pas** le Scope GHG Protocol. Le Lot 1 le
  renomme `lifecycle_boundary` et introduit `ghg_scope`. **Tout lot postérieur** au Lot 1 doit
  utiliser ces noms ; tout lot exécuté **avant** le Lot 1 doit éviter de figer l'ancien nom `scope`.
- **tCO2e dérivé (Lot 2).** La tonne est `co2_kg × 0,001`, **calculée à la lecture/affichage/export**,
  jamais stockée (pas de colonne `co2_t`, pas de backfill). Le kgCO2e reste l'unique source de vérité.
- **Architecture DDD (back Rust).** `src-tauri/src/{domain,application,infrastructure,presentation}`.
  Enums et règles métier dans `domain` (zéro dépendance tierce) ; conversions enum→String et DTO à la
  frontière `application` ; SQL dans `infrastructure`. Le **`SELECT` positionnel** de
  `emission_factor_repository.rs` (lecture par `row.get(N)`) est fragile : mettre à jour `SELECT_COLUMNS`
  **et** les index `row.get(N)` ensemble à chaque ajout de colonne.
- **i18n FR **et** EN.** Ne jamais ajouter une clé dans une seule langue (rendu/export vide).
  Locales sous `src/core/i18n/locales/{fr,en}/translation.json`.
- **CI — un lot n'est « terminé » que si tout passe :**
  ```
  cd src-tauri && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test
  pnpm test && pnpm typecheck && pnpm lint
  ```
- **Commits sémantiques** (cf. historique : `feat(Co2): …`, `fix(Import): …`). Un lot = idéalement
  une branche et un PR cohérent. Voir [`../code-review-checklist.md`](../code-review-checklist.md).

---

## 5. Numérotation des migrations

**Le problème.** Plusieurs lots créent une migration vault. Comme ils ont été planifiés
indépendamment, plusieurs revendiquent naïvement le même numéro `0012`. La dernière migration
réelle au moment de la rédaction est **`0011_add_co2_2026.sql`**.

**La règle impérative (source de vérité).** Le numéro d'une migration n'est **jamais** codé en dur
de façon fiable dans un document de lot. À l'exécution :

```bash
ls src-tauri/migrations/vault/ | sort | tail -1   # → dernière migration existante
```

et prendre **le numéro libre immédiatement supérieur**, puis aligner le champ `version:` ajouté dans
`src-tauri/src/infrastructure/persistence/migrations.rs` (`VAULT_MIGRATIONS`). Les références `00XX`
écrites dans chaque lot **supposent un dépôt à `0011`**.

**Table de réservation (plan d'allocation, si les lots sont exécutés dans l'ordre recommandé).**

| Lot | Migration réservée | Le document du lot écrit aujourd'hui | Action à l'exécution |
|---|---|---|---|
| 1 | `0012_add_ghg_mapping.sql` | `0012` | OK si exécuté en premier ; sinon `max+1` |
| 4 | `0013_add_consolidation.sql` | `0012` | **renuméroter** vers le slot libre |
| 5 | `0014_add_wfh_setting.sql` | `0012` | **renuméroter** vers le slot libre |
| 7 | `0015_add_be_factors.sql` | `0012` | **renuméroter** vers le slot libre |
| 8 | `0016_add_scope2_building.sql` | `0012` | **renuméroter** vers le slot libre |
| 9 | `0017_add_audit_log.sql` | `0013` | **renuméroter** vers le slot libre |
| 2 | *(aucune par défaut)* | — | tCO2e dérivé, pas de migration |

> Cette table suppose l'**ordre d'exécution recommandé** (§3). Si l'ordre réel diffère, seul compte
> le `max+1` calculé dynamiquement. Renuméroter implique : renommer le fichier `.sql`, ajuster son
> `version:` dans `migrations.rs`, et corriger les références `00XX` à l'intérieur du document du lot.

---

## 6. Comment exécuter un lot (checklist pour l'instance)

1. Lire **ce README** (conventions §4, numérotation §5) puis le fichier du lot **en entier**.
2. Vérifier les **dépendances** (§3) : les lots préalables sont-ils mergés ? Sinon, appliquer la
   dégradation gracieuse documentée dans le lot, ou attendre.
3. Si le lot crée une migration : **calculer le numéro libre** (`ls … | tail -1` + 1), renommer en
   conséquence.
4. Implémenter les tâches `T1…Tn` dans l'ordre, en citant les vrais chemins `fichier:ligne`.
5. Écrire/mettre à jour les **tests** (§7 du lot) ; faire passer la **CI** (§4 ci-dessus).
6. Vérifier les **critères d'acceptation** (§6 du lot) un par un.
7. Mettre à jour le **suivi** (§8 ci-dessous) et la doc impactée (`calcul-impact-co2.md` si référentiel).

---

## 7. Caveats réglementaires (cadre mouvant — Omnibus 2025-2026)

- **Seuils CSRD relevés** (Omnibus) à **1 000 salariés + 450 M€** ; le BEGES reste à **500 salariés**.
  Certaines entreprises **sortent de la CSRD mais restent obligées au BEGES** → le BEGES (lots 1-4)
  est la cible la plus stable.
- **Datapoints ESRS en réduction** (~60-70 %) pour les exercices ouverts en 2027 : `esrs_datapoint`
  est volontairement laissé en **texte libre** (Lot 1) pour absorber cette mouvance.
- **Sanctions BEGES** portées à **50 000 € (100 000 € en récidive)** (loi Industrie Verte 2023) +
  conditionnalité des aides ADEME/Bpifrance depuis le 1ᵉʳ juin 2024.

**Seuils de re-priorisation (remonter des lots P3→P1 selon le client cible) :**
- Client **assujetti CSRD** (> 1 000 salariés **et** > 450 M€) → remonter **Lot 8** (Scope 2
  location/market) et **Lot 9** (incertitude/audit) en **P1** (ils sont vérifiés par l'OTI).
- **PME belge > 100 travailleurs** → remonter **Lot 7** (facteurs belges) en **P1**.
- **Entreprise FR 50-500 salariés** cherchant des aides ADEME/Bpifrance → **Lots 2 + 4** (tCO2e +
  consolidation) suffisent pour un bilan simplifié.

---

## 8. Suivi d'avancement

| Lot | État | Branche / PR | Notes |
|---|---|---|---|
| 1 — Mapping Scope/poste | ☐ À faire | | Fondation — à livrer en premier |
| 2 — Unité tCO2e | ☐ À faire | | Indépendant |
| 3 — Export auditable | ☐ À faire | | Dépend de 1, 2 |
| 4 — Consolidation entreprise | ☐ À faire | | Dépend de 1, 2 |
| 5 — Télétravail | ☐ À faire | | Dépend de 1 |
| 6 — Référence cadre normatif | ☐ À faire | | Alimente 3 |
| 7 — Facteurs belges | ☐ À faire | | ~Indépendant |
| 8 — Scope 2 location/market | ☐ À faire | | Dépend de 1 |
| 9 — Documentation/incertitude | ☐ À faire | | Dépend de 1, 3 |

---

## 9. Références transverses

- [`../calcul-impact-co2.md`](../calcul-impact-co2.md) — moteur, formules, règles R1–R9, sources des facteurs.
- [`../code-review-checklist.md`](../code-review-checklist.md) — checklist de revue.
- **BEGES V5** : art. L229-25 Code de l'environnement ; décret n°2022-982 (Scope 3 obligatoire au 01/01/2023) ; plateforme https://bilans-ges.ademe.fr ; Base Empreinte® https://base-empreinte.ademe.fr
- **GHG Protocol** : Corporate Standard (2004) + Scope 3 Standard, catégorie 7 « Employee commuting » — https://ghgprotocol.org
- **CSRD / ESRS E1** : Règlement délégué (UE) 2023/2772, datapoint **E1-6** — https://eur-lex.europa.eu/eli/reg_del/2023/2772/oj
- **Belgique** : SPF Mobilité (diagnostic fédéral) https://mobilit.belgium.be ; facteursdemissionco2.be (CO2logic + EnergieID) https://www.facteursdemissionco2.be
