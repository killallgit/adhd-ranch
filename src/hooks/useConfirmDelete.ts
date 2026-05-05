import { useEffect, useState } from "react";
import { getSettings } from "../api/settings";

export function useConfirmDelete(initial = true): boolean {
  const [confirmDelete, setConfirmDelete] = useState(initial);

  useEffect(() => {
    getSettings()
      .then((s) => setConfirmDelete(s?.widget?.confirm_delete ?? initial))
      .catch(console.error);
  }, [initial]);

  return confirmDelete;
}
