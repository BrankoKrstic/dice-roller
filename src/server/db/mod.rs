use std::{
    env,
    sync::Arc,
    time::{Duration, Instant},
};

use libsql::{Builder, Connection, Database, Rows, Transaction, params::IntoParams};
use thiserror::Error;
use tracing::{error, info, warn};

#[derive(Clone)]
pub struct Db {
    db: Arc<Database>,
    instrumentation: DbInstrumentationConfig,
}

const DEFAULT_EXECUTE_STATEMENT: &str = "sql.execute";
const DEFAULT_QUERY_STATEMENT: &str = "sql.query";
const DEFAULT_DB_SLOW_QUERY_MS: u64 = 100;

fn is_remote_database_url(value: &str) -> bool {
    let normalized = value.trim().to_ascii_lowercase();
    normalized.starts_with("libsql://")
        || normalized.starts_with("https://")
        || normalized.starts_with("http://")
        || normalized.starts_with("wss://")
}

#[derive(Debug, Error)]
pub enum DbError {
    #[error("missing required environment variable {0}")]
    MissingEnv(&'static str),
    #[error("validation failed: {0}")]
    Validation(String),
    #[error("invalid credentials")]
    InvalidCredentials,
    #[error("unauthorized: {0}")]
    Unauthorized(String),
    #[error("conflict: {0}")]
    Conflict(String),
    #[error("database error: {0}")]
    Database(String),
}

#[derive(Clone, Copy, Debug)]
struct DbInstrumentationConfig {
    slow_query_threshold: Duration,
}

impl DbInstrumentationConfig {
    fn from_env() -> Self {
        let slow_query_threshold = env::var("DB_SLOW_QUERY_MS")
            .ok()
            .and_then(|raw| raw.parse::<u64>().ok())
            .map(Duration::from_millis)
            .unwrap_or_else(|| Duration::from_millis(DEFAULT_DB_SLOW_QUERY_MS));

        Self {
            slow_query_threshold,
        }
    }

    fn is_slow(self, duration: Duration) -> bool {
        duration >= self.slow_query_threshold
    }
}

pub struct DbConnection {
    inner: Connection,
    instrumentation: DbInstrumentationConfig,
}

pub struct DbTransaction {
    inner: Option<Transaction>,
    instrumentation: DbInstrumentationConfig,
    opened_at: Instant,
    finished: bool,
}

pub struct DbRows {
    inner: Rows,
    state: QueryLogState,
}

struct QueryLogState {
    statement: &'static str,
    started_at: Instant,
    instrumentation: DbInstrumentationConfig,
    in_transaction: bool,
    rows_returned: u64,
    emitted: bool,
}

#[allow(async_fn_in_trait)]
pub trait DbExecutor {
    async fn execute<P>(&self, sql: &str, params: P) -> Result<u64, libsql::Error>
    where
        P: IntoParams;

    async fn execute_named<P>(
        &self,
        statement: &'static str,
        sql: &str,
        params: P,
    ) -> Result<u64, libsql::Error>
    where
        P: IntoParams;

    async fn query<P>(&self, sql: &str, params: P) -> Result<DbRows, libsql::Error>
    where
        P: IntoParams;

    async fn query_named<P>(
        &self,
        statement: &'static str,
        sql: &str,
        params: P,
    ) -> Result<DbRows, libsql::Error>
    where
        P: IntoParams;
}

impl Db {
    pub async fn from_env() -> Result<Self, DbError> {
        let db_url = env::var("TURSO_DATABASE_URL")
            .map_err(|_| DbError::MissingEnv("TURSO_DATABASE_URL"))?;

        Self::from_url(db_url).await
    }

