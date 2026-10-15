# ADR 0002 — L'état d'intégrité du coffre vit dans le keystore scellé

- **Statut** : acceptée, octobre 2026

## Contexte

libSQL 0.9 chiffre le coffre en AES-256-CBC, **non authentifié**. L'évidence
d'altération reposait sur un HMAC du fichier stocké dans `vault.db.hmac` et sur
un marqueur `vault.db.dirty` (« session en cours », pour distinguer un crash
d'une altération).

Ces deux fichiers n'étaient pas protégés : quiconque pouvait modifier
`vault.db` pouvait aussi **supprimer `.hmac`** (verdict « pas de référence »,
accepté) ou **créer `.dirty`** (verdict « crash », accepté), même en `HardFail`.
Par ailleurs `.dirty` n'était écrit qu'**après** les migrations : un crash
pendant une migration produisait un faux `VAULT_TAMPERED` bloquant.

## Décision

- La référence (`baseline_mac`) et l'indicateur `session_open` vivent dans la
  table `vault_integrity` du **keystore**, scellé par la clé de device du
  trousseau OS : impossibles à effacer ou forger sans cette clé.
- `session_open = 1` est écrit **avant** d'ouvrir et de migrer le coffre ; si
  l'écriture échoue, le coffre ne s'ouvre pas.
- Un coffre absent alors qu'une référence existe est refusé (pas de recréation
  silencieuse d'un coffre vide).
- Format du MAC v2 : étiquette de domaine + longueur de chaque segment
  (fichier principal, WAL) — encodage injectif ; calcul en flux.
- Les fichiers annexes hérités sont lus **une seule fois** (migration) puis
  supprimés.
- Un arrêt non propre reste toléré (sinon tout crash bloquerait l'utilisateur),
  mais il est **journalisé et montré** à l'utilisateur à la connexion suivante.

## Conséquences

- Un attaquant qui détient la clé de device (session utilisateur compromise) peut
  toujours tout faire ; ce n'est pas le modèle de menace couvert.
- Un attaquant peut provoquer un crash puis modifier le fichier : c'est
  accepté (cas « session ouverte »), mais **visible** dans le journal de sécurité.
- La vraie solution est un chiffrement authentifié at-rest (voir ADR 0004).
