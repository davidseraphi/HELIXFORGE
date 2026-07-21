export type FieldDef = {
  key: string;
  label: string;
  placeholder?: string;
  required?: boolean;
};

export type ActionDef = {
  action: string; // path suffix: POST <entity>/{id}/<action>
  label: string;
  /** "delete" = HTTP DELETE <entity>/{id} instead of POST <entity>/{id}/<action> */
  method?: "post" | "delete";
};

export type EntityDef = {
  path: string; // e.g. "/v1/courses"
  singular: string;
  plural: string;
  columns: { key: string; label: string }[];
  createFields?: FieldDef[];
  actions?: ActionDef[];
};

export type ChildDef = EntityDef & {
  /** e.g. "/v1/apps/{id}/pages" — {id} is the parent id */
  listTemplate: string;
  /** if present, POST here to create a child; {id} is the parent id */
  createTemplate?: string;
};

export type ProductDef = {
  slug: string;
  title: string;
  port: number;
  glyph: string;
  blurb: string;
  /** real standalone web app, when one exists */
  external?: string;
  parent?: EntityDef;
  child?: ChildDef;
  summaryPath?: string;
};

const NAME_FIELDS: FieldDef[] = [
  { key: "name", label: "Name", placeholder: "e.g. Q3 launch plan", required: true },
  { key: "description", label: "Description", placeholder: "optional" },
];

const TITLE_FIELDS: FieldDef[] = [
  { key: "title", label: "Title", placeholder: "e.g. First draft", required: true },
  { key: "body", label: "Body", placeholder: "optional" },
];

function childOf(path: string, singular: string, plural: string, actions: ActionDef[]): ChildDef {
  return {
    path,
    singular,
    plural,
    listTemplate: path,
    createTemplate: path,
    columns: [
      { key: "title", label: "Title" },
      { key: "status", label: "Status" },
    ],
    createFields: TITLE_FIELDS,
    actions,
  };
}

