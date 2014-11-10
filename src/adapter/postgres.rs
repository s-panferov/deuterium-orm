
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

#[macro_export]
macro_rules! to_sql_string_pg(
    ($query:expr) => ({
        let mut ctx = SqlContext::new(box ::deuterium::sql::adapter::PostgreSqlAdapter);
        $query.to_final_sql(&mut ctx)
    })
)

#[macro_export]
macro_rules! query_pg(
    ($query:expr, $cn:expr, $params:expr, $rows:ident, $blk:block) => ({
        let (ctx, maybe_stm) = ::deuterium_orm::adapter::postgres::PostgresAdapter::prepare_query($query, $cn);
        let stm = maybe_stm.unwrap();
        let $rows = ::deuterium_orm::adapter::postgres::PostgresAdapter::query(&stm, $params, ctx.data());
        
        $blk
    });

    ($query:expr, $cn:expr, $params:expr) => ({
        let (ctx, maybe_stm) = ::deuterium_orm::adapter::postgres::PostgresAdapter::prepare_query($query, $cn);
        let stm = maybe_stm.unwrap();
        ::deuterium_orm::adapter::postgres::PostgresAdapter::execute(&stm, $params, ctx.data())
    })
)