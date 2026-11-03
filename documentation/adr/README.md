# Décisions d'architecture (ADR)

Une décision par fichier : contexte, décision, conséquences. Une ADR n'est pas
réécrite quand la décision change : on en ajoute une nouvelle qui la remplace.

| # | Décision | Statut |
|---|---|---|
| [0001](0001-session-globale.md) | La session est un état global du processus, pas une capacité passée à chaque commande | Acceptée (2026-10) |
| [0002](0002-integrite-du-coffre.md) | L'état d'intégrité du coffre vit dans le keystore scellé | Acceptée (2026-10) |
| [0003](0003-tanstack-query.md) | L'état serveur du frontend passe par TanStack Query + un conteneur d'injection | Acceptée (2026-10) |
| [0004](0004-report-migration-turso.md) | Rester sur libsql 0.9 (AES-CBC + HMAC) ; migration vers `turso`/AEGIS reportée | Acceptée (2026-10) ; version portée à 0.10 par [0006](0006-migrations-de-stockage-verifiees.md) |
| [0005](0005-depot-pacman.md) | Distribution Arch par un dépôt pacman signé (release `arch-repo`) ; pas d'auto-update pour les installs pacman | Acceptée (2026-11) |
| [0006](0006-migrations-de-stockage-verifiees.md) | Une montée de version touchant au stockage ou à la crypto est prouvée par fixtures et vecteurs figés (libSQL 0.10, RustCrypto, keyring-core) | Acceptée (2026-11) |
