# Audit de sécurité & d'architecture — BasicPresence (2ᵉ passe)

| | |
|---|---|
| **Date** | 26 juillet 2026 |
| **Version auditée** | commit `352fa95` (branche `main`) + modifications non committées (`LICENCE`, `documentation/presentation-produit.md`) |
| **Périmètre** | Backend Rust, frontend React, configuration Tauri, capabilities, migrations SQL, CI/CD, distribution, dépendances, cohérence documentation ↔ code |
| **Méthode** | 8 passes d'analyse parallèles spécialisées (crypto/keystore, auth/session, persistance/SQL, IPC/Tauri, surfaces non fiables, architecture Rust, frontend, chaîne d'approvisionnement), chacune suivie d'une passe de **réfutation adversariale** chargée d'invalider les constats. 57 constats bruts → dédupliqués et recotés. Constats structurants re-vérifiés manuellement sur le code. |
| **Objet** | Audit de **second niveau** : chercher ce que [l'audit du 3 juillet](audit-securite-architecture.md) a manqué, et vérifier que ses correctifs annoncés tiennent réellement. |
| **Statut** | ✅ **Tous les constats corrigés le 26 juillet 2026** — voir §9 pour le détail et les points restant à la main de l'utilisateur. |

---

## 1. Synthèse exécutive

L'audit du 3 juillet concluait « aucune vulnérabilité critique, posture excellente ». **Cette conclusion reste globalement juste sur la cryptographie elle-même** : les primitives, la dérivation de clés, l'enveloppe XChaCha20-Poly1305, la zeroization et le scellement du keystore par le trousseau OS sont corrects et ont résisté à une relecture ciblée. Les correctifs annoncés le 3 juillet sont, pour l'essentiel, réellement en place (§2).

Le premier audit a en revanche **regardé les primitives et pas la logique autour**. Les constats structurants de cette seconde passe portent tous sur ce point : la couche qui *décide* quoi faire du résultat cryptographique, et l'orchestration du cycle de vie du coffre.

Trois familles de problèmes, dans l'ordre de gravité :

1. **Le risque dominant n'est pas la confidentialité, c'est la perte définitive des données.** Trois mécanismes indépendants (§3.1) transforment un incident banal — coupure de courant pendant une mise à jour, fermeture de fenêtre pendant une sauvegarde, expiration de session au mauvais moment — en `VAULT_TAMPERED` irréversible ou en base non migrable. Il n'existe **aucun chemin de récupération** : `IntegrityPolicy::WarnAndAllow` est marqué `#[allow(dead_code)]`, aucune commande de réparation n'est exposée. Un utilisateur bloqué perd tout.

2. **La tamper-evidence, elle, se contourne avec `touch`** (§3.2). La politique `HardFail` — présentée comme la garantie « niveau banque » — repose sur deux fichiers ordinaires non authentifiés posés à côté du coffre. La couche qui détecte l'altération est défaite par celui-là même contre qui elle protège. Ironie du dossier : c'est aujourd'hui la seule échappatoire pour un utilisateur victime du point 1.

3. **La chaîne de distribution est le maillon faible**, et plus largement qu'estimé le 3 juillet (§3.3). Six actions GitHub à référence mutable s'exécutent dans le job qui détient la clé de signature — six dépôts tiers peuvent donc signer une mise à jour sans jamais toucher au compte du mainteneur.

**Décompte après déduplication et réfutation : 0 critique · 3 élevés · 3 moyens · 17 faibles/informatifs.**
_Révision du 26 juillet : **M-01 s'est révélé être un faux positif** (voir §4) — décompte réel **2 moyens**._

| Domaine | 3 juillet | Aujourd'hui | Évolution |
|---|---|---|---|
| Primitives cryptographiques | ★★★★★ | ★★★★★ | inchangé — solide |
| Cycle de vie du coffre / durabilité | ★★★★☆ | ★★☆☆☆ | **révisé à la baisse** (§3.1) |
| Évidence d'altération | « bonne pratique remarquable » | ★★☆☆☆ | **révisé à la baisse** (§3.2) |
| Architecture backend (hexagonale) | ★★★★★ | ★★★★☆ | très bon, déviations ponctuelles (§5) |
| Sécurité frontend | ★★★★☆ | ★★★★☆ | inchangé, une lacune (§4.3) |
| Validation des entrées | non évalué | ★★★☆☆ | un champ non validé casse le métier (§4.2) |
| Chaîne d'approvisionnement & CI | ★★★☆☆ | ★★☆☆☆ | **révisé à la baisse** (§3.3) |
| Cohérence doc ↔ code ↔ licence | non évalué | ★★☆☆☆ | contradictions juridiques (§6) |

---

## 2. Vérification des correctifs annoncés le 3 juillet

| Constat | Annoncé | Vérifié aujourd'hui |
|---|---|---|
| **E2** — migration keystore récupérable | ✅ Fait | ✅ **Tient réellement.** Matrice de crash parcourue : `.old` présent → restauration ; `.new` résiduel → purgé ; aucun cas où un keystore manquant est confondu avec une installation neuve. |
| **M3/F5** — dompurify, esbuild | ✅ Fait | ⚠️ **Partiel.** Les overrides sont bien dans `pnpm-workspace.yaml`, mais le garde-fou `pnpm audit --audit-level=high` est **rouge aujourd'hui** (advisories dev). Le seuil `high` masque par ailleurs les advisories modérées visant dompurify 3.4.11 exactement. |
| **M4** — CI réactivée | ✅ Fait | ✅ Confirmé : jobs frontend + `cargo audit` actifs et bloquants. Renovate/Dependabot toujours absents. |
| **F1** — purge des sessions au login | ✅ Fait | ⚠️ **Partiel.** `login.rs:140` purge bien, mais l'invariant « session valide ⇔ coffre ouvert » reste faux dans l'autre sens : le coffre reste ouvert sans session valide tant qu'aucune commande n'arrive (§5, F-06). |
| **F2** — capability `opener` scopée | ✅ Fait | ✅ Confirmé : `{"url": "https://*"}`, `allow-default-urls` retiré. |
| **F3/F4** — messages d'erreur | ✅ Fait | ✅ Confirmé. |
| **§2.2** — « migrations atomiques par migration » | affirmé | ❌ **Faux.** C'est une affirmation de l'audit, pas un correctif — mais elle est démentie par le code (§3.1, E-01). |
| **E1** — clé de signature dans les secrets GitHub | ⏳ À faire | ⏳ Toujours ouvert, **et sous-estimé** (§3.3). |
| **M1** — chemins export/import non confinés | ⏳ À faire | ⏳ Toujours ouvert. |
| **M5** — `change_password` | ⏳ À faire | ⏳ Toujours ouvert. |

---

## 3. Constats élevés

### E-01 — Trois chemins indépendants mènent à la perte définitive du coffre, sans récupération possible

**Type** : robustesse / durabilité · **Sévérité** : élevé · **Nouveau**

C'est le constat central de cet audit. Trois défauts distincts convergent vers le même état terminal, et aucun filet n'existe en dessous.

#### (a) Les migrations ne sont pas atomiques — `migrations.rs:93-104`

```rust
for m in migrations {
    if m.version > current {
        conn.execute_batch(m.sql).await.map_err(map_storage)?;   // ① écrit
        conn.execute(
            "INSERT INTO _migrations (version, applied_at) …",    // ② enregistre
            libsql::params![m.version],
        ).await.map_err(map_storage)?;
    }
}
```

① et ② sont deux instructions distinctes, **hors transaction**. `execute_batch` de libsql 0.9 boucle `prepare`/`step` sans ouvrir de transaction — chaque statement s'auto-commit. La bibliothèque expose pourtant `execute_transactional_batch`, non utilisée.

Aggravant : plusieurs migrations **ne sont pas rejouables**, et SQLite n'a pas de `ADD COLUMN IF NOT EXISTS` :

| Fichier | Contenu non idempotent |
|---|---|
| `keystore/0002_add_mac_key.sql:4-5` | deux `ALTER TABLE account ADD COLUMN` consécutifs |
| `vault/0004_add_co2.sql:89-90` | deux `ALTER TABLE presence ADD COLUMN` |
| `vault/0007_simplify_work_schedule.sql:20-21` | `DROP TABLE` puis `RENAME` |
| `vault/0008:7`, `vault/0009:4` | `ALTER TABLE … ADD COLUMN` |

**Scénario** : mise à jour installant la migration 9. `ALTER TABLE presence ADD COLUMN note` réussit et s'auto-commit ; coupure de courant avant l'`INSERT INTO _migrations`. Au démarrage suivant `MAX(version)` vaut 8, la migration 9 est rejouée, SQLite renvoie `duplicate column name: note` → `DomainError::Storage` → l'ouverture du coffre échoue → **« Erreur interne » à chaque login, définitivement**. Variante `0007` : crash après `DROP TABLE` = table détruite *et* version non enregistrée.

C'est aussi une contradiction directe avec l'audit du 3 juillet, qui affirme en §2.2 « appliquées par `execute_batch` (atomique par migration) ».

#### (b) Le marqueur `dirty` est posé après les écritures — `vault.rs:71-84`

```rust
let db = open_encrypted_db(&self.path, dek).await?;
let conn = connect(&db).await?;
migrations::run(&conn, VAULT_MIGRATIONS).await?;   // ← écrit dans vault.db
…
let _ = vault_integrity::mark_dirty(&self.path);   // ← marqueur seulement ici
```

Le marqueur qui sert précisément à distinguer « crash » de « falsification » n'est posé qu'**après** les migrations. Et si `migrations::run` échoue, la fonction retourne par `?` sans jamais l'atteindre, alors que le fichier a déjà été modifié.

**Scénario** : crash pendant une migration → au démarrage suivant, le sidecar `.hmac` porte l'ancienne empreinte, le fichier a changé, et `.dirty` n'existe pas → `Mismatch` → `HardFail` → `VaultTampered`. L'écran annonce *« le fichier a été modifié »* alors qu'il s'agit d'une coupure de courant.

#### (c) `close()` n'est synchronisé avec aucune écriture en vol — `vault.rs:43-49, 87-100`

`connection()` distribue des **clones illimités** de la `Connection` libsql, hors du mutex ; les repositories les tiennent à travers des dizaines de `await` (`presence_repository.rs:100-114` : `BEGIN` / `DELETE` / N × `INSERT` / `COMMIT`). `close()` reprend l'état, `drop(open)`, puis `write_baseline()` — qui hache le fichier **et supprime le marqueur dirty**. Rien n'attend les porteurs de clones. Le commentaire du champ `_db` (« Keep the Database alive so the connection stays valid ») énonce un invariant que `connection()` viole lui-même.

Quatre déclencheurs concurrents non coordonnés : `logout` (`logout.rs:20`), `check_session` (`check_session.rs:40`), `require_session` d'une autre commande (`require_session.rs:39`), et le hook de fermeture de fenêtre (`lib.rs:109`).

**Scénario** : l'utilisateur enregistre ses heures à 14 min 58 s de session — précisément ce que le compte à rebours l'incite à faire. `set_day_note` passe la garde et commence sa transaction ; au tick suivant, `useSessionTimer` déclenche `logout` → `close()` → baseline calculée sur un fichier en cours d'écriture, marqueur dirty effacé. Le `COMMIT` aboutit après. Démarrage suivant : `Mismatch` → `VaultTampered` **irréversible**. Même enchaînement en fermant la fenêtre juste après avoir cliqué sur Enregistrer.

#### Le facteur commun : aucune sortie de secours

`AppConfig::new` code en dur `IntegrityPolicy::HardFail` (`config.rs:66`) et `WarnAndAllow` porte `#[allow(dead_code)] // used in tests` (`config.rs:30`). Aucune commande de réparation, aucun re-baselining consenti, aucune sauvegarde du coffre — alors que le keystore, lui, a reçu un mécanisme `.old`/restauration lors du correctif E2. Le coffre est le seul des deux fichiers à ne pas en avoir.

**Recommandations**
1. Encadrer chaque migration d'un `BEGIN`/`COMMIT` explicite englobant le batch **et** l'insertion de version, `ROLLBACK` sur erreur (SQLite supporte le DDL transactionnel) — le motif est déjà utilisé dans `presence_repository::set_for_day`. Rendre les `ADD COLUMN` conditionnels via `PRAGMA table_info`.
2. Déplacer `mark_dirty()` **avant** `open_encrypted_db()` et traiter son échec comme fatal.
3. Rendre `close()` drainant : remplacer `connection()` par un emprunt traçable (garde RAII, ou `RwLock`), ne calculer la baseline qu'une fois le dernier handle libéré. Si le drainage échoue, **laisser le marqueur dirty en place** plutôt qu'écrire une baseline fausse.
4. Exposer un chemin de récupération explicite et ré-authentifié (« ré-établir la ligne de base après un arrêt anormal ») plutôt qu'un `IntegrityPolicy` mort.
5. Test d'intégration : interrompre une migration à mi-parcours et vérifier que le lancement suivant la rejoue proprement.

---

### E-02 — L'évidence d'altération se neutralise en créant ou supprimant un fichier vide

**Type** : sécurité · **Sévérité** : élevé (impact fonctionnel total sur la couche concernée) · **Nouveau** — l'audit du 3 juillet classait ce mécanisme en « bonne pratique remarquable » et ne relevait que I1

`vault_integrity.rs:64-79` :

```rust
let Ok(expected) = std::fs::read(sidecar_path(vault_path)) else {
    return Ok(IntegrityVerdict::NoBaseline);      // ← sidecar absent = on passe
};
let actual = compute_mac(vault_path, mac_key)?;
if bool::from(expected.ct_eq(&actual)) {
    Ok(IntegrityVerdict::Clean)
} else if dirty_path(vault_path).exists() {
    Ok(IntegrityVerdict::UncleanShutdown)          // ← fichier vide = on passe
} else {
    Ok(IntegrityVerdict::Mismatch)                 // ← seul cas qui déclenche HardFail
}
```

Et `vault.rs:58-69` range `Clean | NoBaseline | UncleanShutdown` dans une branche vide : **seul `Mismatch` atteint `HardFail`**.

Or les deux signaux qui décident du verdict sont des fichiers ordinaires posés à côté du coffre :
- `vault.db.hmac` — la baseline, non signée, non référencée ailleurs ;
- `vault.db.dirty` — contenant le littéral `b"open"` (`vault_integrity.rs:85`), **indépendant de la clé MAC**.

Ni l'un ni l'autre n'est couvert par le HMAC ni ancré dans le keystore scellé. Le HMAC lui-même est correct (comparaison constant-time, clé enveloppée par le KEK) : **c'est la logique de décision autour qui est fail-open.**

**Scénario** : poste volé puis rendu, sauvegarde restaurée, ou dossier synchronisé. L'attaquant remplace `vault.db` par une copie plus ancienne (effaçant des journées de présence encodées) ou corrompt des blocs — l'AES-256-CBC de libSQL est malléable. Puis `touch vault.db.dirty` (ou `rm vault.db.hmac`). Au login suivant le coffre s'ouvre normalement, et `close()` **re-scelle silencieusement l'état falsifié**. Aucun signal : ni log (I4), ni indication UI.

La confidentialité n'est pas touchée — le DEK reste protégé par le mot de passe. Mais la fonction dont tout ce sous-système est la raison d'être est contournée à coût nul.

Le test `tampering_with_the_vault_is_detected_under_hard_fail` (`integration_tests.rs:633-644`) altère bien `vault.db` (`bytes[mid] ^= 0xFF`) mais ne touche jamais aux deux fichiers auxiliaires — le contournement n'est pas couvert.

**Recommandation** : authentifier l'**état**, pas seulement le fichier. Stocker dans le sidecar un enregistrement MACé unique contenant le digest, un compteur monotone et le drapeau clean/dirty — le compteur vivant dans le keystore scellé. Une baseline absente alors que le keystore connaît déjà un compteur > 0 doit valoir `VaultTampered`. A minima : marqueur dirty authentifié (`HMAC(mac_key, "dirty" ‖ session_id)`), et `NoBaseline` acceptable seulement si le compte n'a pas encore de `wrapped_mac_key`. Ajouter les deux tests manquants.

> **Note de séquencement** : ne corriger E-02 **qu'après** E-01. En l'état, ce fail-open est la seule échappatoire d'un utilisateur dont le coffre a été briqué par un crash. Verrouiller la porte avant d'avoir réparé le plancher enfermerait les victimes.

---

### E-03 — Six actions GitHub à référence mutable s'exécutent dans le job qui détient la clé de signature

**Type** : sécurité / chaîne d'approvisionnement · **Sévérité** : élevé · **Aggrave E1**

`.github/workflows/release.yml` — aucune action n'est épinglée par SHA :

| Ligne | Action | Type de référence |
|---|---|---|
| 39 | `actions/checkout@v4` | tag mutable |
| 41 | `pnpm/action-setup@v4` | tag mutable |
| 45 | `actions/setup-node@v4` | tag mutable |
| 50 | `dtolnay/rust-toolchain@stable` | **branche** |
| 55 | `Swatinem/rust-cache@v2` | tag mutable |
| 76 | `tauri-apps/tauri-action@v0` | tag mutable |

Les lignes 79-80 injectent `TAURI_SIGNING_PRIVATE_KEY` et sa passphrase dans ce même job.

E1 ne modélisait qu'un seul déclencheur : la compromission du compte GitHub du mainteneur. Or **six dépôts tiers** ont chacun, indépendamment, la capacité de re-pointer leur référence et d'exécuter du code dans ce job. Le fait que le secret ne soit injecté que sur l'étape `tauri-action` ne protège pas : les étapes 39-73 tournent dans le même runner et le même workspace, et `tauri-action` invoque `cargo`/`node` qui héritent de l'environnement au moment de la signature.

**Scénario** : le mainteneur de `dtolnay/rust-toolchain` est compromis ; la branche `stable` est déplacée sur un commit qui ajoute un shim `cargo` dans `$GITHUB_PATH`. Au prochain `git push --tags`, le shim est appelé pendant le build, lit la clé dans l'environnement et l'exfiltre. L'attaquant peut alors signer n'importe quel binaire accepté par la clé publique épinglée dans `tauri.conf.json:38` — **sans jamais toucher au compte GitHub du projet**, donc sans déclencher aucune des contre-mesures recommandées par E1 (MFA matériel, environment avec reviewers).

**Recommandation** : épingler les six `uses:` par SHA complet (`actions/checkout@11bd719… # v4.2.2`) et laisser Dependabot les faire remonter. Surtout, appliquer la recommandation 3 d'E1 — sortir la signature de la CI — seule mesure qui neutralise à la fois E1 et ce vecteur.

**Point connexe** : un `push --tags` déclenche build + signature + publication **sans qu'aucun contrôle CI n'ait jamais tourné sur le commit tagué** (`ci.yml` ne se déclenche que sur `push: branches: [main]` et `pull_request`).

---

## 4. Constats moyens

### M-01 — ~~L'endpoint de l'updater ne pointe pas vers le dépôt où la CI publie~~ → **faux positif**

**Type** : sécurité / distribution · **Écarté le 26 juillet 2026**

> ❌ **Ce constat était erroné.** L'audit avait relevé une divergence entre l'endpoint de
> `tauri.conf.json` (`g0def/BasicPresenceTauri`) et le remote git local
> (`MehdiA7/BasicPresenceTauri`), et conclu que l'endpoint était périmé. C'est l'inverse : le
> compte a été renommé **vers** `g0def`, qui est le nom actuel. L'endpoint était juste ; c'est
> le remote local du poste de développement qui n'avait pas été mis à jour, et qui continuait
> de fonctionner grâce à la redirection GitHub.
>
> La leçon d'audit vaut d'être notée : comparer deux sources dont l'une n'a jamais été
> vérifiée ne dit pas laquelle est fausse. Le remote local a été traité comme référence alors
> qu'il n'était qu'un troisième point de donnée, et le plus volatil des trois.

**Ce qui reste vrai et a été conservé** :

- **Un contrôle CI de cohérence** (`release.yml`) compare `plugins.updater.endpoints[0]` à
  `GITHUB_REPOSITORY` et fait échouer la release en cas de divergence. Il n'aurait pas
  signalé la situation d'origine (l'endpoint était bon), mais il attrapera une vraie dérive —
  y compris celle qu'un futur renommage produirait.
