
use postgres::{
    Rows, 
    Connection, 
    Statement
};

use postgres::Result as PostgresResult;
use postgres::types::ToSql;
use deuterium::{SqlContext, AsPostgresValue, QueryToSql};

pub type PostgresPool = ::r2d2::Pool<
    Connection,
    ::r2d2_postgres::Error,
    ::r2d2_postgres::PostgresPoolManager,
    ::r2d2::NoopErrorHandler>;

pub type PostgresPooledConnection<'a> = ::r2d2::PooledConnection<
    'a, 
    ::postgres::Connection, 
    ::r2d2_postgres::Error, 
    ::r2d2_postgres::PostgresPoolManager, 
    ::r2d2::NoopErrorHandler
>;

pub fn setup(cn_str: &str, pool_size: uint) -> PostgresPool {
    let manager = ::r2d2_postgres::PostgresPoolManager::new(cn_str, ::postgres::NoSsl);
    let config = ::r2d2::Config {
        pool_size: pool_size,
        test_on_check_out: true,
        ..::std::default::Default::default()
    };

    let handler = ::r2d2::NoopErrorHandler;
    ::r2d2::Pool::new(config, manager, handler).unwrap()
}

pub struct PostgresAdapter;

impl PostgresAdapter {
    pub fn prepare_query<'conn>(query: &QueryToSql, cn: &'conn Connection) -> (SqlContext, PostgresResult<Statement<'conn>>){
        let mut ctx = SqlContext::new(box ::deuterium::sql::adapter::PostgreSqlAdapter);
        let sql = query.to_final_sql(&mut ctx);

        (ctx, cn.prepare(sql.as_slice()))
    }

    pub fn prepare_params<'a>(
            ext_params: &[&'a ToSql], 
            ctx_params: &'a[Box<AsPostgresValue + Send + Sync>]
        ) -> Vec<&'a ToSql + 'a> {

        let mut final_params = vec![];

        for param in ext_params.iter() {
            final_params.push(*param);
        }

        for param in ctx_params.iter() {
            final_params.push(param.as_postgres_value());
        }

        final_params
    }

    pub fn query<'conn, 'a>(stm: &'conn Statement<'conn>, params: &[&'a ToSql], ctx_params: &'a[Box<AsPostgresValue + Send + Sync>]) -> PostgresResult<Rows<'conn>> {
        stm.query(PostgresAdapter::prepare_params(params, ctx_params).as_slice())
    }

    pub fn execute<'conn, 'a>(stm: &'conn Statement<'conn>, params: &[&'a ToSql], ctx_params: &'a[Box<AsPostgresValue + Send + Sync>]) -> PostgresResult<uint> {
        stm.execute(PostgresAdapter::prepare_params(params, ctx_params).as_slice())
    }
}

pub trait FromRow {
    fn from_row<T, L>(query: &::deuterium::SelectQuery<T, L, Self>, row: &::postgres::Row) -> Self;
}

pub fn from_row<T, L, M: FromRow>(query: &::deuterium::SelectQuery<T, L, M>, row: &::postgres::Row) -> M {
    FromRow::from_row(query, row)
}

#[macro_export]
macro_rules! unwrap_or_report_sql_error(
    () => ()
)

#[macro_export]
macro_rules! to_sql_string_pg(
    ($query:expr) => ({
        let mut ctx = ::deuterium::SqlContext::new(box ::deuterium::sql::adapter::PostgreSqlAdapter);
        $query.to_final_sql(&mut ctx)
    })
)

#[macro_export]
macro_rules! query_pg_safe(
    ($query:expr, $cn:expr, $params:expr, $rows:ident, $blk:block) => ({
        let (ctx, maybe_stm) = ::deuterium_orm::adapter::postgres::PostgresAdapter::prepare_query($query, $cn);
        let stm = match maybe_stm {
            Ok(stm) => stm,
            Err(e) => panic!("SQL query `{}` panicked at {}:{} with error `{}`", 
                to_sql_string_pg!($query), file!(), line!(), e
            )
        };
        
        let $rows = ::deuterium_orm::adapter::postgres::PostgresAdapter::query(&stm, $params, ctx.data());
        
        $blk
    });
)

#[macro_export]
macro_rules! query_pg(
    ($query:expr, $cn:expr, $params:expr, $rows:ident, $blk:block) => ({
        let res = query_pg_safe!($query, $cn, $params, $rows, $blk);
        match res {
            Ok(res) => res,
            Err(e) => panic!("SQL query `{}` panicked at {}:{} with error `{}`", 
                to_sql_string_pg!($query), file!(), line!(), e
            )
        }
    });
)

#[macro_export]
macro_rules! query_models_iter(
    ($query:expr, $cn:expr, $params:expr) => (
        query_pg!($query, $cn, $params, rows, {
            rows.map(|iter| iter.map(|row| {
                ::deuterium_orm::adapter::postgres::from_row($query, &row)
            }))
        })
    )
)

#[macro_export]
macro_rules! query_models(
    ($query:expr, $cn:expr, $params:expr) => (
        (query_models_iter!($query, $cn, $params)).collect()
    )
)

#[macro_export]
macro_rules! query_model(
    ($query:expr, $cn:expr, $params:expr) => (
        query_pg!($query, $cn, $params, rows, {
            rows.map(|mut iter| iter.next().map(|row| {
                ::deuterium_orm::adapter::postgres::from_row($query, &row)
            }))
        })
    )
)

#[macro_export]
macro_rules! exec_pg_safe(
    ($query:expr, $cn:expr, $params:expr) => ({
        let (ctx, maybe_stm) = ::deuterium_orm::adapter::postgres::PostgresAdapter::prepare_query($query, $cn);
        let stm = maybe_stm.unwrap();
        ::deuterium_orm::adapter::postgres::PostgresAdapter::execute(&stm, $params, ctx.data())
    })
)

#[macro_export]
macro_rules! exec_pg(
    ($query:expr, $cn:expr, $params:expr) => ({
        match exec_pg_safe!($query, $cn, $params) {
            Ok(res) => res,
            Err(e) => panic!("SQL query `{}` panicked at {}:{} with error `{}`", 
                to_sql_string_pg!($query), file!(), line!(), e
            )
        }
    })
)