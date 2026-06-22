/** Curated, canonical reference links per source organisation, opened in the
 * system browser. Labels are proper nouns (language-neutral); the section
 * heading/intro are translated. Mirrors documentation/calcul-impact-co2.md §10. */
export interface SourceLink {
  label: string;
  url: string;
}

export interface SourceGroup {
  /** Organisation name (proper noun, shown as-is). */
  org: string;
  links: SourceLink[];
}

export const SOURCE_GROUPS: SourceGroup[] = [
  {
    org: "ADEME",
    links: [
      { label: "Base Empreinte®", url: "https://base-empreinte.ademe.fr/" },
      {
        label: "Impact CO₂ — Transport",
        url: "https://impactco2.fr/outils/transport",
      },
    ],
  },
  {
    org: "DEFRA / DESNZ (UK)",
    links: [
      {
        label: "GHG conversion factors 2024",
        url: "https://www.gov.uk/government/publications/greenhouse-gas-reporting-conversion-factors-2024",
      },
    ],
  },
  {
    org: "SNCF",
    links: [
      {
        label: "Open Data — émissions CO₂",
        url: "https://ressources.data.sncf.com/explore/dataset/emission-co2-perimetre-complet/",
      },
    ],
  },
  {
    org: "SNCB",
    links: [
      {
        label: "Émissions du trafic ferroviaire",
        url: "http://www.belgianrail.be/fr/corporate/durabilite/planet/~/media/3EE35EEED3F74468B64DFA702CDCFB8A.ashx",
      },
    ],
  },
  {
    org: "Eurostar",
    links: [
      {
        label: "Facteur d'émission (DEFRA)",
        url: "https://help.eurostar.com/faq/uk-en/question/What-is-the-Co2-emission-factor-per-kilometer-when-using-Eurostar",
      },
    ],
  },
  {
    org: "CIBSE / Univ. Exeter",
    links: [
      {
        label: "Homeworking vs office (Circular Ecology)",
        url: "https://circularecology.com/news/the-carbon-emissions-of-homeworking-and-office-working",
      },
    ],
  },
  {
    org: "Électricité (réseaux)",
    links: [
      {
        label: "Our World in Data — intensité carbone",
        url: "https://ourworldindata.org/grapher/carbon-intensity-electricity",
      },
      {
        label: "Agence européenne de l'environnement (EEA)",
        url: "https://www.eea.europa.eu/en/analysis/indicators/greenhouse-gas-emission-intensity-of-1/greenhouse-gas-emission-intensity-of-electricity-generation",
      },
    ],
  },
];
