# BasicPresence — présentation du produit

> Suivez votre **présence** (bureau / télétravail) et l'**empreinte carbone** de vos trajets
> domicile-travail, **par personne** — dans une application desktop **100 % locale** et **chiffrée
> au repos**, sans jamais confier vos données à un service en ligne.

BasicPresence est un logiciel de bureau (Windows / macOS / Linux) qui répond à une question simple :
*« combien de jours ai-je passés au bureau ou en télétravail, et quelle empreinte carbone mes
déplacements ont-ils générée ? »* — tout en traitant ces données comme **hautement confidentielles**.

---

## À quoi ça sert

Le suivi de présence et l'empreinte carbone domicile-travail sont des données **sensibles** :
habitudes de déplacement, présence effective, organisation personnelle. Les confier à un SaaS, c'est
les exposer. BasicPresence prend le parti inverse : **tout reste sur votre machine**, la base ne se
déchiffre qu'après saisie de votre mot de passe, et aucune donnée ne transite par le réseau.

En pratique, l'application permet de :

- **encoder ses journées** dans un calendrier (bureau, télétravail, congés, jour férié) ;
- **décrire ses trajets** (train + vélo, voiture, avion…) et obtenir automatiquement l'**empreinte
  carbone** de chaque journée ;
- **suivre ses heures et ses tâches** au jour le jour ;
- **visualiser** présence, heures et CO₂ dans des graphiques par semaine ou par mois ;
- **exporter** le tout vers un tableur, ou transférer un profil complet vers une autre machine.

## Pour qui

- **Aujourd'hui** : un usage **individuel**. L'appareil a un unique compte propriétaire, mais on peut
  gérer **plusieurs profils** (par exemple soi-même et un proche, ou plusieurs contextes) sur la même
  machine.
