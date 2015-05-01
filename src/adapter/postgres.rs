use postgres::{
    Rows,
    GenericConnection,
    Statement
};

use postgres::Result as PostgresResult;
use postgres::types::ToSql;
use deuterium::{SqlContext, QueryToSql};

pub type PostgresPool = ::r2d2::Pool<::r2d2_postgres::PostgresConnectionManager>;
pub type PostgresPooledConnection<'a> = ::r2d2::PooledConnection<
    'a,
    ::r2d2_postgres::PostgresConnectionManager,
>;

pub fn setup(cn_str: &str, pool_size: u32) -> PostgresPool {
    let manager = ::r2d2_postgres::PostgresConnectionManager::new(cn_str, ::postgres::SslMode::None).unwrap();
    let config = ::r2d2::Config::builder()
        .pool_size(pool_size)
        .build();

    let handler = Box::new(::r2d2::NoopErrorHandler);
    ::r2d2::Pool::new(config, manager, handler).unwrap()
}

#[allow(missing_copy_implementations)]
pub struct PostgresAdapter;

impl PostgresAdapter {
    pub fn prepare_query<'conn>(query: &QueryToSql, cn: &'conn GenericConnection) -> (SqlContext, PostgresResult<Statement<'conn>>){
        let mut ctx = SqlContext::new(Box::new(::deuterium::sql::adapter::PostgreSqlAdapter));
        let sql = query.to_final_sql(&mut ctx);

        (ctx, cn.prepare(sql.as_slice()))
    }

    pub fn prepare_params<'a>(
            ext_params: &[&'a ToSql],
            ctx_params: &'a[Box<ToSql + 'static>]
        ) -> Vec<&'a (ToSql + 'a)> {

        let mut final_params = vec![];

        for param in ext_params.iter() {
            final_params.push(*param);
        }

        for param in ctx_params.iter() {
            final_params.push(&**param);
        }

        final_params
    }

    pub fn query<'conn, 'a>(stm: &'conn Statement<'conn>, params: &[&'a ToSql], ctx_params: &'a[Box<ToSql + 'static>]) -> PostgresResult<Rows<'conn>> {
        stm.query(PostgresAdapter::prepare_params(params, ctx_params).as_slice())
    }

    pub fn execute<'conn, 'a>(stm: &'conn Statement<'conn>, params: &[&'a ToSql], ctx_params: &'a[Box<ToSql + 'static>]) -> PostgresResult<u64> {
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
macro_rules! to_sql_string_pg {
    ($query:expr) => ({
        let mut ctx = ::deuterium::SqlContext::new(Box::new(::deuterium::sql::adapter::PostgreSqlAdapter));
        $query.to_final_sql(&mut ctx)
    })
}

#[macro_export]
macro_rules! query_pg {
    ($query:expr, $cn:expr, $params:expr, $rows:ident, $blk:block) => ({
        let (ctx, maybe_stm) = ::deuterium_orm::adapter::postgres::PostgresAdapter::prepare_query($query, $cn);
        let stm = match maybe_stm {
            Ok(stm) => stm,
            Err(e) => panic!("SQL query `{}` panicked at {}:{} with error `{}`",
                to_sql_string_pg!($query), file!(), line!(), e
            )
        };

        let $rows = ::deuterium_orm::adapter::postgres::PostgresAdapter::query(&stm, $params, ctx.data());

        let $rows = match $rows {
            Ok($rows) => $rows,
            Err(e) => panic!("SQL query `{}` panicked at {}:{} with error `{}`",
                to_sql_string_pg!($query), file!(), line!(), e
            ),
        };

        $blk
    });
}

#[macro_export]
macro_rules! query_models_iter {
    ($query:expr, $cn:expr, $params:expr) => (
        query_pg!($query, $cn, $params, rows, {
            rows.iter().map(|row| {
                ::deuterium_orm::adapter::postgres::from_row($query, &row)
            })
        })
    )
}

#[macro_export]
macro_rules! query_models {
    ($query:expr, $cn:expr, $params:expr) => (
        query_pg!($query, $cn, $params, rows, {
            let vec: Vec<_> = rows.iter().map(|row| {
                ::deuterium_orm::adapter::postgres::from_row($query, &row)
            }).collect();
            vec
        })
    )
}

#[macro_export]
macro_rules! query_model {
    ($query:expr, $cn:expr, $params:expr) => (
        query_pg!($query, $cn, $params, rows, {
            rows.iter().take(1).next().map(|row| {
                ::deuterium_orm::adapter::postgres::from_row($query, &row)
            })
        })
    )
}

#[macro_export]
macro_rules! exec_pg_safe {
    ($query:expr, $cn:expr, $params:expr) => ({
        let (ctx, maybe_stm) = ::deuterium_orm::adapter::postgres::PostgresAdapter::prepare_query($query, $cn);
        let stm = maybe_stm.unwrap();
        ::deuterium_orm::adapter::postgres::PostgresAdapter::execute(&stm, $params, ctx.data())
    })
}

#[macro_export]
macro_rules! exec_pg {
    ($query:expr, $cn:expr, $params:expr) => ({
        match exec_pg_safe!($query, $cn, $params) {
            Ok(res) => res,
            Err(e) => panic!("SQL query `{}` panicked at {}:{} with error `{}`",
                to_sql_string_pg!($query), file!(), line!(), e
            )
        }
    })
}

#[macro_export]
macro_rules! try_pg {
    ($e:expr) => (
        match $e {
            Ok(ok) => ok,
            Err(err) => return Err(::postgres::Error::IoError(err))
        }
    )
}

#[macro_export]
macro_rules! deuterium_enum {
    ($en:ty) => (
        impl ::postgres::types::FromSql for $en {
            fn from_sql<R: ::std::io::Read>(ty: &::postgres::types::Type, raw: &mut R) -> ::postgres::Result<$en> {
                use ::byteorder::{ReadBytesExt};
                let val = raw.read_u8();
                match val {
                    Ok(val) => Ok(::std::num::FromPrimitive::from_u8(val).unwrap()),
                    Err(_) => Err(::postgres::Error::WrongType(ty.clone()))
                }
            }

            fn accepts(_ty: &::postgres::types::Type) -> bool {
                true
            }
        }

        impl ::deuterium::ToSql for $en {
            fn to_sql(&self, ctx: &mut ::deuterium::SqlContext) -> String {
                let i = self.clone() as i16;
                i.to_predicate_value(ctx)
            }
        }

        impl ::deuterium::UntypedExpression for $en {
            fn expression_as_sql(&self) -> &ToSql {
                self
            }

            fn upcast_expression(&self) -> SharedExpression {
                let i = self.clone() as i16;
                ::std::rc::Rc::new(Box::new(i) as ::deuterium::BoxedExpression)
            }
        }

        impl ::deuterium::ToExpression<$en> for $en {}
        impl ::deuterium::ToExpression<i16> for $en {}
        impl ::deuterium::ToExpression<RawExpression> for $en {}

        impl ::deuterium::ToPredicateValue for $en {
            fn to_predicate_value(&self, ctx: &mut SqlContext) -> String {
                let i = self.clone() as i16;
                ctx.hold(Box::new(i))
            }
        }
    )
}
