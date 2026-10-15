# ADR 0001 — Session globale plutôt que token par commande

- **Statut** : acceptée, octobre 2026
- **Constat d'origine** : M2 (audit du 3 juillet 2026)

## Contexte

`require_session` laisse passer une commande du coffre dès qu'**une** session
valide existe ; les commandes ne reçoivent pas le token. Le token retourné au
login ne sert qu'à `check_session` et `logout`.

L'application n'a **qu'une WebView**, locale, sans contenu distant : un seul
principal. Une WebView compromise dispose de toute façon du token (il vit dans
sa mémoire) : l'exiger sur chaque commande n'arrêterait pas cet attaquant, et
ajouterait un paramètre à ~35 commandes.

## Décision

La session est un **état temporel du processus** : « le coffre est déverrouillé
pendant au plus 15 min, et au plus 6 min sans activité ». Elle n'est pas une
capacité transmise à chaque appel.

Les protections réelles contre une WebView compromise sont ailleurs :

- **pattern d'isolation** Tauri : liste blanche des commandes ([isolation/index.js](../../src-tauri/isolation/index.js)) ;
- **aucun chemin de fichier** ne transite par l'IPC (dialogues ouverts côté Rust) ;
- CSP stricte, capabilities minimales ;
- `logout` exige le token du détenteur (une page ne peut pas verrouiller le coffre
  sous les pieds de l'utilisateur) ;
- expiration absolue **et** inactivité appliquées côté Rust.

## Conséquences

- À revoir si plusieurs fenêtres/WebViews ou un contenu distant apparaissent :
  il faudrait alors lier chaque commande à une session (token, ou identité de la
  fenêtre appelante).
- La garde reste centralisée dans `RequireSessionUseCase` ; aucune commande du
  coffre ne doit l'omettre (vérifié en revue, cf. checklist).