- **Demain** (voir [Vision](#vision--ce-qui-arrive)) : le **reporting carbone d'entreprise**, avec
  consolidation multi-employés et alignement réglementaire.

---

## Tour des fonctionnalités

### 🗓️ Calendrier de présence

Le cœur de l'application. Une grille mensuelle où chaque jour se qualifie en un clic parmi **quatre
statuts** :

| Statut | Empreinte carbone |
| --- | --- |
| **Bureau** | calculée selon le trajet déclaré |
| **Télétravail** | calculée (par défaut « aucun déplacement ») |
| **Congés** | toujours 0 |
| **Jour férié** | toujours 0 |

La saisie se fait en deux étapes (choix du statut, puis du trajet). Un **copier-coller de journée
type** permet de reproduire une configuration complète (statut + trajet + horaire + tâches) sur
plusieurs jours d'un coup, avec confirmation avant d'écraser un jour déjà rempli. Chaque cellule
affiche, au choix, l'**empreinte CO₂** ou les **heures travaillées** du jour.

### 🚆 Modes de déplacement

On enregistre des **trajets réutilisables**, éventuellement **multimodaux** — par exemple
« 30 km de train + 5 km de vélo ». Chaque trajet gère l'**aller-retour** et, pour la voiture, le
nombre d'**occupants** (covoiturage). L'application couvre une large palette de modes :

- **Voiture** : essence, diesel, motorisation moyenne, hybride rechargeable, électrique, taxi/VTC ;
- **Mobilité douce** : marche, vélo, vélo électrique, trottinette, scooter électrique ;
- **Transports en commun** : bus urbain, autocar, métro/tram, « transport en commun » générique ;
- **Train** : Intercity (SNCB), régional (TER), TGV, Eurostar, Thalys ;
- **Avion** : intérieur, court, moyen et long-courrier.

### 🌱 Empreinte carbone

Pour chaque journée de présence, l'application calcule l'empreinte en **kgCO₂e**, **par personne et
par jour**, à partir d'un **référentiel de facteurs d'émission versionné par année** (millésime 2025
figé, 2026 par défaut). Le calcul est paramétrable :

- **forçage radiatif** de l'aviation activable/désactivable ;
- **énergie du bâtiment** (bureau / domicile) optionnelle ;
- **pays du réseau électrique** pour les véhicules électriques (France, Belgique, Allemagne, Europe) ;
- **occupants** par voiture pour le covoiturage.

Point clé pour la confiance : le résultat d'un jour est **figé à l'encodage** (avec le millésime de
facteurs utilisé). Faire évoluer le référentiel ne réécrit **jamais** les journées passées — les
chiffres restent **reproductibles**.

> 📖 Formules, facteurs détaillés et sources : [`calcul-impact-co2.md`](./calcul-impact-co2.md).

### 🔬 Méthodologie consultable dans l'application

Une page dédiée expose, en toute transparence : les **formules** de calcul, les **paramètres**, le
**tableau des facteurs** en vigueur, des **exemples chiffrés** calculés en direct, les **sources
officielles** (liens cliquables) et une section **« Transparence & limites »** qui assume les
approximations. Rien n'est caché : l'utilisateur peut vérifier d'où vient chaque chiffre.

### ⏱️ Heures & tâches

Sur chaque jour travaillé, on encode le **détail des heures** : des **tâches** (modèles réutilisables
ou ponctuelles) avec durée (par pas de 5 minutes) et couleur, visualisées dans un **anneau de
répartition**. On saisit l'**heure de début** ; l'heure de fin est **calculée automatiquement**.
Chaque journée peut recevoir une **note libre en Markdown** (éditeur WYSIWYG). Tout est enregistré en
continu, sans bouton « sauvegarder ».

### 📊 Statistiques

Des graphiques (moteur Apache ECharts) donnent une vue d'ensemble : **répartition des journées** par
type, **émissions de CO₂** et **heures travaillées** agrégées **par semaine ou par mois**, avec
filtres, sélection de plage et affichage des 12 dernières périodes par défaut.

### 👥 Multi-profils

Plusieurs profils cohabitent sur le même appareil, chacun avec ses propres réglages (heure de départ
par défaut, pays du réseau, affichage des cellules, police des notes…). On bascule de profil en un
clic depuis le menu du compte.

### 📤 Export & partage

- **Export tableur `.ods`** (ouvrable dans Excel ou LibreOffice) avec un onglet par type de données
  (Présences, Tâches, Trajets, Notes) et choix des colonnes à inclure.
- **Transfert de profil** par bundle `.json` : on choisit les catégories à exporter, et à l'import on
  décide de créer un **nouveau profil** ou de **fusionner** dans un profil existant, avec gestion des
  conflits (ignorer / remplacer).

### ✨ Confort

Thème **clair/sombre**, interface en **français et anglais**, tooltips d'aide, badge de version, et
**mises à jour automatiques signées** : l'application vérifie discrètement les nouvelles versions au
démarrage et signale leur disponibilité, mais **n'installe rien sans action de l'utilisateur**.

---

## Ce qui le rend fiable

### 🔒 Confidentialité « niveau banque »

BasicPresence est **offline-first** et **chiffré au repos**. Concrètement :

- les données métier vivent dans un coffre chiffré (**AES-256-CBC**), dont la clé est **dérivée de
  votre mot de passe** via **Argon2id** (profil OWASP) puis une **envelope encryption** authentifiée
  (**XChaCha20-Poly1305**) ;
- un mécanisme **anti-altération** (HMAC-SHA256) refuse d'ouvrir un coffre modifié en dehors de
  l'application ;
- la **session dure 15 minutes**, uniquement en mémoire, avec **déconnexion automatique en cas
  d'inactivité** — le mot de passe est redemandé à chaque démarrage ;
- une **protection anti-force brute** verrouille le compte après plusieurs tentatives échouées ;
- aucun secret n'est exposé à l'interface ni transmis sur le réseau.

> 🔐 Détails, garanties et limites connues : [`auth.md`](./auth.md).

### 📐 Rigueur méthodologique du calcul CO₂

Les facteurs d'émission s'appuient sur des **sources officielles nommées** (ADEME, DEFRA/DESNZ, SNCF,
SNCB, Our World in Data / EEA), sont **versionnés par millésime**, et chaque résultat est
**reproductible** (les jours encodés sont figés). La documentation **assume ses limites** plutôt que
de les masquer : c'est cette **traçabilité** qui fait la valeur, plus que la prétention à un chiffre
parfait.

> 📖 Méthodologie complète et sources vérifiées : [`calcul-impact-co2.md`](./calcul-impact-co2.md).

### 🧱 Qualité d'ingénierie

Application native **Tauri v2** (backend **Rust**) + **React 19 / TypeScript**, construite en **clean
architecture**, et soumise à des **audits de sécurité et d'architecture internes réguliers**, dont
les rapports complets — constats ouverts compris — sont publiés dans ce dépôt.

> 🛡️ Audits d'architecture et de sécurité : [`audit-securite-architecture.md`](./audit-securite-architecture.md)
> (juillet 2026) et [`audit-securite-architecture-v2.md`](./audit-securite-architecture-v2.md) (seconde
> passe). Aucun audit par un tiers indépendant n'a été réalisé à ce jour.

---

## Multiplateforme

BasicPresence est une **application desktop native**, disponible sur **Windows**, **macOS** et
**Linux**, avec des installeurs pour chaque plateforme (`.deb`, `.rpm`, AppImage, `.msi`, NSIS,
`.dmg`).

---

## Vision — ce qui arrive

> ⚠️ **Cette section décrit la trajectoire du produit, pas des fonctionnalités déjà livrées.**

### Conformité carbone pour l'entreprise (roadmap)

Le constat fondateur : BasicPresence **calcule déjà la bonne donnée physique** (kgCO₂e par trajet,
mode et jour, avec facteurs versionnés) — il lui manque la **structuration réglementaire** pour un
usage en entreprise. Une trajectoire en plusieurs lots est planifiée pour aligner l'outil sur les
cadres de reporting :

- **BEGES réglementaire V5** (France) — poste 3.3 « Déplacements domicile-travail » ;
- **GHG Protocol** — Scope 3, catégorie 7 « Employee commuting » ;
- **CSRD / ESRS E1-6** (Union européenne) ;
- **cadre belge** — diagnostic fédéral mobilité.

Au programme : mapping Scope/poste, unité **tCO₂e**, **export auditable** (retracer donnée → facteur
→ résultat), **consolidation individu → entreprise**, facteurs belges, modélisation du télétravail…

> 🗺️ Plan détaillé, lots et cadres visés : [`conformite/README.md`](./conformite/README.md).

### Synchronisation cloud (Phase 2)

La version actuelle est **100 % locale**. Une **synchronisation cloud** (via libSQL / Turso) est
prévue en Phase 2, en préservant le modèle de confidentialité.

> ☁️ Persistance et compromis chiffrement/synchro : [`turso.md`](./turso.md).

---

## Licence & modèle

BasicPresence suit un modèle **open-core / freemium** (licence propriétaire, droit belge, auteur
g0def) :

- **gratuit** pour l'usage **personnel** et les **petites structures** (sous un seuil d'utilisateurs
  actifs) — avec droit d'utiliser, copier, modifier et distribuer en conservant le crédit de l'auteur ;
- **licence commerciale** (abonnement annuel) requise **au-delà** de ce seuil.

> ℹ️ Le fichier [`../LICENCE`](../LICENCE) fait foi. Le seuil exact d'utilisateurs actifs est en cours
> de clarification et n'est donc pas chiffré ici.

---

## En bref

| Fonctionnalité | Statut |
| --- | --- |
| Calendrier de présence (4 statuts) | ✅ Livré |
| Modes de déplacement multimodaux | ✅ Livré |
| Empreinte carbone par jour/personne (référentiel versionné) | ✅ Livré |
| Page méthodologie & sources | ✅ Livré |
| Heures, tâches & notes Markdown | ✅ Livré |
| Statistiques (semaine/mois) | ✅ Livré |
| Multi-profils | ✅ Livré |
| Export `.ods` + transfert de profil `.json` | ✅ Livré |
| Thème clair/sombre, FR/EN, mises à jour signées | ✅ Livré |
| Chiffrement au repos & session sécurisée | ✅ Livré |
| Conformité réglementaire entreprise (BEGES/GHG/CSRD/BE) | 🗺️ Vision |
| Consolidation multi-employés | 🗺️ Vision |
| Synchronisation cloud | 🗺️ Vision (Phase 2) |

### Pour aller plus loin

- Installation & développement : [`../README.md`](../README.md)
- Sécurité & confidentialité : [`auth.md`](./auth.md)
- Méthodologie CO₂ : [`calcul-impact-co2.md`](./calcul-impact-co2.md)
- Audit sécurité & architecture : [`audit-securite-architecture.md`](./audit-securite-architecture.md)
- Roadmap conformité entreprise : [`conformite/README.md`](./conformite/README.md)
- Persistance & synchro cloud : [`turso.md`](./turso.md)
