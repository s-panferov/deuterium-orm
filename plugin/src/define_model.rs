#[macro_export]
macro_rules! define_model {
    ($model:ident, $table:ident, $many_query:ident, $one_query:ident, $table_name:expr, [ $(($field_name:ident, $field_type:ty, $field_name_f:ident)),+ ]) => (

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

            fn from_row<T, L>(query: &deuterium::SelectQuery<T, L, $model>, row: &postgres::PostgresRow) -> $model {
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

            pub fn from() -> $table {
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

        // For select_* methods with proper type
        impl deuterium::Selectable<$model> for $table { }

        trait $many_query<T> {
            fn as_query(&self) -> &SelectQuery<T, LimitMany, $model>;

            fn query_list(&self, cn: &PostgresConnection) -> Vec<$model> {
                let prepared_query = cn.prepare(self.as_query().to_final_sql().as_slice());
                let rows = prepared_query.as_ref().unwrap().query(&[]).unwrap();

                rows.map(|row| {
                    Jedi::from_row(self.as_query(), &row)
                }).collect()
            }
        }

        trait $one_query<T> {
            fn as_query(&self) -> &SelectQuery<T, LimitOne, $model>;

            fn query_list(&self, cn: &PostgresConnection) -> Vec<$model> {
                let prepared_query = cn.prepare(self.as_query().to_final_sql().as_slice());
                let rows = prepared_query.as_ref().unwrap().query(&[]).unwrap();

                rows.map(|row| {
                    Jedi::from_row(self.as_query(), &row)
                }).collect()
            }

            fn query(&self, cn: &PostgresConnection) -> Option<$model> {
                let prepared_query = cn.prepare(self.as_query().to_final_sql().as_slice());
                let mut rows = prepared_query.as_ref().unwrap().query(&[]).unwrap();

                rows.next().map(|row| Jedi::from_row(self.as_query(), &row))
            }
        }

        impl<T> $many_query<T> for  SelectQuery<T, LimitMany, $model> {
            fn as_query(&self) -> &SelectQuery<T, LimitMany, $model> {
                self
            }
        }

        impl<T> $one_query<T> for  SelectQuery<T, LimitOne, $model> {
            fn as_query(&self) -> &SelectQuery<T, LimitOne, $model> {
                self
            }
        }
    )
}