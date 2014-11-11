#[macro_export]
macro_rules! define_model {
    (
        // Name of model, e.g. `Jedi`
        $model:ident,
        $model_meta:ident,
        // Name of table-helper, e.g. `JediTable`
        $table:ident, 
        // Name of table-helper `ManySelectQueryExt` trait e.g. `JediTableManySelectQueryExt`
        $many_select_query_ext:ident,
        // Name of table-helper `OneSelectQueryExt` trait e.g. `JediTableOneSelectQueryExt` 
        $one_select_query_ext:ident, 
        // Table name in database
        $table_name:expr, 
        // Collection of fields
        [ $((
            $field_name:ident, 
            $field_type:ty, 
            $field_name_f:ident, 
            $field_get:ident, 
            $field_set:ident, 
            $field_changed_flag:ident, // the name of internal flag field
            $field_changed_accessor:ident, // accessor name
            $($vis:tt)*)),+ ],

        [
            $($before_create:ident),*
        ],
        [
            $($before_save:ident),*
        ]
    ) => (
        #[deriving(Default, Show, Clone)]
        #[allow(dead_code)]
        pub struct $model {
            $(
                $field_name: Option<$field_type>,
            )+  
            __meta: $model_meta
        }

        #[deriving(Default, Show, Clone)]
        #[allow(dead_code)]
        pub struct $model_meta {
            $(
                $field_changed_flag: bool,
            )+  
            changed: bool,
        }

        impl $model_meta {
            pub fn new() -> $model_meta {
                $model_meta {
                    $(
                        // FIXME a macros issue don't allows to use `false` here
                        $field_changed_flag: !true,
                    )+  
                    changed: false,
                }
            }
        }

        impl $model {

            $(
                #[allow(dead_code)]
                pub fn $field_get(&self) -> &$field_type {
                    return self.$field_name.as_ref().unwrap();
                }

                #[allow(dead_code)]
                pub fn $field_set(&mut self, value: $field_type) {
                    self.$field_name = Some(value);
                    self.__meta.changed = true;
                    self.__meta.$field_changed_flag = true;
                }

                #[allow(dead_code)]
                pub fn $field_changed_accessor(&self) -> bool {
                    self.__meta.$field_changed_flag
                }
            )+  

            fn empty() -> $model {
                $model {
                   $(
                       $field_name: None,
                   )+
                   __meta: $model_meta::new()
                }
            }

            fn from_row<T, L>(query: &::deuterium::SelectQuery<T, L, $model>, row: &::postgres::Row) -> $model {
                match &query.select {
                    &::deuterium::SelectAll => {
                        $model {
                           $(
                                $field_name: Some(row.get(stringify!($field_name))),
                           )+
                           __meta: $model_meta::new()
                        }
                    },
                    &::deuterium::SelectOnly(_) => {
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
        pub struct $table(::deuterium::TableDef);

        #[allow(dead_code)]
        impl $model {

            pub fn table_name() -> &'static str {
                $table_name
            }

            pub fn table() -> $table {
                $table(::deuterium::TableDef::new($model::table_name()))
            }

            pub fn alias(alias: &str) -> $table {
                $table(::deuterium::TableDef::new_with_alias($model::table_name(), alias))
            }

            $(
                pub fn $field_name_f() -> ::deuterium::NamedField<$field_type> {
                    ::deuterium::NamedField::<$field_type>::new(stringify!($field_name), $model::table_name())
                }
            )+   

            pub fn create(&mut self) -> ::deuterium::InsertQuery<(), (), $model, (), ()> {
                let query = {
                    let mut fields: Vec<&::deuterium::Field> = vec![];
                    let mut values: Vec<&::deuterium::ToExpression<()>> = vec![];
                    
                    $(
                        $before_create(self);
                    )*                

                    $(
                        $before_save(self);
                    )*

                    $(
                        let $field_name;
                        if self.__meta.$field_changed_flag == true {
                            $field_name = $model::$field_name_f();
                            fields.push(&$field_name);
                            values.push(self.$field_get());
                        }
                    )+  

                    let mut query = $model::table().insert_fields(fields.as_slice());
                    query.push_untyped(values.as_slice());
                    query 
                };

                $(
                    if self.__meta.$field_changed_flag == true {
                        self.__meta.$field_changed_flag = false;
                    }
                )+  

                query
            }   

            pub fn update(&mut self) -> ::deuterium::UpdateQuery<(), ::deuterium::NoResult, $model> {

                $(
                    $before_save(self);
                )*

                let mut query = $model::table().update();

                $(
                    if self.__meta.$field_changed_flag == true {
                        let field = $model::$field_name_f().set(self.$field_get());
                        query = query.field(field);
                        self.__meta.$field_changed_flag = false;
                    }
                )+ 

                query.where_(self.lookup_predicate())
            }   

            pub fn delete(&mut self) -> ::deuterium::DeleteQuery<(), ::deuterium::NoResult, $model> {
                $model::table().delete().where_(self.lookup_predicate())
            }

        }

        impl ::deuterium::Table for $table {
            fn upcast_table(&self) -> ::deuterium::RcTable {
                ::std::sync::Arc::new(box self.clone() as ::deuterium::BoxedTable)
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
                pub fn $field_name_f(&self) -> ::deuterium::NamedField<$field_type> {
                    ::deuterium::NamedField::<$field_type>::field_of(stringify!($field_name), self)
                }
            )+  
        }

        impl ::deuterium::From for $table {
            fn as_sql(&self) -> &::deuterium::FromToSql {
                &self.0
            }

            fn upcast_from(&self) -> ::deuterium::RcFrom {
                ::std::sync::Arc::new(box self.clone() as ::deuterium::BoxedFrom)
            }
        }

        impl ::deuterium::Selectable<$model> for $table { }
        impl ::deuterium::Updatable<$model> for $table { }
        impl ::deuterium::Deletable<$model> for $table { }
        impl ::deuterium::Insertable<$model> for $table { }

        // SelectQuery extension
        pub trait $many_select_query_ext<T>: ::deuterium::QueryToSql {
            fn as_model_select_query(&self) -> &::deuterium::SelectQuery<T, ::deuterium::LimitMany, $model>;

            fn query_list(&self, cn: &::postgres::Connection, params: &[&::postgres::types::ToSql]) -> Vec<$model> {
                query_pg!(self, cn, params, rows, {
                    rows.unwrap().map(|row| {
                        $model::from_row(self.as_model_select_query(), &row)
                    }).collect()
                })
            }
        }

        // SelectQuery extension
        pub trait $one_select_query_ext<T>: ::deuterium::QueryToSql {
            fn as_model_select_query(&self) -> &::deuterium::SelectQuery<T, ::deuterium::LimitOne, $model>;

            fn query_list(&self, cn: &::postgres::Connection, params: &[&::postgres::types::ToSql]) -> Vec<$model> {
                query_pg!(self, cn, params, rows, {
                    rows.unwrap().map(|row| {
                        $model::from_row(self.as_model_select_query(), &row)
                    }).collect()
                })
            }

            fn query(&self, cn: &::postgres::Connection, params: &[&::postgres::types::ToSql]) -> Option<$model> {
                query_pg!(self, cn, params, rows, {
                    rows.unwrap().next().map(|row| $model::from_row(self.as_model_select_query(), &row))
                })
            }
        }

        impl<T> $many_select_query_ext<T> for ::deuterium::SelectQuery<T, ::deuterium::LimitMany, $model> {
            fn as_model_select_query(&self) -> &::deuterium::SelectQuery<T, ::deuterium::LimitMany, $model> {
                self
            }
        }

        impl<T> $one_select_query_ext<T> for ::deuterium::SelectQuery<T, ::deuterium::LimitOne, $model> {
            fn as_model_select_query(&self) -> &::deuterium::SelectQuery<T, ::deuterium::LimitOne, $model> {
                self
            }
        }
    )
}

#[macro_export]
macro_rules! primary_key(
    ($s:ident, $model:ident, $body:block) => (
        impl $model {
            #[allow(dead_code)]
            pub fn lookup_predicate(&$s) -> ::deuterium::RcPredicate {
                $body
            }
        }
        // TODO lookup_predicate
        // TODO get_primary()
        // TODO get_primary_f()
    )
)