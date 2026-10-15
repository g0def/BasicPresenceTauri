# ADR 0003 — TanStack Query et conteneur d'injection côté frontend

- **Statut** : acceptée, octobre 2026

## Contexte

Une quinzaine de providers et hooks recodaient à la main le même trio
`isLoading` / `error` / drapeau `cancelled`, et la gestion de `SESSION_EXPIRED`
(→ logout) était copiée sept fois — et oubliée dans quatre hooks. Les dépôts
`Tauri*` étaient instanciés au niveau module dans une douzaine de fichiers de
présentation, ce qui empêchait d'injecter des doublures en test (d'où des
`vi.mock` de modules).

## Décision

- **TanStack Query v5** pour l'état serveur : un `QueryClient` unique
  ([app/query-client.ts](../../src/app/query-client.ts)) ; pas de refetch au focus
  ni de retry (IPC local, l'UI est le seul écrivain, invalidation explicite après
  écriture) ; `SESSION_EXPIRED` traité **une fois** dans `QueryCache` /
  `MutationCache` ; cache **vidé au logout** (données confidentielles).
- Les écritures passent par `useCommand()` ([core/query.ts](../../src/core/query.ts))
  pour bénéficier de la même gestion d'erreur globale.
- Les **providers gardent leur API publique** : les composants n'ont pas changé.
- **Conteneur d'injection** ([core/di.ts](../../src/core/di.ts)) : chaque feature
  déclare les ports qu'elle consomme (`di.ts`, augmentation de module), la
  composition root ([app/container.ts](../../src/app/container.ts)) est le seul
  endroit qui instancie les dépôts `Tauri*`. Les tests injectent des faux via
  `withContainer(...)`.
- Même mécanisme pour l'**IPC typé** : chaque feature déclare ses commandes
  (`data/commands.ts`) dans `CommandRegistry` ; `invoke` est typé par nom de
  commande, et la liste `COMMANDS` est vérifiée exhaustive à la compilation.

## Conséquences

- Nouvelle dépendance (`@tanstack/react-query`), maintenue par l'équipe de
  TanStack Router déjà utilisé.
- Les éditeurs à sauvegarde différée (heures du jour, note) restent en état local
  (ce n'est pas du cache serveur) mais passent par `useCommand()`.
