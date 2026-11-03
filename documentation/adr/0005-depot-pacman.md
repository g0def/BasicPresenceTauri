# ADR 0005 — Distribution Arch par un dépôt pacman signé hébergé sur GitHub

- **Statut** : acceptée, novembre 2026

## Contexte

Les utilisateurs Arch doivent pouvoir installer l'app et la mettre à jour avec
`pacman`. L'AUR, voie habituelle, n'accepte plus de nouvelles inscriptions
depuis plusieurs mois. L'AppImage fonctionne mais demande `fuse2`, ne s'intègre
pas au menu et embarque les bibliothèques d'Ubuntu 22.04.

## Décision

- Un **dépôt pacman signé GPG**, hébergé sur une release GitHub fixe
  (`arch-repo`, prerelease : elle ne devient jamais « latest », ce qui
  casserait `releases/latest/download/latest.json` de l'updater).
- Construit par `.github/workflows/arch-repo.yml` **à la publication** d'une
  release : le brouillon reste le gate manuel. Le `.deb` est vérifié
  (SHA256SUMS + attestation de provenance) puis reconditionné par le PKGBUILD
  existant, qui resservira tel quel sur l'AUR.
- Clé dédiée, dans l'environnement protégé `release` ; `SigLevel = Required`
  côté utilisateurs (pas de `TrustAll`, qui laisserait un compte GitHub
  compromis livrer n'importe quoi).
- Le binaire issu du `.deb` se croit installé par dpkg : l'updater Tauri
  lancerait `pkexec dpkg -i`. Le paquet pose un fichier marqueur
  (`/usr/lib/basic-presence/managed-by-pacman`) ; l'app l'interroge
  (`is_package_managed`) et ne fait alors qu'annoncer la mise à jour en
  renvoyant vers `pacman -Syu`.

## Alternatives écartées

- **AUR** : bloqué ; à reprendre quand les inscriptions rouvrent (en plus du
  dépôt, pas à la place).
- **GitHub Pages** : demande de pousser des binaires dans une branche ; les
  assets de release suffisent et gardent paquets et provenance au même endroit.
- **`.pkg.tar.zst` seul dans chaque release** (`pacman -U`) : pas de mise à
  jour par `pacman -Syu`.