    pub(crate) async fn from_url(db_url: String) -> Result<Self, DbError> {
        let instrumentation = DbInstrumentationConfig::from_env();
        let db = if is_remote_database_url(&db_url) {
            let db_token = env::var("TURSO_AUTH_TOKEN")
                .map_err(|_| DbError::MissingEnv("TURSO_AUTH_TOKEN"))?;
            Builder::new_synced_database("local.db", db_url, db_token)
                .sync_interval(Duration::from_secs(30))
                .build()
                .await
                .map_err(|error| DbError::Database(error.to_string()))?
        } else {
            Builder::new_local(db_url)
                .build()
                .await
                .map_err(|error| DbError::Database(error.to_string()))?
        };

        let db = Self {
            db: Arc::new(db),
            instrumentation,
        };
        let conn = db.connection()?;
        let db_clone = db.clone();
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(Duration::from_secs(30)).await;
                let started_at = Instant::now();

                if let Err(err) = db_clone.db.sync().await {
                    error!(
                        statement = "db.sync",
                        kind = "sync",
                        duration_ms = started_at.elapsed().as_millis() as u64,
                        outcome = "failed",
                        error = %err,
                        "database sync failed"
                    );
                } else {
                    info!(
                        statement = "db.sync",
                        kind = "sync",
                        duration_ms = started_at.elapsed().as_millis() as u64,
                        outcome = "complete",
                        "database sync completed"
                    );
                }
            }
        });
        conn.execute_named("db.enable_foreign_keys", "PRAGMA foreign_keys = ON", ())
            .await
            .map_err(|error| DbError::Database(error.to_string()))?;

        Ok(db)
    }

    pub fn connection(&self) -> Result<DbConnection, DbError> {
        let inner = self
            .db
            .connect()
            .map_err(|error| DbError::Database(error.to_string()))?;

        Ok(DbConnection {
            inner,
            instrumentation: self.instrumentation,
        })
    }
}

impl DbConnection {
    pub async fn execute<P>(&self, sql: &str, params: P) -> Result<u64, libsql::Error>
    where
        P: IntoParams,
    {
        <Self as DbExecutor>::execute(self, sql, params).await
    }

    pub async fn execute_named<P>(
        &self,
        statement: &'static str,
        sql: &str,
        params: P,
    ) -> Result<u64, libsql::Error>
    where
        P: IntoParams,
    {
        <Self as DbExecutor>::execute_named(self, statement, sql, params).await
    }

    pub async fn query<P>(&self, sql: &str, params: P) -> Result<DbRows, libsql::Error>
    where
        P: IntoParams,
    {
        <Self as DbExecutor>::query(self, sql, params).await
    }

    pub async fn query_named<P>(
        &self,
        statement: &'static str,
        sql: &str,
        params: P,
    ) -> Result<DbRows, libsql::Error>
    where
        P: IntoParams,
    {
        <Self as DbExecutor>::query_named(self, statement, sql, params).await
    }

    pub async fn transaction(&self) -> Result<DbTransaction, libsql::Error> {
        let started_at = Instant::now();
        match self.inner.transaction().await {
            Ok(transaction) => {
                log_db_duration(
                    "db.transaction",
                    "transaction",
                    started_at.elapsed(),
                    self.instrumentation,
                    false,
                    None,
                    None,
                    Some(true),
                );

                Ok(DbTransaction {
                    inner: Some(transaction),
                    instrumentation: self.instrumentation,
                    opened_at: Instant::now(),
                    finished: false,
                })
            }
            Err(error) => {
                error!(
                    statement = "db.transaction",
                    kind = "transaction",
                    duration_ms = started_at.elapsed().as_millis() as u64,
                    in_tx = false,
                    error = %error,
                    "database transaction failed"
                );
                Err(error)
            }
        }
    }
}

impl DbTransaction {
    pub async fn execute<P>(&self, sql: &str, params: P) -> Result<u64, libsql::Error>
    where
        P: IntoParams,
    {
        <Self as DbExecutor>::execute(self, sql, params).await
    }

    pub async fn execute_named<P>(
        &self,
        statement: &'static str,
        sql: &str,
        params: P,
    ) -> Result<u64, libsql::Error>
    where
        P: IntoParams,
    {
        <Self as DbExecutor>::execute_named(self, statement, sql, params).await
    }

    pub async fn query<P>(&self, sql: &str, params: P) -> Result<DbRows, libsql::Error>
    where
        P: IntoParams,
    {
        <Self as DbExecutor>::query(self, sql, params).await
    }

    pub async fn query_named<P>(
        &self,
        statement: &'static str,
        sql: &str,
        params: P,
    ) -> Result<DbRows, libsql::Error>
    where
        P: IntoParams,
    {
        <Self as DbExecutor>::query_named(self, statement, sql, params).await
    }

