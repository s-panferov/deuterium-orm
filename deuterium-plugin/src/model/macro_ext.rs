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
            $field_name:ident, // Field name, e.g. id
            $field_type:ty, // Field type, e.g. Uuid
            $field_name_f:ident, // Own field getter name, e.g. `id_f`
            $field_get:ident,  // Field's value getter name
            $field_set:ident,  // Field's value setter name
            $field_changed_flag:ident, // the name of internal flag field
            $field_changed_accessor:ident, // accessor name
            $($vis:tt)*)),+ // Hacky field visibility (not usable for now)
        ],

        [
            $($before_create:ident),*
        ],
        [
            $($before_save:ident),*
        ]
    ) => (

        // Generate Meta struct to hold dirty flags and other helpful internal state.

        #[derive(Default, Debug, Clone)]
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

        // Generate new model struct with all fields of type Option<$field_type>,
        // where None case we need to tell that field was not loaded from database.
        // New struct also will include __meta field where internal model state is stored.

        #[derive(Default, Debug, Clone)]
        #[allow(dead_code)]
        pub struct $model {
            $(
                $field_name: Option<$field_type>,
            )+
            __meta: $model_meta
        }

        impl $model {

            // Generate a set of methods to deal with fields values and fields dirty state.
            // We generate a getter, a setter and an accessor for each defined field.

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

            // Generate method to create empty model instance. `Empty` here means that all the fields
            // are in undefined state and all dirty bits are disabled.

            fn empty() -> $model {
                $model {
                   $(
                       $field_name: None,
                   )+
                   __meta: $model_meta::new()
                }
            }
        }

        // Very helpful stuff to unwrap Model instance from database Row.
        #[cfg(feature = "postgres")]
        impl ::deuterium_orm::adapter::postgres::FromRow for $model {
            fn from_row<T, L>(query: &::deuterium::SelectQuery<T, L, $model>, row: &::postgres::Row) -> $model {
                match query.get_select() {
                    &::deuterium::Select::All => {
                        $model {
                           $(
                                $field_name: Some(row.get(stringify!($field_name))),
                           )+
                           __meta: $model_meta::new()
                        }
                    },
                    &::deuterium::Select::Only(_) => {
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

        // We also generate ModelTable struct to deal with requests.

        #[derive(Clone)]
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

            fn call_before_create_hooks(&mut self) {
                $(
                    $before_create(self);
                )*
            }

            fn call_after_create_hooks(&mut self) {
                unimplemented!()
            }

            fn call_before_save_hooks(&mut self) {
                $(
                    $before_save(self);
                )*
            }

            fn call_after_save_hooks(&mut self) {
                unimplemented!()
            }

            fn call_before_update_hooks(&mut self) {
                unimplemented!()
            }

            fn call_after_update_hooks(&mut self) {
                unimplemented!()
            }

            fn call_before_destroy_hooks(&mut self) {
                unimplemented!()
            }

            fn call_after_destroy_hooks(&mut self) {
                unimplemented!()
            }

            pub fn create_query(&mut self) -> ::deuterium::InsertQuery<(), (), $model, (), ()> {
                let query = {
                    let mut fields: Vec<::deuterium::BoxedField> = vec![];
                    let mut values: Vec<&::deuterium::Expression<::deuterium::RawExpression>> = vec![];

                    self.call_before_create_hooks();
                    self.call_before_save_hooks();

                    $(
                        let $field_name;
                        if self.__meta.$field_changed_flag == true {
                            $field_name = $model::$field_name_f();
                            fields.push(Box::new($field_name));
                            values.push(self.$field_get().as_expr());
                        }
                    )+

                    let mut query = $model::table().insert_fields(fields.iter().map(|f| &**f).collect::<Vec<&::deuterium::Field>>().as_slice());
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

            pub fn update_query(&mut self) -> ::deuterium::UpdateQuery<(), ::deuterium::NoResult, $model> {

                self.call_before_save_hooks();

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

            pub fn delete_query(&mut self) -> ::deuterium::DeleteQuery<(), ::deuterium::NoResult, $model> {
                $model::table().delete().where_(self.lookup_predicate())
            }

        }

        impl ::deuterium::Table for $table {
            fn upcast_table(&self) -> ::deuterium::SharedTable {
                ::std::rc::Rc::new(Box::new(self.clone()) as ::deuterium::BoxedTable)
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

            fn upcast_from(&self) -> ::deuterium::SharedFrom {
                ::std::rc::Rc::new(Box::new(self.clone()) as ::deuterium::BoxedFrom)
            }
        }

        // Implement an ability to produce typed Deuterium queries.

        impl ::deuterium::Selectable<$model> for $table { }
        impl ::deuterium::Updatable<$model> for $table { }
        impl ::deuterium::Deletable<$model> for $table { }
        impl ::deuterium::Insertable<$model> for $table { }

    )
}

#[macro_export]
macro_rules! primary_key {
    ($s:ident, $model:ident, $body:block) => (
        impl $model {
            #[allow(dead_code)]
            pub fn lookup_predicate(&$s) -> ::deuterium::SharedPredicate {
                $body
            }
        }
        // TODO get_primary()
        // TODO get_primary_f()
    )
}

#[macro_export]
macro_rules! create_model {
    ($model:ident, $($field_name:ident: $field_value:expr),+) => (
        $model {
            $(
                $field_name: Some($field_value),
            )+

            ..std::default::Default::default()
        }
    )
}