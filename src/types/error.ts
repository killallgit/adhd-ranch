export type CommandError =
  | { type: "bad_request"; message: string }
  | { type: "not_found"; message: string }
  | { type: "already_exists"; message: string }
  | { type: "validation"; message: string }
  | { type: "internal"; message: string };
