# BasicPresence

Application desktop de **suivi de présence** (bureau / remote) avec le **mode de déplacement** utilisé, afin de calculer les **émissions de CO₂** de chaque personne.

Les données de présence étant **très confidentielles**, l'app est **offline-first** et **chiffrée au repos** (niveau « bancaire ») : la base ne se déchiffre qu'après saisie du mot de passe.

- 🖥️ **Tauri v2** (Rust) + **React 19 / TypeScript** (Vite)
- 🔒 **Auth Argon2id** + chiffrement par _envelope encryption_ (clé dérivée du mot de passe)
- 💾 **libSQL** (moteur de Turso) — 100 % local en v1, **synchro cloud prévue en Phase 2**
- 🧱 **Clean Architecture** côté Rust et React

> Documentation : [Claude.md](Claude.md) (architecture) · [documentation/auth.md](documentation/auth.md) (sécurité) · [documentation/turso.md](documentation/turso.md) (persistance).

---

## Prérequis

| Outil | Version | Notes |
| --- | --- | --- |
| **Rust** | stable (rustup) | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| **Node.js** | ≥ 20.19 | requis par Vite 7 |
| **pnpm** | ≥ 9 | `npm install -g pnpm` (ou `corepack enable`) |
| **Dépendances système Tauri** | — | voir ci-dessous |

### Dépendances système (Tauri v2)

Suivre la page officielle : <https://v2.tauri.app/start/prerequisites/>. Exemples :

```bash
# Debian / Ubuntu
sudo apt update && sudo apt install -y libwebkit2gtk-4.1-dev build-essential \
  curl wget file libxdo-dev libssl-dev libayatana-appindicator3-dev librsvg2-dev

# Arch / Manjaro
sudo pacman -S --needed webkit2gtk-4.1 base-devel curl wget file openssl \
  libayatana-appindicator librsvg
```

(macOS : Xcode Command Line Tools. Windows : Microsoft C++ Build Tools + WebView2.)

---

## Installation (poste de dev)

```bash
# 1. Récupérer le projet
git clone <url-du-repo> BasicPresenceTauri
cd BasicPresenceTauri

# 2. Installer les dépendances front
pnpm install
```

> Les dépendances Rust sont téléchargées/compilées automatiquement au premier lancement
> (la première compilation est longue, ensuite c'est mis en cache).

---

## Lancer en développement

```bash
pnpm tauri dev
```

Cela démarre Vite **et** ouvre la fenêtre Tauri avec hot-reload.

**Au premier lancement** : l'écran **« Créer un compte »** s'affiche (appareil mono-utilisateur).
Une fois le compte créé → écran **Connexion** → **Accueil** (« Bonjour {username} » + compte à rebours de session). La session dure **15 min** et le mot de passe est redemandé à chaque démarrage.

> Les bases de données (`keystore.db` clair + `vault.db` chiffré) sont créées dans le dossier _app-data_ de l'OS (ex. `~/.local/share/com.godef.basic-presence/` sous Linux).

### Dépannage (Linux — affichage WebKitGTK)

Sous Linux, WebKitGTK peut échouer à l'ouverture de la fenêtre :

- `Gdk-Message: Error 71 (Protocol error) dispatching to Wayland display`
- `Failed to create GBM buffer of size 800x600: Invalid argument`

Ce sont des problèmes d'**accélération graphique** (le renderer DMA-BUF de WebKitGTK alloue
ses buffers via **GBM**, ce que les pilotes **NVIDIA** ne gèrent pas correctement) — pas un
bug applicatif. Le correctif fiable est de désactiver le renderer DMA-BUF ; il fonctionne
**aussi bien en Wayland natif qu'en X11**. Deux scripts sont fournis :

```bash
# Wayland natif (recommandé) :
pnpm dev:wayland     # = WEBKIT_DISABLE_DMABUF_RENDERER=1 tauri dev

# Repli X11 (via XWayland) si besoin :
pnpm dev:x11         # = GDK_BACKEND=x11 WEBKIT_DISABLE_DMABUF_RENDERER=1 tauri dev
```

Notes :

- Ces scripts sont **spécifiques à Linux** (ces variables n'ont d'effet que sur WebKitGTK).
- Si l'écran reste blanc, essayer aussi `WEBKIT_DISABLE_COMPOSITING_MODE=1`.
- Le repli X11 requiert XWayland (Arch : `sudo pacman -S --needed xorg-xwayland`).
- **En production**, ces variables doivent être présentes dans l'environnement de
  l'utilisateur final concerné (wrapper de lancement / fichier `.desktop`), ou gérées
  automatiquement côté Rust (ex. le crate `webkit2gtk-nvidia-quirk`, qui détecte le pilote
  NVIDIA et applique le bon réglage).

---

## Commandes utiles

```bash
# Frontend
pnpm dev           # serveur Vite seul
pnpm typecheck     # tsc --noEmit
pnpm lint          # ESLint
pnpm format        # Prettier (écriture)
pnpm test          # Vitest

# Backend Rust (depuis src-tauri/)
cargo check
cargo test                     # tests unitaires + intégration
cargo clippy -- -D warnings    # lint sans warning
cargo fmt
```

Avant un commit : `pnpm typecheck && pnpm lint && pnpm test` puis, dans `src-tauri/`, `cargo fmt && cargo clippy -- -D warnings && cargo test`.

---

## Build de production

```bash
pnpm tauri build
```

Cela compile le front (`tsc && vite build` → `dist/`), compile le binaire Rust en _release_, puis génère les installeurs.

**Artefacts produits** (cible `all` configurée dans [src-tauri/tauri.conf.json](src-tauri/tauri.conf.json)) :

- Binaire natif : `src-tauri/target/release/basic-presence`
- Installeurs : `src-tauri/target/release/bundle/`
  - **Linux** : `bundle/deb/`, `bundle/rpm/`, `bundle/appimage/`
  - **Windows** : `bundle/msi/`, `bundle/nsis/`
  - **macOS** : `bundle/dmg/`, `bundle/macos/` (.app)

> Le bundle est généré **pour la plateforme courante** uniquement. Pour cibler une autre plateforme, builder depuis cette plateforme (ou via CI). Pour la production, pensez à **épingler les versions** et à signer les binaires (signature de code / notarisation selon l'OS).

---

## Structure du projet

```
BasicPresenceTauri/
├── src/                 # Frontend React (core/ + features/auth/ — feature-first)
├── src-tauri/           # Backend Rust (domain / application / infrastructure / presentation)
│   ├── migrations/      # SQL embarqué (keystore/ + vault/)
│   └── src/             # voir Claude.md §5
├── documentation/       # auth.md, turso.md
└── Claude.md            # référence d'architecture
```

Détails dans [Claude.md](Claude.md).

---

## Sécurité (en bref)

- Mot de passe haché en **Argon2id** — jamais stocké en clair, jamais envoyé au frontend.
- Coffre `vault.db` chiffré (AES-256-CBC) avec une clé **dérivée du mot de passe** (envelope encryption + XChaCha20-Poly1305).
- Session **15 min absolue**, en mémoire uniquement ; **anti-bruteforce** par verrouillage.
- Aucun secret commité ni exposé au frontend.

Détails et limites connues : [documentation/auth.md](documentation/auth.md).
