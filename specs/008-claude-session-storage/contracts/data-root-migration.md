# Ranch Data-Root Migration Contract

## Paths

| Platform | Legacy default | New default |
|----------|----------------|-------------|
| macOS/Linux | `$HOME/.adhd-ranch` | `$HOME/.adhd` |
| Windows with `APPDATA` | `%APPDATA%\adhd-ranch` | `%APPDATA%\adhd` |
| Windows fallback | `%USERPROFILE%\.adhd-ranch` | `%USERPROFILE%\.adhd` |

An explicit `ADHD_RANCH_HOME` is neither a legacy nor a new default. It is used exactly as supplied.

## Startup algorithm

The application prepares the data root before resolving Settings, Focuses, database, stable hook
client, or socket paths:

1. Resolve whether the root is explicit or platform-default.
2. For an explicit root, create it if needed and return it without migration.
3. For a default root, inspect the legacy and new paths without creating either one first.
4. Legacy only: rename the whole legacy directory to the new sibling path.
5. Neither: create the new directory.
6. New only: use it unchanged.
7. Both: return a conflict error and write to neither directory.
8. Apply user-only directory permissions where supported.
9. Only then initialize Settings, Focuses, `ranch.db`, hook client, and hook socket below the returned
   root.

## Failure and rollback

- A failed rename leaves the legacy directory authoritative and startup stops before any new-root
  write.
- A successful same-parent rename is the migration commit point.
- An older Ranch version can be restored by stopping Ranch and renaming `.adhd` back to
  `.adhd-ranch`; it ignores `ranch.db`.
- The migration never copies, merges, deletes, or resolves conflicts file-by-file.
- The plugin wrapper must resolve the same new default root in the release that enables this
  migration; otherwise it could continue sending to the legacy socket.
- The rapid-prototype exception for the unshipped archive schema/protocol does not apply here:
  existing Focus, Timer, Settings, client, and socket state still receive this whole-root migration.