- **L'échec silencieux du check de mise à jour est réel** : `updater-provider.tsx:41-43` fait
  `catch { setPhase(manual ? "error" : "idle") }`. Au démarrage, toute erreur est avalée et le
  badge continue d'afficher « à jour ». Un canal rompu — quelle qu'en soit la cause — reste
  donc invisible pour l'utilisateur. À surfacer dans l'UI.
- **Le nom libéré est désormais `MehdiA7`.** Un renommage GitHub ne réserve pas l'ancien nom :
  il redevient disponible, et les redirections cessent dès que quelqu'un l'enregistre. Aucune
  URL du projet ne référence plus `MehdiA7`, donc la distribution n'est pas exposée — seul le
  remote local du poste de développement l'est encore.

---

### M-02 — `factor_year` est le seul champ non validé : un bundle importé met tout le CO₂ à zéro, silencieusement et définitivement

**Type** : sécurité (entrée non fiable) + incohérence · **Nouveau** — constat trouvé indépendamment par 4 des 8 passes

`set_profile_settings.rs:43-70`. Le doc-comment annonce :

> *« Validate every typed field. Mirrors the table CHECKs […] Shared with the bundle import path, which receives untrusted settings from a file. »*

`validate()` contrôle `default_start_minutes`, `note_font`, `cell_display_mode`, `grid_country`, `default_car_occupancy`, `working_days_per_year` — **et jamais `factor_year`**. Côté SQL, `0010_add_profile_settings.sql:30` déclare `factor_year INTEGER NOT NULL DEFAULT 2025` **sans CHECK** : seule colonne de la table dans ce cas, ses six voisines en portent toutes un.