    pub async fn commit(mut self) -> Result<(), libsql::Error> {
        let started_at = Instant::now();
        let total_duration_ms = self.opened_at.elapsed().as_millis() as u64;
        let result = self.inner.take().expect("open transaction").commit().await;
        self.finished = true;

        match result {
            Ok(()) => {
                info!(
                    statement = "db.transaction.commit",
                    kind = "transaction",
                    duration_ms = started_at.elapsed().as_millis() as u64,
                    total_duration_ms,
                    in_tx = true,
                    outcome = "committed",
                    "database transaction completed"
                );
                Ok(())
            }
            Err(error) => {
                error!(
                    statement = "db.transaction.commit",
                    kind = "transaction",
                    duration_ms = started_at.elapsed().as_millis() as u64,
                    total_duration_ms,
                    in_tx = true,
                    error = %error,
                    "database transaction commit failed"
                );
                Err(error)
            }
        }
    }

    pub async fn rollback(mut self) -> Result<(), libsql::Error> {
        let started_at = Instant::now();
        let total_duration_ms = self.opened_at.elapsed().as_millis() as u64;
        let result = self
            .inner
            .take()
            .expect("open transaction")
            .rollback()
            .await;
        self.finished = true;

        match result {
            Ok(()) => {
                info!(
                    statement = "db.transaction.rollback",
                    kind = "transaction",
                    duration_ms = started_at.elapsed().as_millis() as u64,
                    total_duration_ms,
                    in_tx = true,
                    outcome = "rolled_back",
                    "database transaction rolled back"
                );
                Ok(())
            }
            Err(error) => {
                error!(
                    statement = "db.transaction.rollback",
                    kind = "transaction",
                    duration_ms = started_at.elapsed().as_millis() as u64,
                    total_duration_ms,
                    in_tx = true,
                    error = %error,
                    "database transaction rollback failed"
                );
                Err(error)
            }
        }
    }
}

impl Drop for DbTransaction {
    fn drop(&mut self) {
        if !self.finished {
            warn!(
                statement = "db.transaction.drop",
                kind = "transaction",
                duration_ms = self.opened_at.elapsed().as_millis() as u64,
                in_tx = true,
                outcome = "dropped",
                "database transaction dropped without explicit completion"
            );
        }
    }
}

impl DbRows {
    fn new(
        inner: Rows,
        statement: &'static str,
        instrumentation: DbInstrumentationConfig,
        in_transaction: bool,
    ) -> Self {
        Self {
            inner,
            state: QueryLogState {
                statement,
                started_at: Instant::now(),
                instrumentation,
                in_transaction,
                rows_returned: 0,
                emitted: false,
            },
        }
    }

    pub async fn next(&mut self) -> Result<Option<libsql::Row>, libsql::Error> {
        match self.inner.next().await {
            Ok(Some(row)) => {
                self.state.rows_returned += 1;
                Ok(Some(row))
            }
            Ok(None) => {
                self.emit_completion(true);
                Ok(None)
            }
            Err(error) => {
                self.emit_error(&error);
                Err(error)
            }
        }
    }

    fn emit_completion(&mut self, completed: bool) {
        if self.state.emitted {
            return;
        }

        self.state.emitted = true;
        log_db_duration(
            self.state.statement,
            "query",
            self.state.started_at.elapsed(),
            self.state.instrumentation,
            self.state.in_transaction,
            None,
            Some(self.state.rows_returned),
            Some(completed),
        );
    }

    fn emit_error(&mut self, error: &libsql::Error) {
        if self.state.emitted {
            return;
        }

        self.state.emitted = true;
        error!(
            statement = self.state.statement,
            kind = "query",
            duration_ms = self.state.started_at.elapsed().as_millis() as u64,
            in_tx = self.state.in_transaction,
            rows_returned = self.state.rows_returned,
            completed = false,
            error = %error,
            "database query failed"
        );
    }
}

impl Drop for DbRows {
    fn drop(&mut self) {
        self.emit_completion(false);
    }
}

impl DbExecutor for DbConnection {
    async fn execute<P>(&self, sql: &str, params: P) -> Result<u64, libsql::Error>
    where
        P: IntoParams,
    {
        self.execute_named(DEFAULT_EXECUTE_STATEMENT, sql, params)
            .await
    }

