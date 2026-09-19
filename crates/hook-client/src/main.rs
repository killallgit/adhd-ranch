//! The command Claude Code runs when something happens in a session.
//!
//! It hands the event to the running ranch over a Unix socket and gets out of the
//! way. Everything here is in service of one rule: this runs inside someone's turn,
//! so it must be fast and it must never fail in a way they notice. No ranch
//! listening, no socket, a full buffer — all of it exits 0 and says nothing.
//!
//! Deliberately dependency-free: it is spawned several times per turn, so its cost
//! is dominated by how little it has to load.

fn main() {
    // Failure is silent by construction: the caller is an agent mid-turn, not a user
    // at a prompt, and there is nothing useful it could do with an error anyway.
    //
    // On Windows there is nothing to deliver to — the ranch listens on a Unix socket
    // and installs no hook there — but the binary still builds, so a workspace build
    // and a bundle hold the same set of files on every platform.
    #[cfg(unix)]
    let _ = unix::deliver();
}

#[cfg(unix)]
mod unix {
    use std::io::{Read, Write};
    use std::os::unix::net::UnixStream;
    use std::time::Duration;

    /// Long enough for a local socket that is already listening, short enough that a
    /// wedged ranch cannot hold up a turn.
    const TIMEOUT: Duration = Duration::from_millis(250);

    /// A hook payload is a few hundred bytes; anything vastly larger is not ours.
    const MAX_PAYLOAD: u64 = 1 << 20;

    pub fn deliver() -> Option<()> {
        let mut args = std::env::args().skip(1);
        let socket = args.next()?;
        let action = args.next()?;

        let mut payload = Vec::new();
        std::io::stdin()
            .take(MAX_PAYLOAD)
            .read_to_end(&mut payload)
            .ok()?;

        let mut stream = UnixStream::connect(socket).ok()?;
        stream.set_write_timeout(Some(TIMEOUT)).ok()?;

        // One frame: the action on its own line, then the agent's JSON verbatim. The
        // ranch is the only reader and it parses the payload, so nothing is gained by
        // re-encoding it here.
        stream.write_all(action.as_bytes()).ok()?;
        stream.write_all(b"\n").ok()?;
        stream.write_all(&payload).ok()?;
        stream.flush().ok()?;
        Some(())
    }
}
