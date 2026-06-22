# Calcul de l'impact CO₂ — formules & sources

> **Objet.** Ce document recense **toutes les formules** utilisées par BasicPresence pour
> calculer l'empreinte carbone (« impact ») d'une journée de présence, ainsi que **la source
> de chaque facteur d'émission** du référentiel. Les sources présentes dans le code (colonne
> `source` du référentiel) ont été **complétées et vérifiées par recherche web** par rapport
> aux publications de référence (ADEME, DEFRA/DESNZ, SNCF, SNCB, Ember/OWID, EEA…), puisque
> c'est sur cette base que les facteurs ont été établis.
>
> **À jour au :** 2026-06-22. **Millésimes du référentiel :** `2025` (historique, gelé pour la
> reproductibilité) et **`2026` (défaut, corrigé)** — voir §6 bis. Le millésime utilisé par un
> profil est `factor_year` (défaut **2026**). Les jours déjà encodés conservent leur empreinte
> figée quel que soit le millésime courant (règle R9).

---

## 1. Où vit le calcul dans le code

| Élément | Fichier |
|---|---|
| Moteur de calcul (arithmétique pure) | `src-tauri/src/domain/services/co2_calculator.rs` |
| Référentiel des facteurs (seed SQL) | `src-tauri/migrations/vault/0004_add_co2.sql` |
| Entités facteur / unité / catégorie | `src-tauri/src/domain/entities/emission_factor.rs` |
| Paramètres CO₂ du profil | `src-tauri/src/domain/entities/co2_settings.rs` |
| Application au jour de présence | `src-tauri/src/application/use_cases/set_presence.rs` |
| Chargement du référentiel par année | `src-tauri/src/application/use_cases/factor_maps.rs` |

Le **frontend ne contient aucune formule** : il se contente d'afficher la valeur `co2_kg`
calculée et figée par le backend (`src/features/commute/presentation/commute-format.ts`).

**Principe d'architecture.** Les facteurs ne sont **jamais codés en dur** dans la logique :
ils vivent dans une table versionnée par année (`emission_factor`). À l'encodage d'un jour,
le résultat `co2_kg` est **figé** dans `presence_trip` avec l'année de référentiel utilisée
(`factor_year`), de sorte qu'un jour passé reste **reproductible** même si le référentiel
change ensuite (règle R9).

---

## 2. Vue d'ensemble du pipeline

```
Jour de présence (bureau / télétravail / congé / férié)
        │
        ├─ congé ou férié ──────────────────────────────► total = 0  (R7)
        │
        └─ bureau / télétravail
                │
                ├─ pour chaque trajet (segment) : co2_trajet  (cf. §3)
                │        Σ = total_trajets
                │
                ├─ énergie du bâtiment (optionnelle)          (cf. §4.4)
                │
                └─ total_jour = max(total_trajets + énergie_bâtiment, 0)   (R8)
```

---

## 3. Formule par trajet (segment)

