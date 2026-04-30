import type { Health } from "../types/health";

export interface HealthClient {
  check(): Promise<Health>;
}
