import type { Focus } from "../types/focus";

export const HARDCODED_FOCUSES: readonly Focus[] = [
  {
    id: "customer-x-bug",
    title: "Customer X bug",
    description: "Persistence-on-restart issue surfaced on staging",
    tasks: [
      { id: "t-cx-1", text: "add persistence field in compute api" },
      { id: "t-cx-2", text: "update and test sdk" },
    ],
  },
  {
    id: "api-refactor",
    title: "API refactor",
    description: "Split request handler into pipeline stages",
    tasks: [
      { id: "t-ar-1", text: "extract pipeline trait" },
      { id: "t-ar-2", text: "swap inline calls for pipeline" },
      { id: "t-ar-3", text: "rerun benchmarks" },
    ],
  },
];
