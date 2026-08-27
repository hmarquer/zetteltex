use super::*;

/// Run a SQLite-backed operation with retry-on-lock, since parallel renders can
/// transiently contend for the database while notes are being looked up.
pub(crate) fn run_with_sqlite_lock_retry<T, F>(label: &str, mut operation: F) -> Result<T>
where
    F: FnMut() -> Result<T>,
{
    const MAX_ATTEMPTS: usize = 8;

    for attempt in 1..=MAX_ATTEMPTS {
        match operation() {
            Ok(value) => return Ok(value),
            Err(err) => {
                let retryable = is_sqlite_lock_error(&err);
                if retryable && attempt < MAX_ATTEMPTS {
                    let backoff_ms = 200_u64 * attempt as u64;
                    warn!(
                        "{} hit sqlite lock (attempt {}/{}), retrying in {}ms",
                        label, attempt, MAX_ATTEMPTS, backoff_ms
                    );
                    std::thread::sleep(Duration::from_millis(backoff_ms));
                    continue;
                }
                return Err(err);
            }
        }
    }

    bail!("{}", tr!("{label} fallo despues de reintentos", "{label} failed after retries"))
}

fn is_sqlite_lock_error(err: &anyhow::Error) -> bool {
    let msg = err.to_string().to_lowercase();
    msg.contains("database is locked")
        || msg.contains("database table is locked")
        || msg.contains("database busy")
}