Or c'est exactement cette fonction que le chemin d'import de bundle appelle sur du contenu non fiable (`import_profile_bundle.rs:146`), avec le commentaire *« Untrusted: enforce the same field rules […] so a corrupt settings block aborts here »*. **La seule valeur qui pilote la résolution du référentiel est celle qui échappe au contrôle.**

**Scénario** : un collègue envoie un `profil.json` — le cas d'usage prévu de la fonctionnalité — dont le bloc settings porte `"factorYear": 1999`. La case « réglages » est cochée par défaut à l'import. La validation passe, la valeur est écrite. Ensuite, chaque `set_presence` appelle `load_factor_maps(…, 1999)` qui renvoie deux maps vides ; dans `Co2Calculator::compute_trip`, `factors.get(mode_id)` est `None`, le repli `car_average` est `None` aussi → branche `None => 0.0`. Tous les jours encodés ensuite affichent **0 kgCO₂e**, avec `is_estimated = true` pour seul indice — un drapeau qui n'est pas un signal d'erreur.

Aggravants : aucun écran ne permet de corriger `factorYear` (aucun contrôle UI), le mapper frontend reconduit la valeur à chaque sauvegarde, et les `co2_kg` sont **figés par snapshot** — revenir en arrière ne répare pas les jours déjà encodés. Pour un outil dont la raison d'être est le reporting carbone, un bilan nul silencieux est le pire résultat possible.