    async fn execute_named<P>(
        &self,
        statement: &'static str,
        sql: &str,
        params: P,
    ) -> Result<u64, libsql::Error>
    where
        P: IntoParams,
    {
        let started_at = Instant::now();
        match self.inner.execute(sql, params).await {
            Ok(rows_affected) => {
                log_db_duration(
                    statement,
                    "execute",
                    started_at.elapsed(),
                    self.instrumentation,
                    false,
                    Some(rows_affected),
                    None,
                    Some(true),
                );
                Ok(rows_affected)
            }
            Err(error) => {
                error!(
                    statement,
                    kind = "execute",
                    duration_ms = started_at.elapsed().as_millis() as u64,
                    in_tx = false,
                    error = %error,
                    "database execute failed"
                );
                Err(error)
            }
        }
    }

    async fn query<P>(&self, sql: &str, params: P) -> Result<DbRows, libsql::Error>
    where
        P: IntoParams,
    {
        self.query_named(DEFAULT_QUERY_STATEMENT, sql, params).await
    }

    async fn query_named<P>(
        &self,
        statement: &'static str,
        sql: &str,
        params: P,
    ) -> Result<DbRows, libsql::Error>
    where
        P: IntoParams,
    {
        let started_at = Instant::now();
        match self.inner.query(sql, params).await {
            Ok(rows) => {
                let mut traced_rows = DbRows::new(rows, statement, self.instrumentation, false);
                traced_rows.state.started_at = started_at;
                Ok(traced_rows)
            }
            Err(error) => {
                error!(
                    statement,
                    kind = "query",
                    duration_ms = started_at.elapsed().as_millis() as u64,
                    in_tx = false,
                    error = %error,
                    "database query failed"
                );
                Err(error)
            }
        }
    }
}

impl DbExecutor for DbTransaction {
    async fn execute<P>(&self, sql: &str, params: P) -> Result<u64, libsql::Error>
    where
        P: IntoParams,
    {
        self.execute_named(DEFAULT_EXECUTE_STATEMENT, sql, params)
            .await
    }

    async fn execute_named<P>(
        &self,
        statement: &'static str,
        sql: &str,
        params: P,
    ) -> Result<u64, libsql::Error>
    where
        P: IntoParams,
    {
        let started_at = Instant::now();
        match self
            .inner
            .as_ref()
            .expect("open transaction")
            .execute(sql, params)
            .await
        {
            Ok(rows_affected) => {
                log_db_duration(
                    statement,
                    "execute",
                    started_at.elapsed(),
                    self.instrumentation,
                    true,
                    Some(rows_affected),
                    None,
                    Some(true),
                );
                Ok(rows_affected)
            }
            Err(error) => {
                error!(
                    statement,
                    kind = "execute",
                    duration_ms = started_at.elapsed().as_millis() as u64,
                    in_tx = true,
                    error = %error,
                    "database execute failed"
                );
                Err(error)
            }
        }
    }

    async fn query<P>(&self, sql: &str, params: P) -> Result<DbRows, libsql::Error>
    where
        P: IntoParams,
    {
        self.query_named(DEFAULT_QUERY_STATEMENT, sql, params).await
    }

    async fn query_named<P>(
        &self,
        statement: &'static str,
        sql: &str,
        params: P,
    ) -> Result<DbRows, libsql::Error>
    where
        P: IntoParams,
    {
        let started_at = Instant::now();
        match self
            .inner
            .as_ref()
            .expect("open transaction")
            .query(sql, params)
            .await
        {
            Ok(rows) => {
                let mut traced_rows = DbRows::new(rows, statement, self.instrumentation, true);
                traced_rows.state.started_at = started_at;
                Ok(traced_rows)
            }
            Err(error) => {
                error!(
                    statement,
                    kind = "query",
                    duration_ms = started_at.elapsed().as_millis() as u64,
                    in_tx = true,
                    error = %error,
                    "database query failed"
                );
                Err(error)
            }
        }
    }
}

