export interface Caps {
  readonly max_focuses: number;
  readonly max_tasks_per_focus: number;
}

export interface Alerts {
  readonly system_notifications: boolean;
}

export interface Widget {
  readonly always_on_top: boolean;
  readonly confirm_delete: boolean;
}

export interface DisplayConfig {
  readonly enabled_indices: readonly number[];
}

export interface Settings {
  readonly caps: Caps;
  readonly alerts: Alerts;
  readonly widget: Widget;
  readonly displays: DisplayConfig;
}
