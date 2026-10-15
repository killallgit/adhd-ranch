import { useCallback, useEffect, useRef, useState } from "react";
import { getSettings, updateSettings } from "../api/settings";
import { forgetTimerMinute, rememberTimerMinute } from "../lib/timerPreset";

export interface SavedTimerMinutes {
  readonly minutes: readonly number[];
  readonly add: (minutes: number) => void;
  readonly remove: (minutes: number) => void;
  readonly clear: () => void;
  readonly reload: () => void;
}

export function useSavedTimerMinutes(): SavedTimerMinutes {
  const [minutes, setMinutes] = useState<readonly number[]>([]);
  const minutesRef = useRef(minutes);
  minutesRef.current = minutes;
  const revision = useRef(0);
  const writeChain = useRef(Promise.resolve());

  const persist = useCallback((next: readonly number[]) => {
    writeChain.current = writeChain.current
      .then(async () => {
        const current = await getSettings();
        if (!current) return;
        await updateSettings({ ...current, timers: { saved_minutes: [...next] } });
      })
      .catch((error: unknown) => {
        console.error(error);
      });
  }, []);

  const reload = useCallback(() => {
    const rev = revision.current;
    writeChain.current = writeChain.current
      .then(async () => {
        if (revision.current !== rev) return;
        const current = await getSettings();
        if (revision.current !== rev) return;
        setMinutes(current?.timers?.saved_minutes ?? []);
      })
      .catch((error: unknown) => {
        console.error(error);
      });
  }, []);

  useEffect(() => {
    reload();
  }, [reload]);

  const add = useCallback(
    (minute: number) => {
      const next = rememberTimerMinute(minutesRef.current, minute);
      if (next === minutesRef.current) return;
      revision.current += 1;
      setMinutes(next);
      persist(next);
    },
    [persist],
  );

  const remove = useCallback(
    (minute: number) => {
      const next = forgetTimerMinute(minutesRef.current, minute);
      if (next === minutesRef.current) return;
      revision.current += 1;
      setMinutes(next);
      persist(next);
    },
    [persist],
  );

  const clear = useCallback(() => {
    if (minutesRef.current.length === 0) return;
    revision.current += 1;
    setMinutes([]);
    persist([]);
  }, [persist]);

  return { minutes, add, remove, clear, reload };
}