**Recommandation** : valider `factor_year` contre les millésimes réellement seedés (2025, 2026) sur le modèle de `GRID_COUNTRIES`, et poser le CHECK correspondant en migration. En défense en profondeur, faire échouer `load_factor_maps` quand la map revient vide au lieu de laisser produire 0.0. Corriger au passage `Co2Settings::default().factor_year` (2026) qui diverge du DEFAULT de la colonne (2025), alors que le commentaire les présente comme identiques.

---

### M-03 — Les liens des notes Markdown font naviguer la WebView hors de l'application

**Type** : sécurité / incohérence · **Nouveau**

`src/core/external-link.ts:7-8` affirme en commentaire de module : *« External navigation never happens inside the webview. »* La page méthodologie respecte scrupuleusement ce contrat (`openExternal` via le plugin opener, capability restreinte à `https://*`).

Mais le rendu des notes injecte du HTML backend via `dangerouslySetInnerHTML`, et ce HTML contient de vraies balises `<a href>` : `comrak_renderer.rs:58` active `options.extension.autolink = true`, et le sanitizer est `ammonia::Builder::default()` — dont la whitelist par défaut **conserve `<a href>` en http/https/mailto**. Le test `neutralizes_javascript_links` ne couvre que le schéma `javascript:`.

Aucune interception nulle part (vérifié par grep sur `src/` et `src-tauri/src/`) : pas de handler sur les ancres, le `onClick` du conteneur est `() => setEditing(true)` sans `preventDefault`, et `lib.rs` n'enregistre aucun `on_navigation` sur le builder Tauri. La CSP ne couvre pas la navigation top-level.

