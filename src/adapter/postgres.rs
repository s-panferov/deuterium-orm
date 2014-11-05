
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

    pub fn exec<'conn, 'a>(stm: &'conn Statement<'conn>, params: &[&'a ToSql], ctx_params: &'a[Box<AsPostgresValue + Send + Sync>]) -> PostgresResult<Rows<'conn>> {
        stm.query(PostgresAdapter::prepare_params(params, ctx_params).as_slice())
    }
}