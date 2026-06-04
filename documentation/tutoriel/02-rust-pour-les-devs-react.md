# 02 — Rust pour les devs React

## Ce que tu vas apprendre

- Comment Rust organise le code en **modules** (`mod`, `use`, `pub`) et en quoi ça diffère des imports ES de TypeScript.
- La notion centrale d'**ownership / borrowing** (les `&`, `&mut`, `String` vs `&str`) — le truc qui n'existe pas en JS.
- Les **structs**, les **enums à données** (comme `DomainError`) et le **pattern matching** (`match`, `if let`, `let ... else`).
- `Result<T, E>` et `Option<T>` : pas d'exceptions, et l'opérateur `?` qui remplace `try/catch`.
- Les **traits** (= interfaces), les **objets-trait** `dyn Trait`, `Arc<dyn Trait>` (= injection de dépendance) et `async_trait`.
- Les **macros** (`#[derive(...)]`, `#[tauri::command]`), le couple **async/await**, l'effacement mémoire (`Zeroizing`), `Mutex` / `Send + Sync`, et les conventions Rust de ce projet.

> 🧭 **Prérequis** : avoir lu [01 — Vue d'ensemble et Tauri](./01-vue-densemble-et-tauri.md). Tu n'as pas besoin de connaître Rust : ce chapitre explique chaque concept la première fois qu'on le croise dans le vrai code du projet.

---

## Pourquoi ce chapitre

Tu maîtrises React/TypeScript, mais tu n'as jamais construit un vrai projet Rust. Ce chapitre n'est **pas** une référence Rust exhaustive. C'est une **boîte à outils ciblée** : uniquement les concepts qu'on rencontre *vraiment* dans la codebase de BasicPresence. Quand tu auras fini, tu pourras lire le code Rust des chapitres suivants sans bloquer.

Pour chaque concept : une **analogie React/TS**, un **extrait réel** du projet, et le **piège** classique du débutant.

---

## 1. Modules et visibilité : `mod`, `use`, `pub`, `crate::`

En TypeScript, un fichier = un module, et tu importes avec des chemins relatifs (`import { x } from './foo'`). En Rust, l'arbre de modules est **déclaré explicitement**. Le fichier racine (`lib.rs`) dit quels sous-modules existent.

[src-tauri/src/lib.rs](../../src-tauri/src/lib.rs)

```rust
mod application;
mod domain;
mod infrastructure;
mod presentation;
```

Chaque `mod xxx;` dit à Rust : « va chercher le module `xxx` » (soit `xxx.rs`, soit `xxx/mod.rs`, soit un dossier `xxx/`). **Si tu ne déclares pas un module, il n'existe pas pour le compilateur**, même si le fichier est sur le disque. C'est le piège n°1 : tu crées `super_truc.rs`, tu l'utilises, et `cargo` te répond « unresolved module » — il manquait un `mod super_truc;`.

Ensuite, `use` rapproche un chemin pour ne pas le réécrire en entier (comme `import` en TS) :

[src-tauri/src/application/use_cases/register_account.rs](../../src-tauri/src/application/use_cases/register_account.rs)

```rust
use std::sync::Arc;

use uuid::Uuid;
use zeroize::Zeroizing;

use crate::application::dto::user_dto::UserDto;
use crate::domain::entities::account::{Account, KeyMaterial};
use crate::domain::error::DomainError;
```

À décoder :

- `std::` = la bibliothèque standard (toujours dispo, comme les globals JS `Array`, `Map`…).
- `uuid::`, `zeroize::` = des **crates** externes (des packages, comme une dépendance npm). Elles sont déclarées dans `Cargo.toml` (l'équivalent du `package.json`).
- `crate::` = la racine de **notre** projet. Donc `crate::domain::error::DomainError` ≈ `@/domain/error` avec ton alias `@/`. Le `::` est juste le séparateur de chemin (comme le `/` ou le `.`).
- Les accolades `{Account, KeyMaterial}` = import groupé, exactement comme `import { Account, KeyMaterial }`.

**Visibilité avec `pub`.** Par défaut, tout en Rust est **privé au module**. Pour qu'un élément soit visible de l'extérieur, tu le marques `pub` (≈ `export`). Regarde l'entité `Profile` :

[src-tauri/src/domain/entities/profile.rs](../../src-tauri/src/domain/entities/profile.rs)

```rust
pub struct Profile {
    pub id: String,
    pub first_name: String,
    pub last_name: String,
    pub enterprise: String,
    pub poste: Option<String>,
    pub created_at: i64,
    pub updated_at: i64,
}
```

`pub struct Profile` = le type est exporté. **Et chaque champ a aussi son `pub`** : un champ sans `pub` resterait privé même si la struct est publique. En TS, `export interface Profile { id: string }` exporte tout d'un coup ; en Rust, tu choisis champ par champ. C'est plus verbeux mais plus contrôlé.

---

## 2. Ownership, borrowing, et `String` vs `&str`

C'est LE concept qui n'a pas d'équivalent en JS/TS. En JavaScript, le garbage collector (GC) nettoie la mémoire pour toi : tu passes des références partout sans y penser. Rust **n'a pas de GC**. À la place, il a une règle simple appliquée à la compilation :

> Chaque valeur a **un seul propriétaire** (owner). Quand le propriétaire sort du scope (`}`), la valeur est libérée automatiquement.

Pour utiliser une valeur sans en prendre la propriété, on l'**emprunte** (borrow) avec `&` (référence). Deux formes :

- `&T` = emprunt **en lecture seule** (immuable). Tu peux en avoir autant que tu veux en même temps.
- `&mut T` = emprunt **en écriture** (mutable). Un seul à la fois, et aucun `&` en lecture pendant ce temps.

Regarde le contrat du repository de comptes :

[src-tauri/src/domain/repositories/account_repository.rs](../../src-tauri/src/domain/repositories/account_repository.rs)

```rust
async fn find_by_username(&self, username: &str) -> Result<Option<Account>, DomainError>;

async fn create(&self, account: &Account) -> Result<(), DomainError>;
```

- `&self` = la méthode **emprunte** l'objet sans le consommer (comme `this` en lecture, c'est l'équivalent d'une méthode qui ne te « vole » pas l'instance).
- `account: &Account` = on **emprunte** le compte le temps de l'écrire en DB. L'appelant le garde ensuite.

### `String` vs `&str`, `Vec<u8>` vs `&[u8]`

Ce sont des paires qu'on croise partout :

- `String` = une chaîne **possédée**, qui vit sur le tas, redimensionnable. ≈ une `string` que tu détiens et peux modifier.
- `&str` = une **vue empruntée** sur une chaîne (un « slice »). ≈ « je veux juste lire ta chaîne, je ne la copie pas ».
- `Vec<u8>` = un tableau d'octets **possédé** (≈ `Uint8Array` que tu détiens).
- `&[u8]` = une **vue empruntée** sur des octets.

On voit les deux côtes à côte dans le `KeyService` (chiffrement) :

[src-tauri/src/domain/services/key_service.rs](../../src-tauri/src/domain/services/key_service.rs)

```rust
pub struct WrappedKey {
    pub ciphertext: Vec<u8>,
    pub nonce: Vec<u8>,
}

pub trait KeyService: Send + Sync {
    fn derive_kek(&self, password: &[u8], salt: &[u8]) -> Result<Zeroizing<[u8; 32]>, DomainError>;
    fn wrap_dek(&self, dek: &[u8], kek: &[u8]) -> Result<WrappedKey, DomainError>;
}
```

- `WrappedKey` **possède** ses octets (`Vec<u8>`) : c'est une donnée qu'on stocke et qu'on retourne.
- `wrap_dek(&self, dek: &[u8], kek: &[u8])` ne fait qu'**emprunter** la DEK et la KEK le temps de chiffrer : il n'en prend pas la propriété, donc l'appelant peut continuer à les utiliser (et à les effacer ensuite).
- `[u8; 32]` = un tableau de **taille fixe** 32 octets (≠ `Vec<u8>` qui est redimensionnable). Ici, 32 octets, c'est exactement la taille d'une clé 256 bits (celle de XChaCha20-Poly1305, le chiffrement utilisé par ce projet).

> ⚠️ **Le piège « value moved »** : quand tu passes une valeur possédée (sans `&`), tu en transfères la propriété ; tu ne peux plus l'utiliser après. C'est pour ça que tu verras des `.clone()` (copier la donnée) ou des `&` (emprunter) un peu partout. Si `cargo` te dit *« use of moved value »*, c'est que tu as donné une valeur puis voulu la réutiliser.

On voit justement des `.clone()` de chaînes dans `Account::to_user` :

[src-tauri/src/domain/entities/account.rs](../../src-tauri/src/domain/entities/account.rs)

```rust
impl Account {
    pub fn to_user(&self) -> User {
        User {
            id: self.id.clone(),
            username: self.username.clone(),
            created_at: self.created_at,
            updated_at: self.updated_at,
        }
    }
}
```

`self.id` est une `String` possédée par le compte. Comme `to_user` n'emprunte le compte qu'en `&self`, il **ne peut pas voler** `id` ; il en fait donc une copie avec `.clone()`. À l'inverse, `created_at` est un `i64` (un entier 64 bits) : les types numériques sont `Copy` (copiés automatiquement, sans `.clone()`), donc pas de souci.

> Note : `impl Account { ... }` = un **bloc d'implémentation**. C'est là qu'on attache des méthodes à une struct. En TS, méthodes et champs sont dans la même `class {}` ; en Rust, on sépare la *donnée* (`struct`) de son *comportement* (`impl`).

---

## 3. Structs, enums à données, et pattern matching

### Structs

Une `struct` = un objet de données typé. Tu l'as déjà vue (`Profile`, `Account`, `WrappedKey`). C'est l'équivalent d'une `interface`/`type` TS mais c'est aussi un vrai constructeur. On instancie en nommant chaque champ :

[src-tauri/src/application/use_cases/register_account.rs](../../src-tauri/src/application/use_cases/register_account.rs)

```rust
let account = Account {
    id: Uuid::now_v7().to_string(),
    username: username.to_string(),
    password_hash,
    key_material: KeyMaterial {
        wrapped_dek: wrapped.ciphertext,
        kek_salt,
        dek_nonce: wrapped.nonce,
    },
    failed_attempts: 0,
    locked_until: None,
    created_at: now,
    updated_at: now,
};
```

Deux raccourcis sympas et identiques à TS : `password_hash` tout court = `password_hash: password_hash` (raccourci de champ, comme l'object shorthand ES). Et `let ... = ...;` introduit une variable locale (immuable par défaut — il faut `let mut` pour pouvoir la réassigner).

### Enums à données : `DomainError`

En TypeScript, un `enum` ne contient que des constantes. En Rust, un `enum` est **bien plus puissant** : chaque variante peut transporter ses **propres données**. C'est l'équivalent exact d'une **union discriminée** TS (`type T = { kind: 'a', x: number } | { kind: 'b' }`). C'est le cœur du système d'erreurs :

[src-tauri/src/domain/error.rs](../../src-tauri/src/domain/error.rs)

```rust
#[derive(Debug, Error)]
pub enum DomainError {
    #[error("invalid credentials")]
    InvalidCredentials,

    #[error("account locked")]
    AccountLocked { retry_after_ms: i64 },

    #[error("validation error: {0}")]
    Validation(String),

    #[error("crypto error: {0}")]
    Crypto(String),
    // ... AccountAlreadyExists, Unauthorized, ProfileNotFound, Storage, Hashing, Token
}
```

Trois formes de variantes :

- `InvalidCredentials` = variante simple, sans donnée (≈ `{ kind: 'invalid_credentials' }`).
- `AccountLocked { retry_after_ms: i64 }` = variante avec des champs nommés (≈ `{ kind: 'account_locked', retryAfterMs: number }`).
- `Validation(String)` = variante « tuple » qui transporte une `String` anonyme (le `{0}` dans le message la réinjecte).

> `#[derive(Debug, Error)]` et `#[error("...")]` viennent de la crate `thiserror` : on en parle en section 7. Retiens juste que ça génère automatiquement le message d'erreur et l'affichage de debug.

### Pattern matching : `match`

Pour réagir à un enum, on utilise `match`. C'est comme un `switch` TS, mais **exhaustif** : le compilateur t'oblige à traiter *tous* les cas (sinon erreur de compilation — adieu les oublis). On le voit pleinement dans la conversion d'erreur côté présentation :

[src-tauri/src/presentation/commands/error.rs](../../src-tauri/src/presentation/commands/error.rs)

```rust
impl From<DomainError> for AppError {
    fn from(e: DomainError) -> Self {
        match e {
            DomainError::InvalidCredentials => {
                AppError::new("INVALID_CREDENTIALS", "Identifiants invalides")
            }
            DomainError::AccountLocked { retry_after_ms } => AppError::new(
                "ACCOUNT_LOCKED",
                format!(
                    "Compte verrouillé. Réessayez dans {} s.",
                    (retry_after_ms / 1000).max(1)
                ),
            ),
            // ... AccountAlreadyExists, Unauthorized, ProfileNotFound (chacun mappé sur un code dédié)
            DomainError::Validation(m) => AppError::new("VALIDATION", m),
            DomainError::Storage(_)
            | DomainError::Hashing(_)
            | DomainError::Crypto(_)
            | DomainError::Token(_) => AppError::new("INTERNAL", "Erreur interne"),
        }
    }
}
```

À remarquer :

- `DomainError::AccountLocked { retry_after_ms }` **déstructure** la variante : on récupère directement le champ `retry_after_ms` pour calculer le message (≈ `case 'account_locked': const { retryAfterMs } = e`).
- `DomainError::Validation(m)` capture la `String` interne dans `m`.
- `Storage(_) | Hashing(_) | Crypto(_) | Token(_) =>` regroupe quatre variantes dans une seule branche. Le `_` veut dire « il y a une donnée ici mais je l'ignore ». C'est exactement la règle de sécurité : tous les détails bas niveau (SQL, crypto) sont **écrasés** en `"INTERNAL"` et ne fuitent jamais vers la WebView.

> `impl From<DomainError> for AppError` = « comment fabriquer un `AppError` à partir d'un `DomainError` ». Ce trait `From` est ce qui rend l'opérateur `?` magique (section 4).

### `if let` et `let ... else`

Quand tu ne veux gérer **qu'un seul cas**, `match` complet est trop verbeux. Deux raccourcis.

`if let` = « si ça correspond à ce motif, fais quelque chose ». On le voit pour le verrouillage de compte :

[src-tauri/src/application/use_cases/login.rs](../../src-tauri/src/application/use_cases/login.rs)

```rust
if let Some(locked_until) = account.locked_until {
    if locked_until > now {
        return Err(DomainError::AccountLocked {
            retry_after_ms: locked_until - now,
        });
    }
}
```

`account.locked_until` est un `Option<i64>` (présent ou absent — section 4). `if let Some(locked_until) = ...` dit : « **si** il y a une valeur, mets-la dans `locked_until` et exécute le bloc ; sinon, ne fais rien ». ≈ `if (account.lockedUntil != null) { const lockedUntil = account.lockedUntil; ... }`.

`let ... else` est le miroir : « extrais la valeur, **sinon** sors tout de suite ». C'est l'idiome d'early-return le plus important du projet, juste au début du login :

[src-tauri/src/application/use_cases/login.rs](../../src-tauri/src/application/use_cases/login.rs)

```rust
let Some(account) = self.accounts.find_by_username(username).await? else {
    // Equalize timing against username enumeration (hash, discard).
    let _ = self.hasher.hash(pw.as_slice());
    return Err(DomainError::InvalidCredentials);
};
```

À lire : « **si** `find_by_username` renvoie `Some(account)`, alors la variable `account` existe pour la suite de la fonction ; **sinon** (`else`), on entre dans le bloc qui *doit* sortir de la fonction (`return`) ». En TS naïf : `const account = ...; if (account == null) { ...; return; }` — sauf qu'ici, après le `let ... else`, `account` est garanti non-nul par le compilateur. Le `let _ = ...` veut dire « exécute mais jette le résultat » : on hache quand même un mot de passe pour égaliser le timing (anti-énumération de username), puis on l'oublie.

---

## 4. `Result`, `Option`, et l'opérateur `?`

**Rust n'a pas d'exceptions.** Pas de `throw`, pas de `try/catch`. Une fonction qui peut échouer **retourne** son échec dans son type de retour. Deux types pour ça :

- `Option<T>` = `Some(valeur)` ou `None`. ≈ `T | null | undefined`, mais explicite et impossible à oublier. (On l'a vu : `find_by_username` renvoie `Option<Account>` = un compte trouvé ou rien.)
- `Result<T, E>` = `Ok(valeur)` ou `Err(erreur)`. ≈ « soit ça réussit avec un `T`, soit ça échoue avec un `E` ». C'est le remplaçant du `try/catch`.

Tout le domaine renvoie `Result<_, DomainError>`. Exemple de validation :

[src-tauri/src/application/use_cases/register_account.rs](../../src-tauri/src/application/use_cases/register_account.rs)

```rust
fn validate_username(username: &str) -> Result<(), DomainError> {
    let len = username.chars().count();
    if !(MIN_USERNAME_LEN..=MAX_USERNAME_LEN).contains(&len) {
        return Err(DomainError::Validation(format!(
            "username must be {MIN_USERNAME_LEN}-{MAX_USERNAME_LEN} characters"
        )));
    }
    Ok(())
}
```

`Result<(), DomainError>` : le `()` (« unit ») = « pas de valeur utile en cas de succès », l'équivalent d'une fonction `void` qui peut quand même échouer. En cas de problème : `return Err(...)`. À la fin, `Ok(())` = « tout va bien, rien à renvoyer ». **Le compilateur exige que la fonction retourne un `Result` sur tous les chemins** : tu ne peux pas « oublier » de gérer le cas d'échec.

### L'opérateur `?` = early-return de l'erreur

Réécrire un `match` complet à chaque appel faillible serait insupportable. D'où l'opérateur **`?`** : posé après une expression qui renvoie un `Result` (ou un `Option`), il fait :

- si c'est `Ok(v)` → continue avec `v` ;
- si c'est `Err(e)` → **sort immédiatement** de la fonction en renvoyant `Err(e)`.

C'est l'équivalent d'un `await` qui propagerait automatiquement le `throw` vers le haut. On le voit en cascade dans `register` :

[src-tauri/src/application/use_cases/register_account.rs](../../src-tauri/src/application/use_cases/register_account.rs)

```rust
let username = username.trim();
validate_username(username)?;
validate_password(password)?;

// Single-owner device: refuse a second account.
if self.accounts.exists().await? {
    return Err(DomainError::AccountAlreadyExists);
}

let password_hash = self.hasher.hash(pw.as_slice())?;

let dek = self.keys.generate_dek()?;
let kek_salt = self.keys.generate_salt()?;
let kek = self.keys.derive_kek(pw.as_slice(), &kek_salt)?;
let wrapped = self.keys.wrap_dek(dek.as_slice(), kek.as_slice())?;
```

Chaque `?` veut dire : « si cette étape échoue, abandonne tout de suite et remonte l'erreur ». Cinq lignes lisibles au lieu de cinq `match` imbriqués. Mieux : `self.accounts.exists().await?` combine `.await` (attend la fin de l'opération async) **et** `?` (propage l'erreur). Et grâce à l'`impl From<...>` qu'on a vu, `?` peut même convertir un type d'erreur en un autre au passage.

À la fin, on emballe le succès :

```rust
Ok(UserDto::from(account.to_user()))
```

C'est le dernier `Ok(...)` qui signe la réussite. Tu retrouves ce motif à la frontière IPC, dans chaque commande Tauri :

[src-tauri/src/presentation/commands/auth.rs](../../src-tauri/src/presentation/commands/auth.rs)

```rust
#[tauri::command]
pub async fn register(
    username: String,
    password: String,
    state: State<'_, AppState>,
) -> Result<UserDto, AppError> {
    Ok(state.register_account.execute(&username, &password).await?)
}
```

`...execute(...).await?` exécute le use case ; si erreur, `?` la propage (et la convertit en `AppError` via le `From` vu plus haut) ; sinon on emballe le `UserDto` dans `Ok(...)`. Le résultat sera sérialisé puis renvoyé au front (chapitre 06).

> ⚠️ **`?` n'a rien à voir avec le `?` optionnel de TS** (`obj?.prop`). En Rust c'est de la propagation d'erreur. À ne pas confondre.

---

## 5. Traits = interfaces

Un **trait** Rust ≈ une **interface** TypeScript : un contrat de méthodes, sans implémentation. C'est la clé de la Clean Architecture (chapitre 03) : le domaine définit des traits (« ports »), et l'infrastructure les implémente.

[src-tauri/src/domain/services/password_hasher.rs](../../src-tauri/src/domain/services/password_hasher.rs)

```rust
pub trait PasswordHasher: Send + Sync {
    fn hash(&self, password: &[u8]) -> Result<String, DomainError>;
    fn verify(&self, password: &[u8], phc: &str) -> Result<bool, DomainError>;
}
```

≈ `interface PasswordHasher { hash(password: Uint8Array): string; verify(...): boolean }`. Le `: Send + Sync` après le nom = des contraintes « thread-safe » (section 9). Pour l'instant, lis-les comme du bruit obligatoire.

### `impl Trait for Struct` = la classe qui implémente l'interface

On *implémente* un trait pour une struct concrète. Le service de chiffrement réel :

[src-tauri/src/infrastructure/crypto/key_service.rs](../../src-tauri/src/infrastructure/crypto/key_service.rs)

```rust
pub struct Argon2KeyService {
    params: Params,
}

impl KeyService for Argon2KeyService {
    fn derive_kek(&self, password: &[u8], salt: &[u8]) -> Result<Zeroizing<[u8; 32]>, DomainError> {
        let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, self.params.clone());
        let mut out = Zeroizing::new([0u8; 32]);
        argon
            .hash_password_into(password, salt, out.as_mut_slice())
            .map_err(|e| DomainError::Crypto(e.to_string()))?;
        Ok(out)
    }
    // ... generate_dek, wrap_dek, unwrap_dek
}
```

`impl KeyService for Argon2KeyService` = « `Argon2KeyService` est une implémentation de `KeyService` » ≈ `class Argon2KeyService implements KeyService`. Deux nouveautés à noter :

- `let mut out` : `mut` rend la variable **mutable** (par défaut tout est figé). Indispensable ici car Argon2 *écrit* le résultat dans le buffer `out` via `out.as_mut_slice()` (un emprunt `&mut`).
- `.map_err(|e| DomainError::Crypto(e.to_string()))?` : transforme l'erreur technique d'Argon2 **en une `DomainError`** avant de la propager avec `?`. Le `|e| ...` est une **closure** (fonction fléchée ≈ `(e) => ...`). C'est ainsi que le domaine reste ignorant des détails de la crate crypto.

### Objets-trait : `dyn Trait` et `Arc<dyn Trait>`

En TS, tu déclares `private hasher: PasswordHasher` et tu injectes n'importe quelle implémentation. En Rust, la version « interface comme type » s'écrit `dyn Trait` (« dynamique » : la méthode appelée est résolue à l'exécution, comme le polymorphisme objet classique).

Mais un `dyn Trait` n'a pas de taille connue, donc on le met **derrière un pointeur**. Ici, c'est `Arc<dyn Trait>`. Regarde les dépendances du use case de login :

[src-tauri/src/application/use_cases/login.rs](../../src-tauri/src/application/use_cases/login.rs)

```rust
pub struct LoginUseCase {
    accounts: Arc<dyn AccountRepository>,
    hasher: Arc<dyn PasswordHasher>,
    keys: Arc<dyn KeyService>,
    tokens: Arc<dyn TokenGenerator>,
    sessions: Arc<dyn SessionStore>,
    vault: Arc<dyn VaultManager>,
    clock: Arc<dyn Clock>,
    policy: AuthPolicy,
}
```

Lis chaque ligne comme : « `LoginUseCase` dépend d'**un** truc qui respecte l'interface `AccountRepository`, peu importe lequel ». **C'est exactement de l'injection de dépendances typée par interface**, comme tu le ferais dans un constructeur React/TS. Le use case ne connaît jamais l'implémentation concrète (libSQL, Argon2…) : il ne voit que le trait.

`Arc` = *Atomically Reference Counted*, un pointeur partagé à comptage de références **thread-safe**. Quelques points :

- Plusieurs propriétaires peuvent partager la même donnée (le use case login ET le use case logout pointent vers la même session, par exemple).
- `arc.clone()` ne copie **pas** la donnée : il incrémente juste le compteur de références et te rend un deuxième pointeur vers le **même** objet. C'est très bon marché. ≈ partager la même instance d'un service entre plusieurs composants.
- Quand le dernier `Arc` disparaît, la donnée est libérée.

C'est pour ça que le composition root clone des `Arc` à tour de bras pour câbler tout le monde (section 6).

### `async_trait` : pourquoi ce `#[async_trait]` ?

Tu remarqueras un attribut bizarre au-dessus de certains traits :

[src-tauri/src/domain/repositories/account_repository.rs](../../src-tauri/src/domain/repositories/account_repository.rs)

```rust
use async_trait::async_trait;

#[async_trait]
pub trait AccountRepository: Send + Sync {
    async fn exists(&self) -> Result<bool, DomainError>;
    async fn find_by_username(&self, username: &str) -> Result<Option<Account>, DomainError>;
}
```

Raison : à l'époque de cette base de code, Rust ne supporte pas nativement les **méthodes `async` dans un trait avec objets-trait** (`dyn`). La macro `#[async_trait]` (de la crate `async-trait`) réécrit ces méthodes pour que ça marche quand même. Tu n'as rien à faire de plus : **dès qu'un trait a une méthode `async` et qu'on veut l'utiliser en `Arc<dyn Trait>`, on colle `#[async_trait]` au-dessus du `trait` ET au-dessus du `impl ... for ...` correspondant**. C'est tout. (À l'inverse, `KeyService` et `PasswordHasher` n'en ont pas : leurs méthodes sont **synchrones**, le calcul crypto bloque le CPU mais ne fait pas d'I/O.)

---

## 6. Le composition root : tout se câble dans `lib.rs`

On a maintenant tous les morceaux pour lire le point d'assemblage. En React tu mettrais tes Providers/instances de services à la racine ; ici, c'est `build_state` :

[src-tauri/src/lib.rs](../../src-tauri/src/lib.rs)

```rust
let accounts: Arc<dyn AccountRepository> =
    Arc::new(LibsqlAccountRepository::new(keystore_conn));
let hasher: Arc<dyn PasswordHasher> = Arc::new(Argon2PasswordHasher::new(
    config.argon2.m_cost,
    config.argon2.t_cost,
    config.argon2.p_cost,
)?);
let keys: Arc<dyn KeyService> = Arc::new(Argon2KeyService::new(
    config.argon2.m_cost,
    config.argon2.t_cost,
    config.argon2.p_cost,
)?);
```

`Arc::new(LibsqlAccountRepository::new(...))` = « crée l'implémentation concrète, emballe-la dans un `Arc`, et range-la dans une variable typée `Arc<dyn AccountRepository>` ». L'annotation de type explicite **efface** le type concret : à partir de là, tout le monde ne voit plus que le trait. C'est l'inversion de dépendance en action.

Puis on injecte ces `Arc` (clonés) dans les use cases :

[src-tauri/src/lib.rs](../../src-tauri/src/lib.rs)

```rust
login: LoginUseCase::new(
    accounts.clone(),
    hasher.clone(),
    keys.clone(),
    tokens.clone(),
    sessions.clone(),
    vault.clone(),
    clock.clone(),
    config.auth.clone(),
),
```

Chaque `.clone()` ici = **cloner le pointeur `Arc`, pas le service**. Plusieurs use cases partagent ainsi la *même* instance de hasher, de sessions, etc. — exactement comme plusieurs composants React consommeraient le même contexte. (Le détail complet de l'architecture est dans [03 — Clean Architecture](./03-clean-architecture.md) et [04 — Backend Rust couche par couche](./04-backend-rust-couche-par-couche.md).)

---

## 7. Macros : `#[derive(...)]`, `#[tauri::command]`, `#[error(...)]`

Les trucs entre `#[...]` sont des **attributs** : de la génération de code à la compilation. Vu de React, pense aux **décorateurs** ou à du codegen automatique. Deux familles dans ce projet.

### `#[derive(...)]` : génère du boilerplate pour toi

[src-tauri/src/application/dto/user_dto.rs](../../src-tauri/src/application/dto/user_dto.rs)

```rust
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct UserDto {
    pub id: String,
    pub username: String,
    pub created_at: i64,
    pub updated_at: i64,
}
```

`#[derive(...)]` demande au compilateur d'**implémenter automatiquement** certains traits :

- `Debug` → permet d'afficher la struct pour le debug (`println!("{:?}", x)`), comme un `JSON.stringify` de mise au point.
- `Clone` → ajoute la méthode `.clone()` pour copier la valeur.
- `Serialize` (de `serde`) → rend la struct **convertible en JSON**. C'est ça qui permet de renvoyer un `UserDto` au front.
- `#[serde(rename_all = "camelCase")]` → à la sérialisation, `created_at` (snake_case Rust) devient `createdAt` (camelCase JS). Très pratique : le backend reste idiomatique Rust, le front reçoit du JSON idiomatique JS.

Côté erreurs, `thiserror` fonctionne pareil : `#[derive(Error)]` + `#[error("...")]` génèrent l'implémentation du trait standard `Error` et le message lisible (revois `DomainError` en section 3).

### `#[tauri::command]` : expose une fonction Rust au front

[src-tauri/src/presentation/commands/auth.rs](../../src-tauri/src/presentation/commands/auth.rs)

```rust
#[tauri::command]
pub async fn login(
    username: String,
    password: String,
    state: State<'_, AppState>,
) -> Result<LoginResultDto, AppError> {
    Ok(state.login.execute(&username, &password).await?)
}
```

`#[tauri::command]` transforme cette fonction en une **commande invocable depuis le JavaScript** via `invoke("login", { username, password })`. La macro génère toute la « tuyauterie » : désérialiser les arguments JSON entrants, injecter l'état partagé (`State<'_, AppState>`, qu'on n'envoie *pas* depuis le front — Tauri le fournit), exécuter, puis sérialiser le `Result` de retour. Le détail du pont IPC est le sujet du [chapitre 06](./06-le-pont-ipc.md).

### `#[allow(...)]` : faire taire un avertissement

Tu croiseras parfois ça :

[src-tauri/src/application/use_cases/login.rs](../../src-tauri/src/application/use_cases/login.rs)

```rust
#[allow(clippy::too_many_arguments)]
pub fn new(
    accounts: Arc<dyn AccountRepository>,
    // ... 8 paramètres au total
) -> Self {
```

`clippy` est le linter de Rust (≈ ESLint). Il râle quand une fonction a trop de paramètres. Ici, c'est **assumé** (un composition root a beaucoup de dépendances), donc `#[allow(clippy::too_many_arguments)]` désactive cette règle **pour cette fonction uniquement** ≈ `// eslint-disable-next-line ...`.

---

## 8. async / await et le runtime

Comme en JS, une fonction `async` ne renvoie pas directement sa valeur : elle renvoie un **`Future`** (≈ une `Promise`). Mais il y a une différence cruciale :

> En JS, une `Promise` commence à s'exécuter dès sa création. En Rust, **un `Future` ne fait STRICTEMENT RIEN tant qu'il n'est pas « poll » — c'est-à-dire `.await`-é** (ou confié à un runtime). Oublier le `.await`, c'est créer un travail qui ne s'exécute jamais.

D'où le `.await` partout où on appelle une méthode async, par exemple `self.accounts.exists().await?` vu plus haut. Le `?` qui suit s'applique au `Result` une fois le `Future` résolu.

Et qui « poll » ces futures ? Un **runtime** asynchrone (l'équivalent de l'event loop de Node). En contexte Tauri, le runtime est fourni par le framework. On le rencontre une seule fois, au démarrage, pour franchir la frontière entre code **synchrone** et code **async** :

[src-tauri/src/lib.rs](../../src-tauri/src/lib.rs)

```rust
let config = AppConfig::new(&data_dir);
let state = tauri::async_runtime::block_on(build_state(config))
    .map_err(|e| format!("failed to initialize backend: {e:?}"))?;
app.manage(state);
```

`build_state` est `async`, mais le `setup` de Tauri est synchrone. `tauri::async_runtime::block_on(...)` veut dire : « lance ce `Future` et **bloque ce thread** jusqu'à ce qu'il finisse, puis donne-moi le résultat ». ≈ un `await` au tout premier niveau, là où tu ne peux pas écrire `await`. Acceptable ici car c'est l'initialisation, une fois, au lancement. `app.manage(state)` range ensuite l'`AppState` pour que les commandes Tauri y accèdent via `State<'_, AppState>`.

> ⚠️ **N'utilise jamais `block_on` à l'intérieur d'un contexte déjà async** (par exemple dans une commande `#[tauri::command] async fn`) : tu bloquerais le runtime et risquerais un *deadlock*. Dans le code async, on chaîne avec `.await`. `block_on` est réservé au pont synchrone → async du démarrage.

---

## 9. Effacer les secrets de la mémoire : `Zeroizing` / `zeroize`

Voilà un concept **sans équivalent en JS** : en JavaScript, tu ne contrôles pas quand la RAM d'une chaîne est libérée ni si elle est remise à zéro (c'est l'affaire du GC). Pour une app « niveau bancaire », laisser traîner un mot de passe ou une clé en clair dans la mémoire est un risque. Rust permet de l'**effacer activement** dès qu'on n'en a plus besoin, via la crate `zeroize`.

Premier outil, `Zeroizing<T>` : un emballage qui **écrase la mémoire de `T` avec des zéros** automatiquement quand la variable sort du scope. On le voit dès la première manipulation du mot de passe :

[src-tauri/src/application/use_cases/login.rs](../../src-tauri/src/application/use_cases/login.rs)

```rust
let pw = Zeroizing::new(password.as_bytes().to_vec());
```

`pw` se comporte comme un `Vec<u8>` ordinaire (on l'utilise via `pw.as_slice()`), mais à la fin de la fonction `execute`, ses octets sont écrasés — quel que soit le chemin de sortie, succès ou `?` qui propage une erreur. Idem pour les clés : `derive_kek` et `generate_dek` renvoient des `Zeroizing<[u8; 32]>` (revois le `KeyService`), donc KEK et DEK sont nettoyées dès qu'on a fini de s'en servir.

Second outil, la méthode `.zeroize()` qu'on appelle **à la main** sur un buffer temporaire, par exemple dans `unwrap_dek` :

[src-tauri/src/infrastructure/crypto/key_service.rs](../../src-tauri/src/infrastructure/crypto/key_service.rs)

```rust
let mut dek = Zeroizing::new([0u8; 32]);
dek.as_mut_slice().copy_from_slice(&plaintext);
plaintext.zeroize();
Ok(dek)
```

On copie le clair (`plaintext`) dans une cible auto-effaçante (`dek`), puis on **efface immédiatement** le buffer source `plaintext` avec `.zeroize()` avant de retourner. Le secret ne survit nulle part en clair plus longtemps que nécessaire. (Le détail du modèle de sécurité est dans [05 — Sécurité et chiffrement](./05-securite-et-chiffrement.md).)

---

## 10. `Mutex`, `Send + Sync` : la concurrence, démystifiée en deux minutes

Tu as vu `Send + Sync` accroché à tous les traits, et un `Mutex` dans le store de sessions. Pas besoin d'être expert pour lire le code, juste de savoir ce que ça veut dire.

- **`Send`** = « ce type peut être *envoyé* à un autre thread ». **`Sync`** = « ce type peut être *partagé* (référencé) depuis plusieurs threads en même temps ». Quand un trait exige `: Send + Sync`, ça garantit que ses implémentations sont sûres à manipuler en multi-thread. Comme les commandes Tauri peuvent tourner sur plusieurs threads, tout l'état partagé (les `Arc<dyn ...>`) doit respecter ça. En pratique, c'est le compilateur qui vérifie : tu ne *fais* rien de spécial, tu te contentes d'écrire `: Send + Sync` sur tes traits.

- **`Mutex<T>`** = un verrou : « un seul thread à la fois peut accéder à la donnée ». C'est nécessaire pour muter une donnée partagée sans corruption. Le store de sessions en mémoire l'utilise :

[src-tauri/src/infrastructure/session/in_memory_session_store.rs](../../src-tauri/src/infrastructure/session/in_memory_session_store.rs)

```rust
pub struct InMemorySessionStore {
    sessions: Mutex<HashMap<String, Session>>,
}

impl SessionStore for InMemorySessionStore {
    fn insert(&self, session: Session) {
        self.sessions
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(session.token.clone(), session);
    }
}
```

`Mutex<HashMap<...>>` = une table (≈ `Map<string, Session>`) protégée par un verrou. `.lock()` prend le verrou (les autres threads attendent), puis on `.insert(...)`. Le verrou se relâche tout seul à la fin de l'expression. Détail subtil : `.unwrap_or_else(|p| p.into_inner())` récupère proprement l'accès même si un *autre* thread avait paniqué en tenant le verrou (un mutex « empoisonné ») — on choisit ici de continuer plutôt que de planter. Tu peux survoler ce détail au premier passage.

> Note : c'est en mémoire **volontairement**. Les sessions disparaissent à la fermeture de l'app, ce qui force une reconnexion au prochain démarrage — exactement le comportement voulu (TTL absolu 15 min, jamais persisté).

---

## 11. Les conventions Rust de ce projet

Pour finir, le « style maison » que tu retrouveras partout :

- **`snake_case`** pour les variables, fonctions, champs et modules (`failed_attempts`, `find_by_username`, `register_account`). Les **types** (structs, enums, traits) sont en `PascalCase` (`LoginUseCase`, `DomainError`). Les **constantes** en `SCREAMING_SNAKE_CASE` (`MIN_PASSWORD_LEN`).
- **Pas d'exceptions** : toute fonction faillible retourne un `Result<_, DomainError>`, et on propage avec `?`. Le domaine ne connaît qu'un seul type d'erreur métier, `DomainError`.
- **Pas de `unwrap()` / `panic!` en production.** `.unwrap()` sur un `Result`/`Option` fait **planter le programme** si c'est `Err`/`None` (≈ `throw` non rattrapé). Tu en verras *uniquement* dans les tests, où un panic = un test rouge, ce qui est acceptable :

[src-tauri/src/infrastructure/crypto/key_service.rs](../../src-tauri/src/infrastructure/crypto/key_service.rs)

```rust
#[cfg(test)]
mod tests {
    #[test]
    fn wrap_then_unwrap_recovers_dek() {
        let ks = service();
        let dek = ks.generate_dek().unwrap();
        // ...
        assert_eq!(dek.as_slice(), unwrapped.as_slice());
    }
}
```

`#[cfg(test)]` = « ne compile ce module **que** lors des tests » (le code de test n'embarque pas dans le binaire final). `#[test]` marque une fonction de test (≈ un `it(...)` Vitest). Ici les `.unwrap()` sont OK : on *veut* que le test échoue bruyamment si une étape rate.

- **`thiserror`** pour définir les erreurs (les `#[error("...")]` vus en section 3), et un `impl From<DomainError> for AppError` à la frontière pour mapper l'erreur interne vers une forme sûre côté front.

---

## En résumé

| Concept Rust | Équivalent React/TS mental |
|---|---|
| `mod` / `use` / `pub` / `crate::` | déclaration de module + `import` + `export`, `crate::` ≈ alias `@/` |
| `&T` / `&mut T`, `String`/`&str` | référence empruntée vs valeur possédée (pas de GC) |
| `enum` à données + `match` | union discriminée + `switch` exhaustif |
| `let ... else` | extraire ou early-return |
| `Result<T,E>` + `?` | pas de `try/catch` ; `?` ≈ `await` qui propage le throw |
| `trait` / `impl ... for` | `interface` / `implements` |
| `Arc<dyn Trait>` | dépendance injectée typée par une interface (instance partagée) |
| `#[derive(...)]`, `#[tauri::command]` | décorateurs / codegen |
| `async` → `Future`, `.await` | `async` → `Promise`, mais rien ne tourne sans `.await` |
| `Zeroizing` / `zeroize` | (aucun équivalent JS) effacer activement la RAM |
| `Mutex`, `Send + Sync` | verrou de concurrence / « sûr en multi-thread » |

Tu as maintenant le vocabulaire pour lire le backend sans dictionnaire. Les chapitres suivants s'appuient sur ces briques sans les réexpliquer.

## Étape suivante

Passe à [03 — Clean Architecture](./03-clean-architecture.md) : la règle de dépendance, l'inversion par traits/interfaces, et le rôle du composition root qu'on vient d'apercevoir dans `build_state`.
