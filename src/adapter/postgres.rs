
pub type PostgresPool = ::r2d2::Pool<
    ::postgres::PostgresConnection,
    ::r2d2_postgres::Error,
    ::r2d2_postgres::PostgresPoolManager,
    ::r2d2::NoopErrorHandler>;