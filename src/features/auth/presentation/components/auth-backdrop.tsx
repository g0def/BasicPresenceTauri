/**
 * Décor d'arrière-plan purement visuel de l'écran d'authentification :
 * deux halos diffus (vert fougère / bleu présence) plus une scène SVG ancrée en
 * bas de l'écran, pleine largeur — une maison (télétravail) et un immeuble de
 * bureau dans les coins, reliés par un trajet qui défile, surmontés d'une
 * feuille (l'empreinte carbone). Ancrée en bas et étalée sur toute la largeur,
 * elle reste visible de part et d'autre de la carte de connexion centrée.
 *
 * Entièrement décoratif : `aria-hidden`, non focusable, sans interaction. Les
 * couleurs passent par les tokens de thème (clair/sombre automatiques) et les
 * animations sont neutralisées si l'utilisateur préfère moins de mouvement
 * (classes `auth-route` / `auth-leaf` définies dans index.css).
 */
export function AuthBackdrop() {
  return (
    <div
      aria-hidden="true"
      className="pointer-events-none fixed inset-0 -z-10 overflow-hidden"
    >
      {/* Halos diffus — ambiance « éco-tech », teinte le fond mint/sombre.
          Ils dérivent et respirent lentement (cf. auth-glow-* dans index.css). */}
      <div className="auth-glow-1 absolute -left-24 -top-24 size-80 rounded-full bg-success/15 blur-3xl" />
      <div className="auth-glow-2 absolute -bottom-32 -right-24 size-104 rounded-full bg-primary/15 blur-3xl" />

      {/* Scène ancrée en bas, pleine largeur : domicile ↔ bureau */}
      <svg
        viewBox="0 0 1200 200"
        fill="none"
        preserveAspectRatio="xMidYMax meet"
        className="absolute inset-x-0 bottom-0 h-[min(40vh,280px)] w-full"
      >
        {/* Trajet domicile → bureau : pointillés qui défilent */}
        <path
          d="M230 150 C 430 96, 770 96, 970 150"
          className="auth-route stroke-current text-muted-foreground/60"
          strokeWidth={3}
          strokeLinecap="round"
          strokeDasharray="6 10"
        />

        {/* Maison — télétravail (vert fougère), coin bas-gauche */}
        <g
          className="stroke-current text-success/70"
          strokeWidth={4}
          strokeLinejoin="round"
          strokeLinecap="round"
        >
          <path d="M70 176 V120 L150 76 L230 120 V176" />
          <path d="M130 176 V146 H170 V176" />
        </g>

        {/* Bureau (bleu présence), coin bas-droite */}
        <g
          className="stroke-current text-primary/70"
          strokeWidth={4}
          strokeLinejoin="round"
          strokeLinecap="round"
        >
          <path d="M970 176 V96 H1130 V176" />
          <path d="M996 118 H1036 M1064 118 H1104 M996 144 H1036 M1064 144 H1104" />
        </g>

        {/* Feuille flottante (l'empreinte carbone), à gauche du centre */}
        <g className="auth-leaf text-success/80">
          <path
            d="M360 44 C 386 56, 386 92, 360 104 C 334 92, 334 56, 360 44 Z"
            className="fill-success/10 stroke-current"
            strokeWidth={3}
            strokeLinejoin="round"
          />
          <path
            d="M360 52 V100"
            className="stroke-current"
            strokeWidth={2.5}
            strokeLinecap="round"
          />
        </g>
      </svg>
    </div>
  );
}