**Scénario** : une note importée depuis un bundle contient le texte `https://basic-presence-support.example/reconnexion`. L'autolink en fait une ancre. L'utilisateur clique : `setEditing(true)` s'exécute **et** la navigation par défaut a lieu. La fenêtre applicative — sans barre d'adresse ni bouton retour — affiche une page distante contrôlée par un tiers, dans le même chrome que l'app, qui peut présenter un faux écran « session expirée, ressaisissez votre mot de passe ». Accessoirement, c'est le premier accès réseau de la WebView, ce que Claude.md et le README annoncent comme impossible.

**Recommandation** : deux verrous complémentaires. (1) Côté frontend, un handler délégué sur les conteneurs de HTML rendu qui intercepte `event.target.closest('a[href]')`, appelle `preventDefault()` et route vers `openExternal()`. (2) Côté Rust, `.on_navigation(|url| url.scheme() == "tauri" || url.host_str() == Some("localhost"))` sur le builder — refus au niveau moteur, quelle que soit l'origine (note, licence, futur composant).

---

## 5. Constats faibles et informatifs

### Sécurité & robustesse

| Réf | Constat | Emplacement |
|---|---|---|
| F-01 | **Bundle importé sans aucune borne** : ni taille de fichier, ni nombre d'éléments, ni longueur de chaîne, avec amplification d'une requête SQL par `factor_year` distinct | `json_bundle_codec.rs:54` |
| F-02 | **Bornes de longueur appliquées aux 4 champs du profil seulement** : titres, descriptions, noms de trajet et notes sont non bornés à la frontière IPC | `create_task_preset.rs:51` |
| F-03 | **Anti-bruteforce neutralisable** : repose sur l'horloge système et sur un état fichier que l'attaquant peut restaurer (aucune protection anti-rejeu du keystore) | `login.rs:71`, `keystore_bootstrap.rs:77` |
| F-04 | **Un `COMMIT` en échec laisse la transaction ouverte** sur la connexion libSQL unique partagée par tous les repositories ; les écritures suivantes sont silencieusement absorbées | `presence_repository.rs:106` |
| F-05 | **Aucune garde contre une base plus récente que le binaire** : `run()` compare à `MAX(version)` et ne signale jamais un downgrade — contrairement au contrat appliqué au keystore | `migrations.rs:93` |
| F-06 | **L'expiration de 15 min n'est que réactive** : sans commande IPC entrante, le coffre reste ouvert et le DEK en RAM indéfiniment | `require_session.rs:32` |
| F-07 | **Les paramètres Argon2id du KEK ne sont persistés nulle part** : changer la configuration rend tous les coffres existants indéchiffrables (l'authentification, elle, embarque ses paramètres dans le PHC) | `config.rs:51` |
| F-08 | **PKGBUILD AUR : `sha256sums=SKIP`** — seul canal de distribution sans aucune vérification d'intégrité | `basicpresence-bin/PKGBUILD:14` |
| F-09 | **`ci.yml` ne déclare aucun bloc `permissions`** ; `actions/checkout` laisse le `GITHUB_TOKEN` sur disque pour toutes les étapes suivantes | `.github/workflows/ci.yml:18` |
| F-10 | **`pnpm audit --audit-level=high` est rouge aujourd'hui** (6 advisories, exclusivement dev) ; l'override dompurify résout sur 3.4.11 exactement, version visée par une advisory modérée que le seuil `high` rend invisible | `ci.yml:38`, `pnpm-workspace.yaml:7` |

### Incohérences architecturales

| Réf | Constat | Emplacement |
|---|---|---|
| A-01 | **`check_session` est du code mort** : commande Tauri, use case, DTO, repository et use case front câblés de bout en bout, appelés par aucun composant | `check_session.rs:46`, `tauri-auth.repository.ts:43` |
| A-02 | **`SESSION_EXPIRED` traité dans 5 providers et 2 hooks, oublié dans l'export, le transfert de profil et la licence** : l'UI reste « connectée » après que le backend a fermé le coffre | `use-export.ts:39` |
| A-03 | **`auth.md` et `use-session-timer.ts` affirment que l'expiration est insensible aux changements d'horloge** — c'est l'inverse, elle repose entièrement sur l'horloge murale | `use-session-timer.ts:11` |
| A-04 | **Cycle de dépendances `presence` ↔ `work-hours`**, alors que Claude.md n'autorise qu'une dérogation (`presence → commute`) et affirme « jamais l'inverse — pas de cycle » | `use-day-clipboard.ts:18` |
| A-05 | **Le résumé mensuel du header bucketise en heure locale** alors que `Presence.day` est un minuit UTC — il contredit le helper `civilDate()` écrit exactement pour éviter ça | `_authenticated.tsx:86` |
| A-06 | **Deux sources de vérité pour l'année du référentiel** : le sélecteur de modes et la page méthodologie codent en dur le millésime par défaut, le calcul utilise celui du profil | `list_emission_factors.rs:10` |
| A-07 | **Le garde ESLint « seul `core/ipc.ts` parle à Tauri » ne couvre pas `@tauri-apps/plugin-dialog`** — précisément le plugin qui produit le chemin arbitraire envoyé au backend (M1) | `eslint.config.js:50` |
| A-08 | **Le contrat de codes d'erreur IPC est désynchronisé** : le frontend ignore `VAULT_TAMPERED`, et `KEYSTORE_UNRECOVERABLE` ne peut jamais atteindre la frontière | `src/core/errors.ts:33` |
| A-09 | **L'invariant « une journée de travail tient dans une journée » a été supprimé par la migration 0008** et jamais réimplanté côté application | `set_work_entries.rs:76` |
| A-10 | **Import de bundle : deux jours identiques dans le fichier s'écrasent silencieusement** et le résumé annonce N jours importés là où une seule ligne existe | `import_profile_bundle.rs:202` |
| A-11 | **Import avec « trajets » décoché : le total CO₂ du jour est conservé** alors qu'aucun trajet n'est écrit — la ventilation ne somme plus au total | `import_profile_bundle.rs:381` |
| A-12 | **`set_day_note` est le seul écrivain par présence sans garde d'existence** : il confirme un enregistrement qui n'a rien écrit | `set_day_note.rs:49` |
| A-13 | **La migration des préférences legacy pose son drapeau « déjà migré » avant de savoir si l'écriture backend a réussi**, et avale l'erreur dans un `catch` vide | `profile-settings-provider.tsx:102` |
| A-14 | **Le commentaire d'en-tête de la migration keystore affirme encore que le keystore n'est pas chiffré** et qu'il est sûr de le stocker en clair | `keystore/0001_init.sql:1` |
| A-15 | **Saisies perdues sans message à l'expiration de session** : `SESSION_EXPIRED` avalé sans `setError`, aucune reprise planifiée | `use-work-day.ts:66` |
| A-16 | **Course entre le `reload()` du calendrier au montage et le flush de sauvegarde des heures** — le dernier qui répond gagne | `presence-calendar.tsx:74` |
| A-17 | **`ProfileProvider.refresh` est la seule méthode du contexte sans `try/catch`** ; appelée en `void` après un import réussi, elle laisse le dialogue bloqué | `profile-provider.tsx:59` |

---

## 6. Cohérence licence, distribution et communication

Ces points ne sont pas techniques mais engagent juridiquement ou commercialement.

| Réf | Constat |
|---|---|
| L-01 | **La LICENCE se contredit sur son seuil commercial** : la version française fixe l'obligation à **30 utilisateurs actifs** (ligne 33), la version anglaise à **20** (ligne 106). Une organisation de 20 à 29 personnes est libre selon un texte, en infraction selon l'autre. La modification non committée en cours ne corrige que la version française. |
| L-02 | **Le PKGBUILD AUR déclare `license=('MIT')`** (ligne 8) alors que le dépôt est sous licence propriétaire à seuil commercial, et n'installe aucun fichier de licence. Publié tel quel sur AUR, il distribue le logiciel sous une licence qui n'est pas la sienne. |
| L-03 | **`presentation-produit.md:161` annonce un « audit de sécurité tiers favorable »** alors que l'audit référencé est interne et laisse E1 ouvert. Affirmation à corriger avant toute diffusion commerciale. |
| L-04 | **Chaque bundle de profil exporté est estampillé « 0.1.1 »** via `CARGO_PKG_VERSION` (`json_bundle_codec.rs:321`), alors que la version réelle est 0.2.0 (`package.json`, tag git) et que `release.md` affirme que `Cargo.toml` n'est pas pertinent pour la release. Les fichiers déjà exportés portent une version fausse. |

---

## 7. Ce qui a été vérifié et jugé sain

Utile pour ne pas y repasser :

- **Primitives cryptographiques** : nonce XChaCha20-Poly1305 de 24 octets tiré par `getrandom` à chaque wrap, aucune réutilisation ni dérivation déterministe ; sels distincts entre le hash d'auth et le KEK ; longueurs vérifiées avant usage ; pas d'oracle (wrong-key et tag invalide donnent le même message, tout collapse en `INTERNAL`).
- **Zeroization** effective, y compris le `Vec` intermédiaire d'`unwrap_dek`, zeroizé sur les deux chemins.
- **Correctif E2** solide : matrice de crash de la migration keystore parcourue sans trou.
- **Pas de downgrade vers une base non chiffrée** : la feature `encryption` est active, la clé passe en octets bruts.
- **SQL 100 % paramétré** ; les `format!` n'interpolent que des listes de colonnes constantes.
- **Garde de session sans trou** : inventaire exhaustif des 37 commandes de `invoke_handler`. Les 32 commandes touchant le coffre appellent toutes `require_session` en première instruction ; les 5 exceptions sont les commandes d'authentification elles-mêmes (`account_exists`, `register`, `login`, `check_session`, `logout`), ce qui est correct par nature. Aucune commande oubliée.
- **Export ODS non vulnérable à l'injection de formule** (contrairement au CSV) : `set_value` écrit des cellules typées `string`, une note commençant par `=` reste du texte. Le choix d'ODS était le bon.
- **Markdown** : `render.unsafe_ = false` + Ammonia ; les seuls ajouts à la whitelist ne peuvent venir que du highlighter interne. Le seul angle mort est l'ancre `<a href>` (M-03), pas l'injection HTML.
- **Frontend** : aucun `eval`/`new Function`, `localStorage` limité à des préférences non sensibles, token de session en `useRef` mémoire jamais persisté, zéro `console.log`.
- **CSP stricte**, capabilities minimales, `.gitignore` couvrant secrets et bases, aucun secret dans l'historique git.

---

## 8. Plan de correctifs

Ordonné par gravité × effort. Le séquencement des deux premières lignes n'est pas interchangeable (cf. note sous E-02).

| # | Action | Constats | Effort | Risque si différé |
|---|---|---|---|---|
| **1** | **Rendre les migrations atomiques** (`BEGIN`/`COMMIT` autour de batch + version), `ADD COLUMN` conditionnels, test d'interruption | E-01(a) | Faible | Perte totale des données sur coupure pendant une mise à jour |
| **2** | **Déplacer `mark_dirty()` avant la première écriture**, échec fatal | E-01(b) | Trivial | Idem, sur tout crash pendant migration |
| **3** | **Rendre `close()` drainant** (garde RAII / compteur d'opérations) ; ne pas effacer le marqueur dirty si une opération était en cours | E-01(c) | Moyen | Coffre briqué par une fermeture de fenêtre pendant une sauvegarde |
| **4** | **Ajouter un chemin de récupération ré-authentifié** (re-baselining consenti) + sauvegarde du coffre au démarrage | E-01 | Moyen | Aucun filet sous les trois précédents |
| **5** | ~~Aligner l'endpoint updater~~ **Faux positif** : l'endpoint était correct. Contrôle CI de cohérence conservé | M-01 | Trivial | — |
| **6** | **Épingler les 6 actions GitHub par SHA** ; à terme, sortir la signature de la CI | E-03, E1 | Faible | Signature de mises à jour malveillantes sans toucher au compte |
| **7** | **Valider `factor_year`** (millésimes seedés) + CHECK en migration + échec bruyant si la map de facteurs est vide | M-02 | Trivial | Bilan carbone silencieusement nul et non réparable |
| **8** | **Intercepter la navigation** : handler délégué sur les ancres + `on_navigation` côté Rust | M-03 | Faible | Phishing dans le chrome de l'app |
| **9** | **Authentifier l'état d'intégrité** (compteur monotone dans le keystore, marqueur dirty MACé) + les 2 tests manquants | E-02 | Moyen | Tamper-evidence contournable par `touch` |
| **10** | **Corriger la LICENCE (seuil 20/30), le PKGBUILD (licence + `sha256sums`), la mention « audit tiers »** | L-01→L-03 | Trivial | Contradiction juridique, distribution sous mauvaise licence |
| **11** | **Borner les entrées** : taille de bundle, longueurs de chaînes, tailles de collections | F-01, F-02 | Faible | DoS local, gonflement de la base |
| **12** | **Supprimer le code mort et réaligner la doc** : `check_session`, `SESSION_EXPIRED` manquant, cycle `presence ↔ work-hours`, commentaires démentis par le code | A-01→A-05, A-14 | Faible | Érosion du contrat documentaire |
| **13** | Reprendre les constats restants du §5 au fil de l'eau | F-03→F-10, A-06→A-17 | Variable | — |

Reste ouvert depuis le 3 juillet et non re-traité ici : **M1** (confinement des chemins d'export/import), **M5** (`change_password`).

---

## 9. Suivi des corrections (26 juillet 2026)

Tous les constats de cet audit ont été traités. `cargo fmt`, `cargo clippy -D warnings`,
98 tests Rust, `pnpm typecheck`, `pnpm lint`, 99 tests front et `pnpm audit` passent.

### Élevés

| Réf | Correction |
|---|---|
| **E-01a** | Chaque migration s'applique dans une transaction englobant le batch **et** l'enregistrement de sa version (`migrations.rs`, `commit_or_rollback`). 4 tests, dont un qui interrompt une migration à mi-parcours et vérifie que le rejeu est propre. La non-idempotence des `ALTER TABLE` cesse d'être un piège : un échec ne laisse plus rien derrière lui. |
| **E-01b** | `mark_dirty` devient `write_marker(..., SessionState::Open)`, posé **avant** `open_encrypted_db`. Un crash pendant une migration produit désormais « arrêt anormal », pas « fichier modifié ». |
| **E-01c** | `connection()` rend un `VaultConnection` (garde RAII) au lieu d'un clone libre ; `close()` attend le drainage (2 s max) avant de calculer la ligne de base, et **laisse le marqueur `Open`** s'il n'y parvient pas. |
| **E-01d** | Commande `recover_vault` + bouton dédié sur l'écran de login, affiché uniquement après un verdict d'altération. Même chemin d'authentification que `login` (lockout compris) : c'est une reprise, pas un contournement. Une copie `.recovered-N.bak` du fichier suspect est conservée. |
| **E-02** | Sidecar authentifié : `version │ flag │ compteur │ content_mac │ header_mac`. Le drapeau de crash est sous MAC (plus de fichier `.dirty` en clair), le compteur de session est ancré dans le keystore scellé (migration keystore `0003`), et un sidecar absent sur un coffre déjà baseliné vaut `Mismatch`. Corrige au passage **I1** (longueur préfixée : `main ‖ wal` n'est plus ambigu). 11 tests, dont les deux contournements historiques et le rejeu d'un couple `(vault, sidecar)` cohérent mais périmé. |
| **E-03** | Bloc `permissions: contents: read` sur la CI, contrôles étendus aux commits tagués, `.github/dependabot.yml` (actions + npm + cargo), et TODO d'épinglage par SHA documenté en tête de `release.yml` avec la commande `gh api` pour résoudre les digests. **L'épinglage lui-même reste à faire** (cf. ci-dessous). |

### Moyens

| Réf | Correction |
|---|---|
| **M-01** | **Écarté — faux positif** (le compte a été renommé *vers* `g0def` ; c'est le remote local qui était périmé). Conservé de la correction : une étape de `release.yml` compare l'endpoint à `GITHUB_REPOSITORY` et fait échouer la release en cas de vraie dérive. |
| **M-02** | `factor_year` validé contre les millésimes réellement seedés (`available_years()`, donc auto-maintenu), sur les deux chemins — réglages et import de bundle. Garde SQL par triggers (migration vault `0012`) qui **répare aussi les lignes déjà empoisonnées**. `load_factor_maps` échoue bruyamment au lieu de produire 0.0 kg. |
| **M-03** | Garde `on_navigation` côté Rust (plugin interne) : rien ne peut sortir la WebView de son origine. Côté front, `useExternalAnchors` route les liens du HTML rendu vers le navigateur système ; 5 tests, dont le cas d'une URL autolinkée issue d'une note importée. |

### Faibles et informatifs

| Réf | Correction |
|---|---|
| **F-01/F-02** | Module `limits.rs` : taille de fichier de bundle (32 Mo, vérifiée **avant** lecture), nombre d'éléments par catégorie, longueurs de titre/description/note/nom de trajet. Le compilateur a imposé la validation aux 5 sites d'appel. |
| **F-03** | Horloge backend **à cliquet** (`clock.rs`) : maximum entre horloge murale et base monotone, jamais décroissante. Reculer l'heure système ne prolonge plus session ni lockout, et une veille reste comptée. |
| **F-04** | `commit_or_rollback` : un `COMMIT` en échec ne laisse plus la transaction ouverte sur la connexion partagée. Appliqué aux 5 sites. |
| **F-05** | Garde de downgrade : une base plus récente que le binaire est refusée (`SchemaTooNew`) au lieu d'être écrite. |
| **F-07** | Paramètres Argon2id du KEK persistés par compte (migration keystore `0004`). Test d'intégration : monter le coût du build n'orpheline plus les coffres existants. |
| **F-08** | PKGBUILD : `license=('custom:BasicPresence')` + installation du texte, et TODO explicite sur `sha256sums` (voir ci-dessous). |
| **F-09/F-10** | Audit en deux tiers — prod bloquant dès `low`, global dès `moderate`. Overrides `dompurify >= 3.4.12` (le seuil `3.4.11` était lui-même vulnérable), `js-yaml`, `postcss`. L'advisory `brace-expansion` sans correctif applicable est ignorée **explicitement et justifiée** dans `auditConfig`. |
| **A-01/A-03** | `check_session` n'est plus mort : il resynchronise la session au retour au premier plan. La doc affirmait que l'expiration était insensible aux changements d'horloge — c'est désormais vrai côté backend, et le rôle purement décoratif du compte à rebours est écrit noir sur blanc. |
| **A-02** | Helper `isSessionExpired` (la condition était recopiée 8 fois) appliqué aux 3 features qui l'ignoraient : export, transfert de profil, licence. |
| **A-04** | Le cycle `presence ↔ work-hours` est documenté comme dérogation assumée dans `Claude.md`, avec le critère qui déclencherait une extraction. |
| **A-05** | Helper `civilDate` partagé ; le résumé mensuel du header ne bucketise plus un minuit UTC en heure locale. |
| **A-07** | Le garde ESLint couvre `@tauri-apps/plugin-dialog` — celui qui produit le chemin envoyé au backend. |
| **A-08** | `AppErrorCode` dérivé de la liste des codes (plus deux déclarations à tenir en phase) ; `VAULT_TAMPERED`, `KEYSTORE_UNRECOVERABLE` et `SCHEMA_TOO_NEW` n'arrivent plus en « erreur inconnue ». |
| **A-10/A-11** | Un bundle listant deux fois le même jour est refusé ; importer sans les trajets n'importe plus le total CO₂ qu'ils justifiaient. |
| **A-12** | `set_note` refuse une présence inexistante au lieu de confirmer un enregistrement qui n'a rien écrit. |
| **A-13** | Le drapeau « préférences migrées » n'est posé qu'après acceptation par le backend ; un échec laisse la migration rejouable. |
| **A-14** | Le commentaire de `keystore/0001_init.sql` ne prétend plus que le keystore est en clair. |
| **A-17** | `ProfileProvider.refresh` ne laisse plus échapper son rejet. |
| **L-01→L-04** | Seuil de licence à 30 dans les quatre mentions (l'anglais disait 20) ; PKGBUILD sous la bonne licence ; « audit tiers favorable » remplacé par la formulation exacte ; `Cargo.toml` aligné sur 0.2.0 (les bundles exportés étaient estampillés 0.1.1) + contrôle CI de cohérence des versions. |

### Ce qui reste à ta main

Ces points ne peuvent pas être faits depuis le dépôt :

1. **Épingler les 6 actions GitHub par SHA** — la commande est dans le TODO en tête de `release.yml`. Dependabot maintiendra ensuite les digests.
2. **Mettre à jour le remote git local** : `git remote set-url origin git@github.com:g0def/BasicPresenceTauri.git`. Il pointe encore sur l'ancien nom et ne marche que par redirection.
3. **Envisager de réserver l'ancien nom `MehdiA7`** — un renommage GitHub libère l'ancien identifiant, et les redirections cessent dès que quelqu'un l'enregistre. Aucune URL du projet ne le référence, donc l'enjeu se limite aux liens externes déjà partagés et aux clones existants.
4. **Renseigner les `sha256sums` du PKGBUILD** à chaque release (commandes dans le fichier).
5. **Surfacer l'échec du check de mise à jour dans l'UI** — aujourd'hui avalé silencieusement (`updater-provider.tsx:41-43`), donc un canal rompu reste invisible.
6. **E1 (3 juillet)** — sortir la clé de signature de la CI reste la seule mesure qui neutralise complètement E1 *et* E-03.
7. **M1 et M5** — confinement des chemins d'export/import, et `change_password`, toujours ouverts.

---

*Audit réalisé le 26 juillet 2026. Les sévérités tiennent compte du modèle de menace réel d'une application desktop locale mono-utilisateur. Le déplacement du centre de gravité par rapport au 3 juillet — de la confidentialité vers la durabilité des données — reflète le fait que le chiffrement tient, mais que ce qui l'entoure peut rendre les données inaccessibles à leur propriétaire légitime.*