Réf. `co2_calculator.rs::compute_trip`. Pour **un segment** (une étape d'un trajet) :

```
co2_trajet = distance_km × facteur_effectif × (aller_retour ? 2 : 1) ÷ diviseur_occupants

avec :  co2_trajet ≥ 0          (borné, R8)
```

Détail des étapes, dans l'ordre exact d'application :

1. **Résolution du facteur** (`mode_id` → ligne du référentiel).
   Si le `mode_id` est inconnu → repli sur `car_average` (0,2311) et le trajet est marqué
   `is_estimated = true` (règle R6). Si aucun repli n'existe → 0 (mais signalé).

2. **Facteur effectif** `facteur_effectif`, dans cet ordre de priorité (mutuellement exclusifs) :
   - **Variante réseau électrique** : s'il existe une variante `(mode_id, pays_de_recharge)`
     (uniquement `car_ev` aujourd'hui), elle **remplace** la valeur de base (règle R4, cf. §5).
   - **Sinon, forçage radiatif aérien** : si la catégorie est `air` **et** que le forçage
     radiatif est désactivé, on **divise** la valeur par le multiplicateur **dépendant de
     l'année** (`radiative_forcing_factor` : **1,9** pour le millésime ≤ 2025, **1,7** pour
     ≥ 2026, car DEFRA l'a abaissé en juin 2023) afin de revenir au CO₂ seul (règle R5 — on
     divise, jamais on ne multiplie, pour ne pas double-compter, car les facteurs aériens sont
     stockés **forçage radiatif inclus** avec le multiplicateur de leur millésime).
   - **Sinon** : valeur de base du référentiel.

3. **Distance × aller-retour** : `distance_km` est toujours **un aller simple** ; le drapeau
   aller-retour (porté par le trajet, appliqué à chaque segment) double la distance (règle R1).

4. **Division par les occupants** (covoiturage, règle R2) : **uniquement** pour les facteurs
   exprimés **par véhicule** (`unit = kgCO2e/veh.km`). On divise par `max(occupants, 1)`.
   Les facteurs `passager.km` et `km` sont **déjà par personne** → pas de division.

5. **Bornage** : `co2_trajet = max(co2_trajet, 0)` (R8, jamais négatif).

### Les 4 unités et leur effet

Réf. `emission_factor.rs::EmissionUnit`.

| Unité (en base) | Signification | Divisé par occupants ? |
|---|---|---|
| `kgCO2e/veh.km` | par **véhicule**.km (réparti en covoiturage) | **Oui** |
| `kgCO2e/passenger.km` | par **passager**.km (déjà par personne) | Non |
| `kgCO2e/km` | par km (absolu, mobilité active/douce) | Non |
| `kgCO2e/day` | par **jour** (énergie bâtiment) | Non (cf. §4.4) |

---

## 4. Total de la journée & paramètres

Réf. `co2_calculator.rs::compute_day` et `co2_settings.rs`.

### 4.1 Total du jour

```
total_jour = max( Σ co2_trajet  +  énergie_bâtiment , 0 )
```

- **Congé / férié** → `total_jour = 0` même si des trajets traînent (R7).
- **Bureau / télétravail** → somme des trajets + énergie bâtiment éventuelle.

### 4.2 Paramètres du profil (`Co2Settings`)

| Paramètre | Défaut | Effet |
|---|---|---|
| `grid_country` | `"BE"` | Pays de recharge → sélectionne la variante réseau de `car_ev` (§5) |
| `default_car_occupancy` | `1` | Occupants par défaut si un trajet ne les précise pas |
| `include_radiative_forcing` | `true` | Si `false`, divise les facteurs aériens par le multiplicateur de l'année (1,9 ≤ 2025 / 1,7 ≥ 2026) (§3, étape 2) |
| `count_building_energy` | `false` | Si `true`, ajoute l'énergie bâtiment (§4.4) |
| `working_days_per_year` | `220` | Pour les projections annuelles (Phase 2) |
| `factor_year` | **`2026`** | Millésime du référentiel à résoudre |

### 4.3 Covoiturage (rappel)

Pour une voiture (`veh.km`) : `co2 = distance × facteur × AR ÷ occupants`. Exemple : 3 personnes
dans une voiture essence divisent l'empreinte du trajet par 3.

### 4.4 Énergie du bâtiment (optionnelle)

Si `count_building_energy = true`, on **ajoute au total du jour** un forfait journalier
selon le type de présence :

- **Bureau** → `office_day` = **3,5 kgCO₂e/jour**
- **Télétravail** → `home_day` = **2,7 kgCO₂e/jour**
- Congé / férié → rien.

Ce forfait représente l'énergie du lieu de travail (chauffage + électricité + éclairage),
**hors trajet** (le trajet est compté séparément). Désactivé par défaut.

---

## 5. Variantes réseau électrique (`car_ev`)

Réf. seed `0004_add_co2.sql` + `co2_calculator.rs`. La voiture électrique a une empreinte qui
dépend de **l'intensité carbone du réseau de recharge**. La variante du pays sélectionné
**remplace** la valeur de base.

| Pays (`grid_country`) | `car_ev` 2025 (kgCO₂e/veh.km) | `car_ev` **2026** |
|---|---|---|
| `FR` (France) | 0,09 | 0,09 |
| `BE` (Belgique) — *défaut* | 0,1393 | **0,11** |
| `DE` (Allemagne) | 0,18 | 0,18 |
| `EU` (moyenne UE) | 0,13 | 0,13 |

> La valeur **de base** `car_ev` est alignée sur la variante **BE** (défaut applicatif belge) :
> 0,1393 en 2025, **0,11 en 2026**. La correction 2026 reflète le mix réseau belge réel
> (~135 gCO₂/kWh), l'ancien 0,1393 surestimant la Belgique d'environ 20-25 %.

---

## 6. Référentiel des facteurs d'émission (millésime 2025)

Valeurs telles que seedées dans `0004_add_co2.sql`. La colonne **Source (code)** est la chaîne
inscrite en base ; la **vérification détaillée** (source réelle, millésime, écarts) est en §7.

### 6.1 Voiture (`veh.km`, divisé par occupants)

| `mode_id` | Libellé | Valeur | Périmètre (`scope`) | Source (code) |
|---|---|---|---|---|
| `car_petrol` | Voiture — essence | 0,2388 | usage+amont+fabrication | ADEME 2024-2025 |
| `car_diesel` | Voiture — diesel | 0,2275 | usage+amont+fabrication | ADEME 2024-2025 |
| `car_average` | Voiture — motorisation moyenne | 0,2311 | usage+amont+fabrication | ADEME 2024-2025 |
| `car_phev` | Voiture — hybride rechargeable | 0,1021 | usage+amont+fabrication | ADEME 2024-2025 |
| `car_ev` | Voiture — électrique | 0,1393 | usage+fabrication | ADEME 2024-2025 |
| `taxi` | Taxi / VTC | 0,2311 | usage+amont+fabrication | ADEME 2024-2025 |

### 6.2 Mobilité active & micro-mobilité (`km`, par personne)

| `mode_id` | Libellé | Valeur | Périmètre | Source (code) |
|---|---|---|---|---|
| `walk` | À pied | 0,0 | convention | ADEME 2024-2025 |
| `bike` | Vélo musculaire | 0,0 | convention | ADEME 2024-2025 |
| `ebike` | Vélo électrique | 0,01095 | usage+fabrication | ADEME 2024-2025 |
| `escooter` | Trottinette électrique | 0,0249 | usage+fabrication | ADEME 2024-2025 |
| `scooter_elec` | Scooter électrique | 0,0249 | usage+fabrication | ADEME 2024-2025 |

### 6.3 Transports en commun (`passenger.km`, par personne)

| `mode_id` | Libellé | Valeur | Périmètre | Source (code) |
|---|---|---|---|---|
| `public_transport` | Transport en commun (mix) | 0,04525 | usage | ADEME 2024-2025 |
| `bus` | Bus urbain | 0,1515 | usage | ADEME 2024-2025 |
| `coach` | Autocar longue distance | 0,0295 | usage | ADEME 2024-2025 |
| `metro_tram` | Métro / tram | 0,005 | usage | ADEME 2024-2025 |

### 6.4 Train (`passenger.km`, par personne)

| `mode_id` | Libellé | Valeur | Périmètre | Source (code) |
|---|---|---|---|---|
| `train_sncb` | Train Intercity (SNCB) | 0,021 | usage+fabrication | SNCB |
| `train_ter` | Train régional (TER) | 0,0299 | usage+fabrication | SNCF Open Data |
| `train_hs_fr` | TGV (France) | 0,00343 | usage+fabrication | SNCF Open Data |
| `train_eurostar` | Eurostar | 0,0055 | usage+fabrication | opérateur |
| `train_thalys` | Thalys | 0,0084 | usage+fabrication | opérateur |

### 6.5 Avion (`passenger.km`, **forçage radiatif inclus**)

| `mode_id` | Libellé | Valeur | Périmètre | Source (code) |
|---|---|---|---|---|
| `plane_domestic` | Avion — intérieur / très court | 0,2582 | usage+forçage radiatif | DEFRA 2024-2025 |
| `plane_short` | Avion — court-courrier | 0,2582 | usage+forçage radiatif | DEFRA 2024-2025 |
| `plane_medium` | Avion — moyen-courrier | 0,1872 | usage+forçage radiatif | DEFRA 2024-2025 |
| `plane_long` | Avion — long-courrier | 0,1517 | usage+forçage radiatif | DEFRA 2024-2025 |

> **Forçage radiatif** : multiplicateur **1,9** intégré aux facteurs aériens
> (`RADIATIVE_FORCING_FACTOR`, `co2_calculator.rs:11`). Désactivable par l'utilisateur → division.

### 6.6 Énergie du bâtiment (`day`)

| `mode_id` | Libellé | Valeur | Périmètre | Source (code) |
|---|---|---|---|---|
| `office_day` | Jour au bureau (énergie bâtiment) | 3,5 | usage | DEFRA 2024-2025 |
| `home_day` | Jour télétravail (énergie domicile) | 2,7 | usage | DEFRA 2024-2025 |

---

## 6 bis. Référentiel 2026 (par défaut) — corrections

Le millésime **2026** (`src-tauri/migrations/vault/0011_add_co2_2026.sql`) est désormais le défaut
(`factor_year = 2026`). Il corrige les écarts du §8 **sans toucher au millésime 2025** (gelé pour
la reproductibilité des jours déjà encodés, R9). **Seuls les facteurs ci-dessous changent** ; tous
les autres sont **identiques à 2025** (voitures thermiques, marche/vélo/VAE/trottinette, autocar,
SNCB, télétravail…).

| `mode_id` | 2025 | **2026** | Raison / source |
|---|---|---|---|
| `plane_domestic` | 0,2582 | **0,2293** | DEFRA/DESNZ 2024 (forçage radiatif **1,7**) |
| `plane_short` | 0,2582 | **0,1258** | DEFRA 2024 — court-courrier (n'était plus égal au domestique) |
| `plane_medium` | 0,1872 | **0,121** | Interpolation court/long (pas de bande DEFRA dédiée) |
| `plane_long` | 0,1517 | **0,117** | DEFRA 2024 — long-courrier |
| **forçage radiatif** | 1,9 | **1,7** | Valeur DEFRA depuis juin 2023 — résolue par année dans le code |
| `bus` | 0,1515 | **0,113** | Bus urbain ADEME représentatif (usage) |
| `metro_tram` | 0,005 | **0,0044** | ADEME *Impact CO₂* (métro) |
| `train_ter` | 0,0299 | **0,0238** | SNCF Open Data — périmètre complet 2024 |
| `train_hs_fr` | 0,00343 | **0,0035** | SNCF Open Data — périmètre complet 2024 (TGV) |
| `train_eurostar` | 0,0055 | **0,006** | Facteur officiel DEFRA cité par Eurostar |
| `train_thalys` | 0,0084 | **0,0069** | Estimation SNCF 2023 / Eurostar Group |
| `car_ev` (base + BE) | 0,1393 | **0,11** | Mix réseau belge réel (~135 gCO₂/kWh) |

**Sources clarifiées (valeur inchangée, libellé `source` corrigé) :**
- `office_day` (3,5) → **CIBSE TM46 / Circular Ecology (Univ. Exeter)** au lieu de « DEFRA ».
- `public_transport` (0,04525) → **moyenne pondérée interne** (pas un facteur ADEME publié).
- `scooter_elec` (0,0249) → **aligné sur la trottinette** (pas de facteur ADEME scooter électrique dédié).

> **Mécanique du forçage radiatif par année.** Comme les facteurs aériens sont stockés
> « forçage inclus », le multiplicateur à retrancher dépend du millésime des facteurs. La fonction
> `radiative_forcing_factor(factor_year)` (`co2_calculator.rs`) renvoie **1,9 pour ≤ 2025** et
> **1,7 pour ≥ 2026** : un jour 2025 ré-édité reste divisé par 1,9, un jour 2026 par 1,7.

---

## 7. Sources vérifiées, par famille

Méthodologie de vérification : pour chaque famille, recherche de la **source officielle**, de
sa **méthodologie/périmètre**, puis **re-contrôle indépendant** (sceptique) de chaque valeur
par rapport à une publication. Verdicts : ✅ concorde · ≈ plausible/ordre de grandeur · ⚠️ écart
· ❔ non sourçable directement.

### 7.1 Voiture — ADEME (Base Empreinte® / Base Carbone® + Impact CO₂)

**Source officielle :** ADEME — Base Empreinte® (ex-Base Carbone®) et outil *Impact CO₂*.
**Méthodologie :** facteur = **usage « du puits à la roue » (well-to-wheel)** + **fabrication
du véhicule** amortie au km (~40 gCO₂e/km pour le thermique ; 75-82 % de l'empreinte d'un VE).
Reconstruction : essence ≈ 0,200 (usage) + 0,040 (fab) ≈ 0,24 ; diesel ≈ 0,19 + 0,04 ≈ 0,23.

| `mode_id` | Code | Verdict | Note |
|---|---|---|---|
| `car_petrol` | 0,2388 | ≈ | Usage WtW (~0,20) + fabrication (~0,04). Cohérent avec un **millésime ADEME antérieur**. |
| `car_diesel` | 0,2275 | ≈ | Usage WtW diesel (~0,19) + fab. Diesel < essence : attendu. |
| `car_average` | 0,2311 | ≈ | Pile entre essence et diesel ; cohérent avec une moyenne de parc. |
| `car_phev` | 0,1021 | ≈ (faible) | **Hypothèse optimiste** d'usage majoritairement électrique. La fourchette ADEME « hybride » citée est 0,147-0,232 ; les PHEV en usage réel émettent bien plus que les tests. À documenter. |
| `car_ev` | 0,1393 | ⚠️ | = variante **BE** (pas la France, qui est à 0,09). Cf. §7.7. |
| `taxi` | 0,2311 | ❔ | Pas de facteur « taxi » ADEME dédié : **proxy = voiture moyenne** (choix de modélisation). |

⚠️ **Millésime.** L'outil ADEME *Impact CO₂* **actuel** (modélisation 2025) affiche des valeurs
**nettement plus basses** (thermique moyen ≈ 0,142 ; VE FR ≈ 0,067) suite à une révision
méthodologique 2025. Les valeurs seedées correspondent donc à un **millésime antérieur** : le
libellé « ADEME 2024-2025 » est optimiste sur l'année réelle, même si l'ordre de grandeur reste correct.

**Liens :**
- ADEME — Base Empreinte® : https://base-empreinte.ademe.fr/
- ADEME — Doc Base Carbone, transport routier : https://bilans-ges.ademe.fr/documentation/UPLOAD_DOC_FR/routier.htm
- ADEME *Impact CO₂* — voiture thermique : https://impactco2.fr/outils/transport/voiturethermique
- ADEME *Impact CO₂* — voiture électrique : https://impactco2.fr/outils/transport/voitureelectrique
- Étude ADEME « Facteurs d'émissions des modes de transport routier » : https://data.ademe.fr/applications/etude-facteurs-d'emissions-des-differents-modes-de-transport-routier-liste-et-fiches

### 7.2 Mobilité active — ADEME (Impact CO₂)

**Source officielle :** ADEME — *Impact CO₂* (alimenté par la Base Empreinte). **Périmètre :**
cycle de vie (fabrication + usage). Pour les modes électriques légers, la **fabrication domine**.

| `mode_id` | Code | Verdict | Note |
|---|---|---|---|
| `walk` | 0,0 | ✅ | Convention zéro : ADEME affiche la marche à 0 g/km. |
| `bike` | 0,0 | ✅ | Vélo mécanique ≈ 0,17 g/km (fabrication), usage nul → arrondi à 0 dans le comparateur ADEME. |
| `ebike` | 0,01095 | ✅ **exact** | ADEME : **10,95 g/km** = 8,72 g fab (80 %) + 2,23 g usage (20 %), affiché « 11 g ». |
| `escooter` | 0,0249 | ✅ **exact** | ADEME : **24,9 g/km** = 22,9 g fab (92 %) + 2 g usage (8 %). |
| `scooter_elec` | 0,0249 | ⚠️ | **Recopie de la trottinette.** ADEME n'a pas de fiche « scooter électrique » ; sa fiche `/scooter` est un scooter **thermique** (76,3 g/km). Un vrai cyclomoteur électrique (plus lourd, grosse batterie) émettrait plus. À ré-étiqueter en alias trottinette ou re-sourcer. |

**Liens :**
- ADEME — vélo électrique : https://impactco2.fr/outils/transport/veloelectrique
- ADEME — trottinette : https://impactco2.fr/outils/transport/trottinette
- ADEME — vélo : https://impactco2.fr/outils/transport/velo
- ADEME — comparateur Transport : https://impactco2.fr/outils/transport

### 7.3 Transports en commun — ADEME

**Source officielle :** ADEME — *Impact CO₂* / Base Empreinte, recoupé SNCF Voyageurs (InfoGES
2024) et notre-environnement.gouv.fr. **Unité :** kgCO₂e/**voyageur**.km (déjà normalisé par le
taux d'occupation moyen). Les valeurs seedées correspondent au **périmètre usage** (combustion).

| `mode_id` | Code | Verdict | Note |
|---|---|---|---|
| `coach` | 0,0295 | ✅ **exact** | = autocar longue distance ADEME/SNCF, **29-29,5 g/voyageur.km** (usage). |
| `metro_tram` | 0,005 | ≈ | Bon ordre de grandeur, mais **haut** : métro ≈ 4,44 g (cycle de vie) / tram ≈ 3,2-3,8 g (usage). Arrondi conservateur. Valeur exacte recommandée ≈ 0,0044. |
| `bus` | 0,1515 | ⚠️ | **Surestimé.** Bus urbain ADEME représentatif ≈ 113-128 g (jusqu'à 145,7 g en petite agglo gazole) ; *Impact CO₂* = 122 g cycle de vie ; bus RATP ≈ 84 g. 151,5 g est une **borne haute**, pas un « bus urbain » moyen. |
| `public_transport` | 0,04525 | ❔ | **Pas de facteur « mix TC » publié par l'ADEME** (ventilation par mode uniquement). Valeur **dérivée/custom** (pondération interne bus+métro+tram+car). Le libellé « ADEME 2024-2025 » surestime la provenance → à documenter comme moyenne interne. |

**Liens :**
- ADEME — comparateur Transport : https://impactco2.fr/outils/transport
- ADEME — bus thermique : https://impactco2.fr/outils/transport/busthermique
- ADEME — autocar : https://impactco2.fr/outils/transport/autocar
- ADEME — métro : https://impactco2.fr/outils/transport/metro
- SNCF Voyageurs — Méthodologie InfoGES 2024 (PDF) : https://www.sncf-voyageurs.com/medias-publics/2025-01/sncf_voyageurs_methodologiegenerale-infoges_2024.pdf
- notre-environnement.gouv.fr — GES des transports : https://www.notre-environnement.gouv.fr/themes/climat/les-emissions-de-gaz-a-effet-de-serre-et-l-empreinte-carbone-ressources/article/les-emissions-de-gaz-a-effet-de-serre-des-transports

### 7.4 Train — SNCF Open Data, SNCB, opérateurs

**Sources officielles :** SNCF Open Data (périmètres « usage » et « complet »), SNCB/belgianrail,
Eurostar (facteur DEFRA). La grande différence **TGV ≪ TER** vient du mix électrique (TGV 100 %
électrique sur réseau FR très décarboné + forte occupation ; TER partiellement diesel).

| `mode_id` | Code | Verdict | Note |
|---|---|---|---|
| `train_sncb` | 0,021 | ✅ | = données SNCB **2012-2013** (20,7 / 21,5 g ; 19 g en 2014), traction well-to-wheel. *Caveat :* le chiffre SNCB **actuel** est ~8 g (électricité verte) → la valeur seedée est correcte mais **datée**. |
| `train_ter` | 0,0299 | ≈ | = périmètre **usage 2015** (~29,2 g). **Pas** le périmètre complet SNCF actuel (23,8 g). Incohérence de millésime avec le TGV ci-dessous. |
| `train_hs_fr` | 0,00343 | ✅ | ≈ TGV France actuel **3,5 g/voyageur.km** (à ~2 %). |
| `train_eurostar` | 0,0055 | ≈ | Facteur **officiel DEFRA = 6 g** (cité par Eurostar) ; moyenne communiquée ~5,8 g. 5,5 g est ~9 % sous l'officiel. Valeur canonique = 0,006. |
| `train_thalys` | 0,0084 | ❔ | **Aucune source primaire** (Thalys intégré à Eurostar Group, ancien bilan carbone redirigé). Plausible (entre SNCF 2023 ~6,9 g et presse Thalys ~11,25 g). À documenter comme **estimation**. |

**Liens :**
- SNCF Open Data — périmètre complet : https://ressources.data.sncf.com/explore/dataset/emission-co2-perimetre-complet/
- SNCF Open Data — périmètre usage : https://ressources.data.sncf.com/explore/dataset/emission-co2-perimetre-usage/
- SNCF Connect — calcul des émissions CO₂ : https://www.sncf-connect.com/aide/calcul-des-emissions-de-co2-sur-votre-trajet-en-train
- SNCB / belgianrail — émissions CO₂ du ferroviaire (PDF) : http://www.belgianrail.be/fr/corporate/durabilite/planet/~/media/3EE35EEED3F74468B64DFA702CDCFB8A.ashx
- Eurostar — facteur d'émission (DEFRA 6 g) : https://help.eurostar.com/faq/uk-en/question/What-is-the-Co2-emission-factor-per-kilometer-when-using-Eurostar

### 7.5 Avion & forçage radiatif — DEFRA / DESNZ (UK)

**Source officielle :** *UK Government GHG Conversion Factors for Company Reporting*
(DESNZ + DEFRA), jeu « Business travel – air », en kgCO₂e/passager.km, **forçage radiatif inclus**.

**L'arithmétique de l'app est saine** : les facteurs sont stockés **avec** forçage radiatif et,
quand l'utilisateur le désactive, l'app **divise par le même multiplicateur** (jamais multiplier)
→ pas de double-comptage. **MAIS** plusieurs points de millésime sont à corriger :

| `mode_id` | Code | Verdict | Note |
|---|---|---|---|
| `radiative_forcing` (constante) | 1,9 | ⚠️ | **1,9 est la valeur DEFRA *pré-2023***. Depuis la mise à jour de **juin 2023**, DEFRA/DESNZ recommande **1,7**. 1,9 a été la valeur ~2013-2022. Le mécanisme reste correct ; seul le **chiffre** correspond à un millésime antérieur. |
| `plane_long` | 0,1517 | ✅ | = DEFRA **pré-2023** long-courrier **0,15119** (à 0,4 % près). Confirme que toute la famille aérienne est un **instantané pré-2023**. |
| `plane_domestic` | 0,2582 | ≈ | Dans la plage domestique pré-2023 (0,246-0,267). DEFRA 2024/2025 ≈ 0,229. |
| `plane_short` | 0,2582 | ⚠️ | **Identique à `plane_domestic`** : or DEFRA sépare toujours domestique (plus haut) et court-courrier (~0,123 en 2024/25, ~0,158 pré-2023). Ressemble à une valeur domestique recopiée. |
| `plane_medium` | 0,1872 | ❔ | DEFRA n'a **pas** de bande « moyen-courrier » (seulement domestique / court / long). Valeur **interpolée**, non traçable à une cellule DEFRA. |

⚠️ **Synthèse aérien.** La famille est **cohérente en interne comme un instantané DEFRA pré-2023**
(ancien multiplicateur 1,9 + magnitudes plus hautes), mais **mal étiquetée « DEFRA 2024-2025 »**.
Deux options de correction : (a) ré-étiqueter en « DEFRA ~2019/2022 », garder 1,9, et corriger
`plane_short` vers ~0,158 ; ou (b) re-sourcer sur le jeu **2024/2025** (domestique ≈ 0,229,
court ≈ 0,123, long ≈ 0,117, **RF = 1,7**).

**Liens :**
- GOV.UK — conversion factors 2024 (page) : https://www.gov.uk/government/publications/greenhouse-gas-reporting-conversion-factors-2024
- Jeu condensé 2024 (XLSX) : https://assets.publishing.service.gov.uk/media/6722566a3758e4604742aa1e/ghg-conversion-factors-2024-condensed_set__for_most_users__v1_1.xlsx
- Méthodologie 2024 (PDF — RF = 1,7, §8.42-8.43) : https://assets.publishing.service.gov.uk/media/66a9fe4ca3c2a28abb50da4a/2024-greenhouse-gas-conversion-factors-methodology.pdf
- GOV.UK — Journey emissions comparisons (multiplicateur RF) : https://www.gov.uk/government/publications/transport-energy-and-environment-statistics-notes-and-definitions/journey-emissions-comparisons-methodology-and-guidance

### 7.6 Énergie du bâtiment — DEFRA (domicile) & CIBSE/Exeter (bureau)

**Périmètre :** énergie du bâtiment par jour ouvré (chauffage + électricité + éclairage),
**hors trajet**. L'unité `kgCO2e/day` suppose une journée de **8 h**.

| `mode_id` | Code | Verdict | Note |
|---|---|---|---|
| `home_day` | 2,7 | ✅ **exact** | Facteur **DEFRA/DESNZ « Homeworking » = 0,33378 kgCO₂e/heure** × 8 h = **2,67** → 2,7. Équipement bureau + chauffage (méthodo EcoAct 2020, Ofgem). Attribution « DEFRA » **correcte**. |
| `office_day` | 3,5 | ✅ valeur / ⚠️ source | **0,4375 kgCO₂e/h × 8 h = 3,5** — mais ce **n'est pas un facteur DEFRA**. Source réelle : **CIBSE TM46 « general office » / University of Exeter / Circular Ecology**. Le libellé « DEFRA 2024-2025 » est **incorrect** pour cette ligne. |

**Liens :**
- Circular Ecology — Homeworking vs office (source des **deux** chiffres 2,67 et 3,5) : https://circularecology.com/news/the-carbon-emissions-of-homeworking-and-office-working
- GOV.UK — conversion factors (collection) : https://www.gov.uk/government/collections/government-conversion-factors-for-company-reporting
- EcoAct — Homeworking Emissions Whitepaper 2020 (méthodo DEFRA) : https://info.eco-act.com/en/homeworking-emissions-whitepaper-2020
- CIBSE — TM46 Energy Benchmarks (base du 3,5) : https://www.cibse.org/knowledge-research/knowledge-portal/tm46-energy-benchmarks

### 7.7 Variantes réseau électrique (`car_ev` par pays)

**Construction :** `car_ev = fabrication (~0,0836 kgCO₂e/km, dont ~45 % batterie) + consommation
(~0,20 kWh/km) × intensité carbone du mix national (gCO₂/kWh)`. **Intensités (Ember/OWID, EEA) :**
FR ~40-79, BE ~127-167, EU-27 ~211-292, DE ~336-420 gCO₂/kWh selon l'année.

| Pays | Code | Verdict | Note |
|---|---|---|---|
| `FR` | 0,09 | ✅ | Reconstruction 0,092-0,096. Mix nucléaire FR au plancher. Conservateur vs ADEME 2025 (~0,067). |
| `EU` | 0,13 | ✅ | Reconstruction 0,126-0,136 (<1 %). Excellente cohérence. |
| `DE` | 0,18 | ≈ | Reconstruction 0,151-0,168. 0,18 est en **bord haut** (correspond à un mix DE plus ancien/charbonné ou une conso supérieure). Classement FR < EU < DE respecté. |
| `BE` | 0,1393 | ⚠️ | **Surestimé (~+20-25 %).** Le mix belge (~135 g/kWh) donnerait ~0,11. **Choix assumé** : BE fixé = valeur de base (défaut applicatif), cf. `0004_add_co2.sql` L122-124. Anomalie de classement (BE > EU alors que le réseau BE est plus propre que l'UE). |

**Liens :**
- Our World in Data — intensité carbone de l'électricité (Ember + Energy Institute) : https://ourworldindata.org/grapher/carbon-intensity-electricity
- EEA — intensité GES de la production d'électricité (par pays) : https://www.eea.europa.eu/en/analysis/indicators/greenhouse-gas-emission-intensity-of-1/greenhouse-gas-emission-intensity-of-electricity-generation
- Ember — European Electricity Review 2025 : https://ember-energy.org/latest-insights/european-electricity-review-2025/2024-at-a-glance/

---

## 8. Points d'attention / écarts identifiés

Synthèse des divergences entre la **valeur/source seedée en 2025** et les **publications
vérifiées**. Aucun n'est une erreur de calcul ; ce sont des questions de **millésime, de périmètre
ou d'attribution de source**.

> ✅ **Tous ces points sont traités par le millésime 2026** (§6 bis), désormais le défaut : valeurs
> rafraîchies (#1-#5, #7, #9-#11, #13) ou source/hypothèse clarifiée à valeur égale (#6, #8, #12,
> #14). Le tableau ci-dessous documente l'historique (référentiel 2025, toujours présent et gelé).
> Seul `train_sncb` (0,021) reste volontairement inchangé en 2026 (verrouillé par un test
> d'intégration ; le chiffre SNCB courant ~8 g n'est sourcé que par voie de presse).

| # | Facteur | Constat | Piste de correction |
|---|---|---|---|
| 1 | **Étiquettes « ADEME/DEFRA 2024-2025 »** | Les valeurs correspondent surtout à des **millésimes antérieurs** (voitures = ancien Impact CO₂ ; aérien = DEFRA pré-2023). Ordre de grandeur correct. | Aligner l'étiquette sur le millésime réel, ou rafraîchir les valeurs. |
| 2 | **`radiative_forcing` = 1,9** | Valeur **pré-2023** ; DEFRA recommande **1,7** depuis juin 2023. | Passer à 1,7 **si** on re-source les facteurs aériens 2024/25. |
| 3 | **`plane_short` = `plane_domestic`** (0,2582) | DEFRA sépare toujours les deux (domestique > court). | Mettre court-courrier ≈ 0,158 (pré-2023) ou ~0,123 (2024/25). |
| 4 | **`plane_medium`** | Pas de bande DEFRA « moyen-courrier » → valeur interpolée. | Documenter comme interpolation, ou supprimer la bande. |
| 5 | **`bus` = 0,1515** | Borne haute, pas un bus urbain moyen (~113-128 g). | Ramener vers ~0,113-0,122. |
| 6 | **`public_transport` = 0,04525** | Pas de facteur « mix TC » ADEME ; moyenne **dérivée**. | Documenter la pondération, ou re-sourcer via parts modales. |
| 7 | **`metro_tram` = 0,005** | Arrondi haut (réel ≈ 0,0044). | Optionnel : 0,0044. |
| 8 | **`scooter_elec` = 0,0249** | = recopie trottinette ; pas de fiche ADEME scooter électrique. | Ré-étiqueter en alias trottinette, ou re-sourcer (ACV scooter élec). |
| 9 | **`train_ter` = 0,0299** | Périmètre **usage 2015** ; incohérent avec le TGV (≈ actuel). | Harmoniser : TER complet actuel ≈ 0,0238. |
| 10 | **`train_thalys` = 0,0084** | Aucune source primaire. | Documenter comme estimation, ou facteur Eurostar Group. |
| 11 | **`train_eurostar` = 0,0055** | Sous l'officiel DEFRA (6 g). | Optionnel : 0,006. |
| 12 | **`office_day` source** | Valeur 3,5 correcte, mais source = **CIBSE TM46 / Circular Ecology**, pas DEFRA. | Corriger la chaîne `source`. |
| 13 | **`car_ev` BE = 0,1393** | Surestime la Belgique (~+20-25 %) vs son mix réseau. | Choix assumé ; ~0,11 pour une méthodo homogène. |
| 14 | **`car_phev` = 0,1021** | Hypothèse d'usage très électrique (optimiste). | Documenter l'hypothèse, ou facteur usage-réel plus haut. |

---

## 9. Exemples chiffrés (validés par les tests)

Issus de `co2_calculator.rs` (tests `ac1`…`ac10`). Paramètres par défaut : `BE`, forçage radiatif
activé, énergie bâtiment désactivée, 1 occupant.

| Scénario | Calcul | Résultat |
|---|---|---|
| Voiture essence, 20 km aller simple | `20 × 0,2388` | **4,776 kg** |
| Voiture essence, 20 km aller-retour | `20 × 0,2388 × 2` | **9,552 kg** |
| Voiture moyenne, 30 km A/R, 3 occupants | `30 × 0,2311 × 2 ÷ 3` | **4,622 kg** |
| TGV, 400 km A/R | `400 × 0,00343 × 2` | **2,744 kg** |
| Vélo électrique, 8 km A/R | `8 × 0,01095 × 2` | **0,1752 kg** |
| Marche 3 km / vélo 12 km A/R | `× 0` | **0 kg** |
| Avion court, 500 km, **forçage activé** | `500 × 0,2582` | **129,1 kg** |
| Avion court, 500 km, **forçage désactivé** | `500 × 0,2582 ÷ 1,9` | **≈ 67,95 kg** |
| Semaine : 3 j voiture essence 15 km A/R + 2 j télétravail | `(15 × 0,2388 × 2) × 3 + 0 × 2` | **21,492 kg** |
| Semaine idem + énergie bâtiment | `(7,164 + 3,5) × 3 + 2,7 × 2` | **37,392 kg** |
| Multimodal : TGV 30 km A/R + e-bike 5 km A/R | `30 × 2 × 0,00343 + 5 × 2 × 0,01095` | **≈ 0,3153 kg** |

---

## 10. Récapitulatif des sources

| Famille | Organisme | Référence principale |
|---|---|---|
| Voiture, mobilité active, transports en commun | **ADEME** (Agence de la transition écologique) | Base Empreinte® / *Impact CO₂* |
| Train (France) | **SNCF Voyageurs** | SNCF Open Data + Méthodologie InfoGES 2024 |
| Train (Belgique) | **SNCB** | belgianrail — émissions CO₂ du ferroviaire |
| Train (Eurostar / Thalys) | **Eurostar Group** | facteur DEFRA cité par Eurostar |
| Avion + forçage radiatif | **DEFRA / DESNZ** (UK) | GHG Conversion Factors for Company Reporting |
| Énergie domicile (télétravail) | **DEFRA / DESNZ** (UK) | facteur « Homeworking » (méthodo EcoAct 2020) |
| Énergie bureau | **CIBSE TM46 / Univ. of Exeter / Circular Ecology** | benchmark « general office » |
| Intensité carbone des réseaux électriques | **Ember / Energy Institute** (via Our World in Data), **EEA** | Carbon intensity of electricity |

> *Document généré à partir du code (`co2_calculator.rs`, `0004_add_co2.sql`) et d'une
> vérification web multi-sources. Pour rafraîchir le référentiel, voir les écarts en §8.*