fn log_db_duration(
    statement: &'static str,
    kind: &'static str,
    duration: Duration,
    instrumentation: DbInstrumentationConfig,
    in_transaction: bool,
    rows_affected: Option<u64>,
    rows_returned: Option<u64>,
    completed: Option<bool>,
) {
    let duration_ms = duration.as_millis() as u64;
    let slow = instrumentation.is_slow(duration);

    info!(
        statement,
        kind,
        duration_ms,
        in_tx = in_transaction,
        rows_affected,
        rows_returned,
        completed,
        slow,
        "database operation completed"
    );

    if slow {
        warn!(
            statement,
            kind,
            duration_ms,
            in_tx = in_transaction,
            rows_affected,
            rows_returned,
            completed,
            slow = true,
            "slow database operation"
        );
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        io,
        sync::{Arc, Mutex},
    };

    use tracing::dispatcher::Dispatch;
    use tracing_subscriber::{
        fmt::{self, MakeWriter},
        layer::SubscriberExt,
        registry,
    };

    #[derive(Clone, Default)]
    struct TestLogWriter {
        buffer: Arc<Mutex<Vec<u8>>>,
    }

    impl TestLogWriter {
        fn output(&self) -> String {
            String::from_utf8(self.buffer.lock().expect("test log buffer lock").clone())
                .expect("test log buffer should be utf-8")
        }
    }

    impl<'a> MakeWriter<'a> for TestLogWriter {
        type Writer = Self;

        fn make_writer(&'a self) -> TestLogWriter {
            self.clone()
        }
    }

    impl io::Write for TestLogWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.buffer
                .lock()
                .expect("test log buffer lock")
                .extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    async fn capture_logs<F, Fut>(run: F) -> String
    where
        F: FnOnce() -> Fut,
        Fut: std::future::Future<Output = ()>,
    {
        let writer = TestLogWriter::default();
        let subscriber = registry().with(
            fmt::layer()
                .with_ansi(false)
                .without_time()
                .with_target(false)
                .with_writer(writer.clone()),
        );

        let dispatch = Dispatch::new(subscriber);
        let _guard = tracing::dispatcher::set_default(&dispatch);
        run().await;
        writer.output()
    }

    #[tokio::test(flavor = "current_thread")]
    async fn logs_query_completion_with_row_count() {
        let logs = capture_logs(|| async {
            let db = Db::from_url(":memory:".to_string()).await.expect("db");
            let conn = db.connection().expect("connection");

            conn.execute(
                "CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT NOT NULL)",
                (),
            )
            .await
            .expect("create table");
            conn.execute("INSERT INTO items (name) VALUES (?1)", ["alpha"])
                .await
                .expect("insert alpha");
            conn.execute("INSERT INTO items (name) VALUES (?1)", ["beta"])
                .await
                .expect("insert beta");

            let mut rows = conn
                .query("SELECT id, name FROM items ORDER BY id", ())
                .await
                .expect("query rows");

            while rows.next().await.expect("next row").is_some() {}
        })
        .await;

        assert!(
            logs.contains("kind=\"query\""),
            "expected query instrumentation log, got:\n{logs}"
        );
        assert!(
            logs.contains("rows_returned=2"),
            "expected returned row count in logs, got:\n{logs}"
        );
        assert!(
            logs.contains("completed=true"),
            "expected completed query log, got:\n{logs}"
        );
    }

    #[tokio::test(flavor = "current_thread")]
    async fn logs_partial_query_when_rows_are_dropped_early() {
        let logs = capture_logs(|| async {
            let db = Db::from_url(":memory:".to_string()).await.expect("db");
            let conn = db.connection().expect("connection");

            conn.execute(
                "CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT NOT NULL)",
                (),
            )
            .await
            .expect("create table");
            conn.execute("INSERT INTO items (name) VALUES (?1)", ["alpha"])
                .await
                .expect("insert alpha");
            conn.execute("INSERT INTO items (name) VALUES (?1)", ["beta"])
                .await
                .expect("insert beta");

            let mut rows = conn
                .query("SELECT id, name FROM items ORDER BY id", ())
                .await
                .expect("query rows");

            rows.next().await.expect("first row");
            drop(rows);
        })
        .await;

        assert!(
            logs.contains("kind=\"query\""),
            "expected query instrumentation log, got:\n{logs}"
        );
        assert!(
            logs.contains("rows_returned=1"),
            "expected partial row count in logs, got:\n{logs}"
        );
        assert!(
            logs.contains("completed=false"),
            "expected incomplete query log, got:\n{logs}"
        );
    }
}
