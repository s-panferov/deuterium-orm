#[macro_export]
macro_rules! define_model {
    (
        // Name of model, e.g. `Jedi`
        $model:ident,
        // Name of table-helper, e.g. `JediTable`
        $table:ident, 
        // Name of table-helper `ManySelectQueryExt` trait e.g. `JediTableManySelectQueryExt`
        $many_select_query_ext:ident,
        // Name of table-helper `OneSelectQueryExt` trait e.g. `JediTableOneSelectQueryExt` 
        $one_select_query_ext:ident, 
        // Table name in database
        $table_name:expr, 
        // Collection of fields
        [ $(($field_name:ident, $field_type:ty, $field_name_f:ident)),+ ]
    ) => (
        #[deriving(Default, Show, Clone)]
        #[allow(dead_code)]
        struct $model {
            $(
                $field_name: Option<$field_type>,
            )+  
        }

        impl $model {
            fn empty() -> $model {
                $model {
                   $(
                        $field_name: None,
                    )+
                }
            }

            fn from_row<T, L>(query: &deuterium::SelectQuery<T, L, $model>, row: &postgres::Row) -> $model {
                match &query.select {
                    &SelectAll => {
                        $model {
                           $(
                                $field_name: Some(row.get(stringify!($field_name))),
                            )+
                        }
                    },
                    &SelectOnly(_) => {
                        let mut model = $model::empty();
                        $(
                            model.$field_name = match row.get_opt(stringify!($field_name)) {
                                Ok(val) => Some(val),
                                Err(_) => None
                            };
                        )+

                        model
                    }
                }
            }
        }

        #[deriving(Clone)]
        struct $table(deuterium::TableDef);

        #[allow(dead_code)]
        impl $model {

            pub fn table_name() -> &'static str {
                $table_name
            }

            pub fn table() -> $table {
                $table(deuterium::TableDef::new($model::table_name()))
            }

            pub fn alias(alias: &str) -> $table {
                $table(deuterium::TableDef::new_with_alias($model::table_name(), alias))
            }

            $(
                pub fn $field_name_f() -> NamedField<$field_type> {
                    NamedField::<$field_type>::new(stringify!($field_name), $model::table_name())
                }
            )+   
        }

        impl deuterium::Table for $table {
            fn upcast_table(&self) -> RcTable {
                Arc::new(box self.clone() as BoxedTable)
            }

            fn get_table_name(&self) -> &String {
                self.0.get_table_name()
            }

            fn get_table_alias(&self) -> &Option<String> {
                self.0.get_table_alias()
            }
        }

        #[allow(dead_code)]
        impl $table {
            $(
                pub fn $field_name_f(&self) -> NamedField<$field_type> {
                    NamedField::<$field_type>::field_of(stringify!($field_name), self)
                }
            )+  
        }

        impl deuterium::From for $table {
            fn as_sql(&self) -> &deuterium::FromToSql {
                &self.0
            }

            fn upcast_from(&self) -> RcFrom {
                Arc::new(box self.clone() as BoxedFrom)
            }
        }

        impl deuterium::Selectable<$model> for $table { }
        impl deuterium::Updatable<$model> for $table { }
        impl deuterium::Deletable<$model> for $table { }
        impl deuterium::Insertable<$model> for $table { }

        // SelectQuery extension
        trait $many_select_query_ext<T>: QueryToSql {
            fn as_model_select_query(&self) -> &SelectQuery<T, LimitMany, $model>;

            fn query_list(&self, cn: &Connection, params: &[&postgres::types::ToSql]) -> Vec<$model> {
                let (ctx, maybe_stm) = deuterium_orm::adapter::postgres::PostgresAdapter::prepare_query(self, cn);
                let stm = maybe_stm.unwrap();
                let rows = deuterium_orm::adapter::postgres::PostgresAdapter::exec(&stm, params, ctx.data()).unwrap();

                rows.map(|row| {
                    $model::from_row(self.as_model_select_query(), &row)
                }).collect()
            }
        }

        // SelectQuery extension
        trait $one_select_query_ext<T>: QueryToSql {
            fn as_model_select_query(&self) -> &SelectQuery<T, LimitOne, $model>;

            fn query_list(&self, cn: &Connection, params: &[&postgres::types::ToSql]) -> Vec<$model> {
                let (ctx, maybe_stm) = deuterium_orm::adapter::postgres::PostgresAdapter::prepare_query(self, cn);
                let stm = maybe_stm.unwrap();
                let rows = deuterium_orm::adapter::postgres::PostgresAdapter::exec(&stm, params, ctx.data()).unwrap();
                
                rows.map(|row| {
                    $model::from_row(self.as_model_select_query(), &row)
                }).collect()
            }

            fn query(&self, cn: &Connection, params: &[&postgres::types::ToSql]) -> Option<$model> {
                let (ctx, maybe_stm) = deuterium_orm::adapter::postgres::PostgresAdapter::prepare_query(self, cn);
                let stm = maybe_stm.unwrap();
                let mut rows = deuterium_orm::adapter::postgres::PostgresAdapter::exec(&stm, params, ctx.data()).unwrap();

                rows.next().map(|row| $model::from_row(self.as_model_select_query(), &row))
            }
        }

        impl<T> $many_select_query_ext<T> for SelectQuery<T, LimitMany, $model> {
            fn as_model_select_query(&self) -> &SelectQuery<T, LimitMany, $model> {
                self
            }
        }

        impl<T> $one_select_query_ext<T> for SelectQuery<T, LimitOne, $model> {
            fn as_model_select_query(&self) -> &SelectQuery<T, LimitOne, $model> {
                self
            }
        }
    )
}