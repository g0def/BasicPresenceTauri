import { readdirSync } from "node:fs";

import js from "@eslint/js";
import globals from "globals";
import tseslint from "typescript-eslint";
import reactHooks from "eslint-plugin-react-hooks";
import reactRefresh from "eslint-plugin-react-refresh";
import jsxA11y from "eslint-plugin-jsx-a11y";
import prettier from "eslint-config-prettier";

// ---------------------------------------------------------------------------
// Import restrictions. In a flat config a rule set again in a later block
// REPLACES the earlier options, so every block that sets `no-restricted-imports`
// is built from the same pieces below (Tauri boundary + layer + domain purity).
// ---------------------------------------------------------------------------

/** Only designated boundary files may talk to Tauri (see the exemption block). */
const TAURI_PATHS = [
  {
    name: "@tauri-apps/api",
    message:
      "Only src/core/ipc.ts may import @tauri-apps/api. Go through a repository.",
  },
];
const TAURI_PATTERNS = [
  {
    group: ["@tauri-apps/api/*", "@tauri-apps/plugin-*"],
    message:
      "Tauri is reached only through src/core/ipc.ts, src/core/external-link.ts and the updater repository.",
  },
];
const NO_PARENT_RELATIVE = {
  group: ["../*"],
  message: 'Use the "@/..." alias, not parent-relative imports.',
};

/**
 * Architecture layers of the frontend (a layer may import the layers below it,
 * never the ones above):
 *
 *   app       composition root (src/app, main.tsx, App.tsx, router)
 *   routes    thin route files
 *   pages     screens composing several features
 *   features  one business capability each — isolated from one another
 *   entities  shared business models/contexts (profile, presence, commute, work day)
 *   shared    UI kit, generic hooks and formatters (also components/, lib/)
 *   core      IPC, DI container, errors, i18n, config
 *
 * Checked on the "@/..." alias every import uses (parent-relative imports are
 * banned, so a layer cannot be crossed with "../../").
 */
const UPPER = {
  app: [],
  routes: ["@/app/**"],
  pages: ["@/app/**", "@/routes/**"],
  features: ["@/app/**", "@/routes/**", "@/pages/**"],
  entities: ["@/app/**", "@/routes/**", "@/pages/**", "@/features/**"],
  shared: [
    "@/app/**",
    "@/routes/**",
    "@/pages/**",
    "@/features/**",
    "@/entities/**",
  ],
  core: [
    "@/app/**",
    "@/routes/**",
    "@/pages/**",
    "@/features/**",
    "@/entities/**",
    "@/shared/**",
    "@/components/**",
    "@/lib/**",
  ],
};

const FEATURES = readdirSync(new URL("./src/features", import.meta.url), {
  withFileTypes: true,
})
  .filter((d) => d.isDirectory())
  .map((d) => d.name);

/** Deliberate cross-feature dependencies: a presence day embeds its commute
 *  (segment editor + commute context). See CLAUDE.md, "presence → commute". */
const ALLOWED_FEATURE_DEPS = { presence: ["commute"] };

/** The domain layer of a feature (and the entity models) stays pure. */
const DOMAIN_PURITY = [
  {
    group: ["@/features/*/data/**", "@/features/*/presentation/**"],
    message: "The domain layer must not depend on data or presentation.",
  },
  {
    group: ["react", "react-dom", "@tanstack/*"],
    message: "The domain layer is framework-free.",
  },
];

function restrictImports({ forbidden = [], message = "", domain = false }) {
  return {
    "no-restricted-imports": [
      "error",
      {
        paths: TAURI_PATHS,
        patterns: [
          ...TAURI_PATTERNS,
          NO_PARENT_RELATIVE,
          ...(forbidden.length ? [{ group: forbidden, message }] : []),
          ...(domain ? DOMAIN_PURITY : []),
        ],
      },
    ],
  };
}

const TESTS = ["src/**/*.{test,spec}.{ts,tsx}", "src/test/**"];

const layerBlock = (layer, files) => ({
  files,
  ignores: TESTS,
  rules: restrictImports({
    forbidden: UPPER[layer],
    message: `The ${layer} layer must not import an upper layer.`,
  }),
});

// gitignore semantics: a path whose parent directory is excluded cannot be
// re-included, so the exclusion is on the feature *directory* ("@/features/*")
// and the own feature (plus the allowed ones) is re-included by name.
const featureForbidden = (name) => [
  ...UPPER.features,
  "@/features/*",
  `!@/features/${name}`,
  ...(ALLOWED_FEATURE_DEPS[name] ?? []).map((dep) => `!@/features/${dep}`),
];
const FEATURE_MESSAGE =
  "A feature must not import another feature (share through src/entities or src/shared).";

