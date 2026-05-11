export type Unsubscribe = () => void;

export interface PolledReader<T> {
  read(): Promise<T>;
  subscribe?(onChange: () => void): Unsubscribe | Promise<Unsubscribe>;
}
