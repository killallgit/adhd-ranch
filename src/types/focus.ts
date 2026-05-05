export interface Task {
  readonly id: string;
  readonly text: string;
  readonly done: boolean;
}

export interface Focus {
  readonly id: string;
  readonly title: string;
  readonly description: string;
  readonly tasks: readonly Task[];
}