const ARCHITECTURE_BLOCKS = [
  layerBlock("routes", ["src/routes/**/*.{ts,tsx}"]),
  layerBlock("pages", ["src/pages/**/*.{ts,tsx}"]),
  layerBlock("entities", ["src/entities/**/*.{ts,tsx}"]),
  layerBlock("shared", ["src/{shared,components,lib}/**/*.{ts,tsx}"]),
  layerBlock("core", ["src/core/**/*.{ts,tsx}"]),
  // Pure entity models (not their React contexts/hooks).
  {
    files: ["src/entities/*/model/*.ts"],
    ignores: [
      ...TESTS,
      "src/entities/*/model/use-*.ts",
      "src/entities/*/model/*-context.ts",
    ],
    rules: restrictImports({
      forbidden: UPPER.entities,
      message: "The entities layer must not import an upper layer.",
      domain: true,
    }),
  },
  ...FEATURES.flatMap((name) => [
    {
      files: [`src/features/${name}/**/*.{ts,tsx}`],
      ignores: TESTS,
      rules: restrictImports({
        forbidden: featureForbidden(name),
        message: FEATURE_MESSAGE,
      }),
    },
    {
      files: [`src/features/${name}/domain/**/*.{ts,tsx}`],
      ignores: TESTS,
      rules: restrictImports({
        forbidden: featureForbidden(name),
        message: FEATURE_MESSAGE,
        domain: true,
      }),
    },
  ]),
];

export default tseslint.config(
  // Ignore build output, deps and the Rust side
  {
    ignores: [
      "dist",
      "node_modules",
      "src-tauri",
      "coverage",
      "*.config.js",
      "*.config.ts",
      // Generated by @tanstack/router-plugin — never hand-edited
      "src/routeTree.gen.ts",
    ],
  },

  // Base JS + type-aware TS recommended (floating/misused promises, unsafe any…)
  js.configs.recommended,
  ...tseslint.configs.recommendedTypeChecked,
  {
    languageOptions: {
      parserOptions: {
        projectService: true,
        tsconfigRootDir: import.meta.dirname,
      },
    },
  },

  // App source (browser env)
  {
    files: ["src/**/*.{ts,tsx}"],
    languageOptions: {
      ecmaVersion: 2020,
      globals: globals.browser,
    },
    plugins: {
      "react-hooks": reactHooks,
      "react-refresh": reactRefresh,
    },
    rules: {
      ...reactHooks.configs.recommended.rules,
      "react-refresh/only-export-components": [
        "warn",
        { allowConstantExport: true },
      ],
      "@typescript-eslint/no-explicit-any": "error",
      "@typescript-eslint/consistent-type-imports": "warn",
      ...restrictImports({}),
    },
  },

  // Accessibility of the JSX.
  { files: ["src/**/*.tsx"], ...jsxA11y.flatConfigs.recommended },
  {
    files: ["src/**/*.tsx"],
    rules: {
      // Focusing the first field of a login form / dialog is intended (and
      // what WAI-ARIA recommends for dialogs); only raw DOM autofocus is flagged.
      "jsx-a11y/no-autofocus": ["error", { ignoreNonDOM: true }],
    },
  },

  // Raw HTML injection is reserved to the two views that display HTML already
  // sanitized by the backend (comrak with unsafe_=false + ammonia).
  {
    files: ["src/**/*.tsx"],
    ignores: [
      "src/features/licence/presentation/pages/licence-page.tsx",
      "src/features/work-hours/presentation/components/day-note.tsx",
    ],
    rules: {
      "no-restricted-syntax": [
        "error",
        {
          selector: "JSXAttribute[name.name='dangerouslySetInnerHTML']",
          message:
            "Only backend-sanitized HTML may be injected (licence page, day note).",
        },
      ],
    },
  },

  // Architecture guard: layers, feature isolation, domain purity.
  ...ARCHITECTURE_BLOCKS,

  // TanStack Router signals redirects by throwing `redirect(...)`, and tests
  // reject with the plain `{ code, message }` objects the backend sends.
  {
    files: ["src/routes/**/*.{ts,tsx}", ...TESTS],
    rules: { "@typescript-eslint/only-throw-error": "off" },
  },

  // The Tauri boundary files + tests may import @tauri-apps/* directly
  // (must stay after every block that sets no-restricted-imports).
  {
    files: [
      "src/core/ipc.ts",
      "src/core/external-link.ts",
      "src/features/updater/data/repositories/tauri-updater.repository.ts",
      ...TESTS,
    ],
    rules: {
      "no-restricted-imports": "off",
    },
  },

  // MUST be last: disable formatting rules so Prettier owns formatting
  prettier,
);
