import { useCallback, useEffect, useRef, useState } from "react";
import { editSavedTimerMinutes, getSettings } from "../api/settings";
import { forgetTimerMinute, rememberTimerMinute } from "../lib/timerPreset";
import type { TimerPresetEdit } from "../types/generated/TimerPresetEdit";

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
  const revision = useRef(0);
  const writeChain = useRef(Promise.resolve());

  const persist = useCallback((edit: TimerPresetEdit) => {
    const rev = revision.current;
    writeChain.current = writeChain.current
      .then(async () => {
        const current = await editSavedTimerMinutes(edit);
        if (revision.current !== rev) return;
        minutesRef.current = current.saved_minutes;
        setMinutes(current.saved_minutes);
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
        const next = current?.timers?.saved_minutes ?? [];
        minutesRef.current = next;
        setMinutes(next);
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
      minutesRef.current = next;
      setMinutes(next);
      persist({ Add: minute });
    },
    [persist],
  );

  const remove = useCallback(
    (minute: number) => {
      const next = forgetTimerMinute(minutesRef.current, minute);
      if (next === minutesRef.current) return;
      revision.current += 1;
      minutesRef.current = next;
      setMinutes(next);
      persist({ Remove: minute });
    },
    [persist],
  );

  const clear = useCallback(() => {
    if (minutesRef.current.length === 0) return;
    revision.current += 1;
    minutesRef.current = [];
    setMinutes([]);
    persist("Clear");
  }, [persist]);

  return { minutes, add, remove, clear, reload };
}
