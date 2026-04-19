export type ViewSection = "runtime" | "settings" | "about";

export const SECTION_COPY = {
  runtime: {
    kicker: "Runtime",
    title: "Live monitor",
    description: "Watch captured activity, current RPC output and the recent runtime log. Requires a saved RPC profile first.",
  },
  settings: {
    kicker: "Settings",
    title: "RPC and local rules",
    description: "Configure Discord RPC first, then tune monitor behavior and local rule clauses in one place.",
  },
  about: {
    kicker: "About",
    title: "ActivityPing",
    description: "Project links and build information.",
  },
} satisfies Record<ViewSection, { kicker: string; title: string; description: string }>;

export const SECTION_ORDER: ViewSection[] = ["runtime", "settings", "about"];