export const PRODUCTS: ProductDef[] = [
  {
    slug: "helix-collab",
    title: "HelixCollab",
    port: 8101,
    glyph: "Co",
    blurb: "Real-time collaborative workspace",
    external: "http://127.0.0.1:3101",
  },
  {
    slug: "helix-code",
    title: "HelixCode",
    port: 8102,
    glyph: "Ic",
    blurb: "AI-native collaborative IDE",
    external: "http://127.0.0.1:3102",
  },
  {
    slug: "helix-flow",
    title: "HelixFlow",
    port: 8103,
    glyph: "Fl",
    blurb: "Agentic automation & workflow engine",
    parent: {
      path: "/v1/workflows",
      singular: "workflow",
      plural: "workflows",
      columns: [
        { key: "name", label: "Name" },
        { key: "status", label: "Status" },
      ],
      createFields: NAME_FIELDS,
      actions: [{ action: "runs", label: "Run" }],
    },
    summaryPath: "/v1/domain/status",
  },
  {
    slug: "helix-insights",
    title: "HelixInsights",
    port: 8104,
    glyph: "In",
    blurb: "Predictive analytics & decision OS",
    parent: {
      path: "/v1/datasets",
      singular: "dataset",
      plural: "datasets",
      columns: [
        { key: "name", label: "Name" },
        { key: "status", label: "Status" },
      ],
      createFields: NAME_FIELDS,
      actions: [{ action: "delete", label: "Delete", method: "delete" }],
    },
    child: {
      path: "/v1/metrics",
      singular: "metric",
      plural: "metrics",
      listTemplate: "/v1/metrics",
      columns: [
        { key: "name", label: "Metric" },
        { key: "status", label: "Status" },
      ],
      actions: [{ action: "delete", label: "Delete", method: "delete" }],
    },
    summaryPath: "/v1/domain/status",
  },
  {
    slug: "helix-commerce",
    title: "HelixCommerce",
    port: 8105,
    glyph: "Cm",
    blurb: "AI e-commerce & marketplace builder",
    parent: {
      path: "/v1/products",
      singular: "product",
      plural: "products",
      columns: [
        { key: "name", label: "Name" },
        { key: "status", label: "Status" },
      ],
      createFields: [
        { key: "sku", label: "SKU", placeholder: "SKU-001", required: true },
        { key: "name", label: "Name", placeholder: "Product name", required: true },
        { key: "price_cents", label: "Price (cents)", placeholder: "1000", required: true },
        { key: "currency", label: "Currency", placeholder: "USD" },
        { key: "stock", label: "Stock", placeholder: "10" },
      ],
      actions: [],
    },
    child: {
      path: "/v1/orders",
      singular: "order",
      plural: "orders",
      listTemplate: "/v1/orders",
      columns: [
        { key: "id", label: "Order" },
        { key: "status", label: "Status" },
      ],
      actions: [{ action: "cancel", label: "Cancel" }],
    },
    summaryPath: "/v1/domain/status",
  },
  {
    slug: "helix-edu",
    title: "HelixEdu",
    port: 8106,
    glyph: "Ed",
    blurb: "Adaptive AI learning & certification",
    parent: {
      path: "/v1/courses",
      singular: "course",
      plural: "courses",
      columns: [
        { key: "title", label: "Title" },
        { key: "status", label: "Status" },
      ],
      createFields: TITLE_FIELDS,
      actions: [
        { action: "publish", label: "Publish" },
        { action: "unpublish", label: "Unpublish" },
        { action: "delete", label: "Delete" },
        { action: "restore", label: "Restore" },
      ],
    },
    child: {
      path: "/v1/enrollments",
      singular: "enrollment",
      plural: "enrollments",
      listTemplate: "/v1/enrollments",
      columns: [
        { key: "id", label: "Enrollment" },
        { key: "status", label: "Status" },
      ],
      actions: [{ action: "withdraw", label: "Withdraw" }],
    },
    summaryPath: "/v1/domain/status",
  },
  {
    slug: "helix-capital",
    title: "HelixCapital",
    port: 8107,
    glyph: "Ca",
    blurb: "AI financial operating system",
    parent: {
      path: "/v1/accounts",
      singular: "account",
      plural: "accounts",
      columns: [
        { key: "name", label: "Name" },
        { key: "status", label: "Status" },
      ],
      createFields: [
        { key: "name", label: "Name", placeholder: "Operating account", required: true },
        { key: "kind", label: "Kind", placeholder: "asset | liability | equity | revenue | expense" },
      ],
      actions: [
        { action: "close", label: "Close" },
        { action: "reopen", label: "Reopen" },
      ],
    },
    summaryPath: "/v1/reports/trial-balance",
  },
  {
    slug: "helix-well",
    title: "HelixWell",
    port: 8108,
    glyph: "We",
    blurb: "AI personal & team wellness",
    parent: {
      path: "/v1/habits",
      singular: "habit",
      plural: "habits",
      columns: [
        { key: "name", label: "Name" },
        { key: "status", label: "Status" },
      ],
      createFields: [
        { key: "name", label: "Name", placeholder: "Morning walk", required: true },
        { key: "cadence", label: "Cadence", placeholder: "daily" },
        { key: "target_per_period", label: "Target per period", placeholder: "1" },
      ],
      actions: [
        { action: "pause", label: "Pause" },
        { action: "resume", label: "Resume" },
        { action: "end", label: "End" },
        { action: "delete", label: "Delete" },
        { action: "restore", label: "Restore" },
      ],
    },
    summaryPath: "/v1/reports/habit-summary",
  },
  {
    slug: "helix-network",
    title: "HelixNetwork",
    port: 8109,
    glyph: "Ne",
    blurb: "Professional networking & opportunities",
    parent: {
      path: "/v1/profiles",
      singular: "profile",
      plural: "profiles",
      columns: [
        { key: "display_name", label: "Name" },
        { key: "status", label: "Status" },
      ],
      createFields: [
        { key: "display_name", label: "Display name", placeholder: "Ada Byte", required: true },
        { key: "headline", label: "Headline", placeholder: "Systems thinker" },
      ],
      actions: [
        { action: "deactivate", label: "Deactivate" },
        { action: "reactivate", label: "Reactivate" },
        { action: "delete", label: "Delete" },
        { action: "restore", label: "Restore" },
      ],
    },
    child: childOf("/v1/opportunities", "opportunity", "opportunities", [
      { action: "close", label: "Close" },
      { action: "reopen", label: "Reopen" },
      { action: "delete", label: "Delete" },
      { action: "restore", label: "Restore" },
    ]),
    summaryPath: "/v1/reports/network-summary",
  },
  {
    slug: "helix-forge-studio",
    title: "HelixForge Studio",
    port: 8110,
    glyph: "St",
    blurb: "No-code AI app & internal tool builder",
    parent: {
      path: "/v1/apps",
      singular: "app",
      plural: "apps",
      columns: [
        { key: "name", label: "Name" },
        { key: "status", label: "Status" },
      ],
      createFields: NAME_FIELDS,
      actions: [
        { action: "publish", label: "Publish" },
        { action: "unpublish", label: "Unpublish" },
        { action: "delete", label: "Delete" },
        { action: "restore", label: "Restore" },
      ],
    },
    child: childOf("/v1/apps/{id}/pages", "page", "pages", [
      { action: "archive", label: "Archive" },
      { action: "reopen", label: "Reopen" },
      { action: "delete", label: "Delete" },
      { action: "restore", label: "Restore" },
    ]),
    summaryPath: "/v1/reports/studio-summary",
  },
  {
    slug: "helix-synthbio",
    title: "HelixSynthBio",
    port: 8111,
    external: "http://127.0.0.1:3201",
    glyph: "Sb",
    blurb: "Synthetic biology design & virtual wet-lab",
    parent: {
      path: "/v1/designs",
      singular: "design",
      plural: "designs",
      columns: [
        { key: "name", label: "Name" },
        { key: "status", label: "Status" },
      ],
      createFields: NAME_FIELDS,
      actions: [
        { action: "submit", label: "Submit" },
        { action: "approve", label: "Approve" },
        { action: "return", label: "Return" },
        { action: "delete", label: "Delete" },
        { action: "restore", label: "Restore" },
      ],
    },
    child: childOf("/v1/designs/{id}/sims", "sim", "sims", [
      { action: "start", label: "Start" },
      { action: "complete", label: "Complete" },
      { action: "fail", label: "Fail" },
      { action: "delete", label: "Delete" },
      { action: "restore", label: "Restore" },
    ]),
    summaryPath: "/v1/reports/synthbio-summary",
  },
  {
    slug: "helix-lex-prime",
    title: "HelixLex Prime",
    port: 8112,
    glyph: "Lx",
    blurb: "Autonomous legal & regulatory intelligence",
    parent: {
      path: "/v1/matters",
      singular: "matter",
      plural: "matters",
      columns: [
        { key: "name", label: "Name" },
        { key: "status", label: "Status" },
      ],
      createFields: NAME_FIELDS,
      actions: [
        { action: "open", label: "Open" },
        { action: "close", label: "Close" },
        { action: "reopen", label: "Reopen" },
        { action: "delete", label: "Delete" },
        { action: "restore", label: "Restore" },
      ],
    },
    child: childOf("/v1/matters/{id}/filings", "filing", "filings", [
      { action: "file", label: "File" },
      { action: "withdraw", label: "Withdraw" },
      { action: "delete", label: "Delete" },
      { action: "restore", label: "Restore" },
    ]),
    summaryPath: "/v1/reports/lex-summary",
  },
  {
    slug: "helix-cura-prime",
    title: "HelixCura Prime",
    port: 8113,
    glyph: "Cu",
    blurb: "Enterprise clinical AI platform",
    parent: {
      path: "/v1/care_cases",
      singular: "case",
      plural: "cases",
      columns: [
        { key: "name", label: "Name" },
        { key: "status", label: "Status" },
      ],
      createFields: NAME_FIELDS,
      actions: [
        { action: "activate", label: "Activate" },
        { action: "discharge", label: "Discharge" },
        { action: "reopen", label: "Reopen" },
        { action: "delete", label: "Delete" },
        { action: "restore", label: "Restore" },
      ],
    },
    child: childOf("/v1/care_cases/{id}/notes", "note", "notes", [
      { action: "sign", label: "Sign" },
      { action: "void", label: "Void" },
      { action: "delete", label: "Delete" },
      { action: "restore", label: "Restore" },
    ]),
    summaryPath: "/v1/reports/cura-summary",
  },
  {
    slug: "helix-terra-prime",
    title: "HelixTerra Prime",
    port: 8114,
    glyph: "Te",
    blurb: "Precision agriculture & climate-smart farming",
    parent: {
      path: "/v1/fields",
      singular: "field",
      plural: "fields",
      columns: [
        { key: "name", label: "Name" },
        { key: "status", label: "Status" },
      ],
      createFields: NAME_FIELDS,
      actions: [
        { action: "activate", label: "Activate" },
        { action: "retire", label: "Retire" },
        { action: "reopen", label: "Reopen" },
        { action: "delete", label: "Delete" },
        { action: "restore", label: "Restore" },
      ],
    },
    child: childOf("/v1/fields/{id}/observations", "observation", "observations", [
      { action: "confirm", label: "Confirm" },
      { action: "dismiss", label: "Dismiss" },
      { action: "delete", label: "Delete" },
      { action: "restore", label: "Restore" },
    ]),
    summaryPath: "/v1/reports/terra-summary",
  },
  {
    slug: "helix-climate-prime",
    title: "HelixClimate Prime",
    port: 8115,
    glyph: "Cl",
    blurb: "Climate risk modeling & net-zero orchestration",
    parent: {
      path: "/v1/scenarios",
      singular: "scenario",
      plural: "scenarios",
      columns: [
        { key: "name", label: "Name" },
        { key: "status", label: "Status" },
      ],
      createFields: NAME_FIELDS,
      actions: [
        { action: "activate", label: "Activate" },
        { action: "archive", label: "Archive" },
        { action: "reopen", label: "Reopen" },
        { action: "delete", label: "Delete" },
        { action: "restore", label: "Restore" },
      ],
    },
    child: childOf("/v1/scenarios/{id}/risk_scores", "risk score", "risk scores", [
      { action: "assess", label: "Assess" },
      { action: "dismiss", label: "Dismiss" },
      { action: "delete", label: "Delete" },
      { action: "restore", label: "Restore" },
    ]),
    summaryPath: "/v1/reports/climate-summary",
  },
  {
    slug: "helix-orbit-prime",
    title: "HelixOrbit Prime",
    port: 8116,
    glyph: "Or",
    blurb: "Commercial space operations & satellites",
    parent: {
      path: "/v1/assets",
      singular: "asset",
      plural: "assets",
      columns: [
        { key: "name", label: "Name" },
        { key: "status", label: "Status" },
      ],
      createFields: NAME_FIELDS,
      actions: [
        { action: "commission", label: "Commission" },
        { action: "decommission", label: "Decommission" },
        { action: "recommission", label: "Recommission" },
        { action: "delete", label: "Delete" },
        { action: "restore", label: "Restore" },
      ],
    },
    child: childOf("/v1/assets/{id}/passes", "pass", "passes", [
      { action: "plan", label: "Plan" },
      { action: "complete", label: "Complete" },
      { action: "cancel", label: "Cancel" },
      { action: "delete", label: "Delete" },
      { action: "restore", label: "Restore" },
    ]),
    summaryPath: "/v1/reports/orbit-summary",
  },
  {
    slug: "helix-quantum-forge",
    title: "HelixQuantum Forge",
    port: 8117,
    glyph: "Qf",
    blurb: "Hybrid quantum-classical computing",
    parent: {
      path: "/v1/jobs",
      singular: "job",
      plural: "jobs",
      columns: [
        { key: "name", label: "Name" },
        { key: "status", label: "Status" },
      ],
      createFields: NAME_FIELDS,
      actions: [
        { action: "submit", label: "Submit" },
        { action: "complete", label: "Complete" },
        { action: "fail", label: "Fail" },
        { action: "delete", label: "Delete" },
        { action: "restore", label: "Restore" },
      ],
    },
    child: childOf("/v1/jobs/{id}/circuits", "circuit", "circuits", [
      { action: "validate", label: "Validate" },
      { action: "archive", label: "Archive" },
      { action: "delete", label: "Delete" },
      { action: "restore", label: "Restore" },
    ]),
    summaryPath: "/v1/reports/quantum-summary",
  },
  {
    slug: "helix-vita-prime",
    title: "HelixVita Prime",
    port: 8118,
    glyph: "Vi",
    blurb: "Precision medicine & longevity research",
    parent: {
      path: "/v1/studies",
      singular: "study",
      plural: "studies",
      columns: [
        { key: "name", label: "Name" },
        { key: "status", label: "Status" },
      ],
      createFields: NAME_FIELDS,
      actions: [
        { action: "recruit", label: "Recruit" },
        { action: "complete", label: "Complete" },
        { action: "terminate", label: "Terminate" },
        { action: "delete", label: "Delete" },
        { action: "restore", label: "Restore" },
      ],
    },
    child: childOf("/v1/studies/{id}/cohorts", "cohort", "cohorts", [
      { action: "enroll", label: "Enroll" },
      { action: "withdraw", label: "Withdraw" },
      { action: "delete", label: "Delete" },
      { action: "restore", label: "Restore" },
    ]),
    summaryPath: "/v1/reports/vita-summary",
  },
  {
    slug: "helix-grid-prime",
    title: "HelixGrid Prime",
    port: 8119,
    glyph: "Gr",
    blurb: "Autonomous smart energy systems",
    parent: {
      path: "/v1/sites",
      singular: "site",
      plural: "sites",
      columns: [
        { key: "name", label: "Name" },
        { key: "status", label: "Status" },
      ],
      createFields: NAME_FIELDS,
      actions: [
        { action: "energize", label: "Energize" },
        { action: "offline", label: "Offline" },
        { action: "online", label: "Online" },
        { action: "delete", label: "Delete" },
        { action: "restore", label: "Restore" },
      ],
    },
    child: childOf("/v1/sites/{id}/readings", "reading", "readings", [
      { action: "verify", label: "Verify" },
      { action: "reject", label: "Reject" },
      { action: "delete", label: "Delete" },
      { action: "restore", label: "Restore" },
    ]),
    summaryPath: "/v1/reports/grid-summary",
  },
  {
    slug: "helix-nova-labs",
    title: "HelixNova Labs",
    port: 8120,
    glyph: "Nv",
    blurb: "Open scientific discovery accelerator",
    parent: {
      path: "/v1/experiments",
      singular: "experiment",
      plural: "experiments",
      columns: [
        { key: "name", label: "Name" },
        { key: "status", label: "Status" },
      ],
      createFields: NAME_FIELDS,
      actions: [
        { action: "start", label: "Start" },
        { action: "conclude", label: "Conclude" },
        { action: "reopen", label: "Reopen" },
        { action: "delete", label: "Delete" },
        { action: "restore", label: "Restore" },
      ],
    },
    child: childOf("/v1/experiments/{id}/findings", "finding", "findings", [
      { action: "confirm", label: "Confirm" },
      { action: "reject", label: "Reject" },
      { action: "delete", label: "Delete" },
      { action: "restore", label: "Restore" },
    ]),
    summaryPath: "/v1/reports/nova-summary",
  },
  {
    slug: "helix-pulse",
    title: "HelixPulse",
    port: 8121,
    glyph: "Pu",
    blurb: "Sovereign monitoring & incident response",
    parent: {
      path: "/v1/monitors",
      singular: "monitor",
      plural: "monitors",
      columns: [
        { key: "name", label: "Name" },
        { key: "status", label: "Status" },
      ],
      createFields: NAME_FIELDS,
      actions: [
        { action: "activate", label: "Activate" },
        { action: "pause", label: "Pause" },
        { action: "resume", label: "Resume" },
        { action: "delete", label: "Delete" },
        { action: "restore", label: "Restore" },
      ],
    },
    child: childOf("/v1/monitors/{id}/incidents", "incident", "incidents", [
      { action: "acknowledge", label: "Acknowledge" },
      { action: "resolve", label: "Resolve" },
      { action: "delete", label: "Delete" },
      { action: "restore", label: "Restore" },
    ]),
    summaryPath: "/v1/reports/pulse-summary",
  },
];

export function findProduct(slug: string): ProductDef | undefined {
  return PRODUCTS.find((p) => p.slug === slug);
}
