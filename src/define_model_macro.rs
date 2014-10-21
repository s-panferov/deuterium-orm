#[macro_export]
macro_rules! define_model {
    ($model:ident, $table:ident, $table_inst:ident, $table_name:expr, [ $(($field_name:ident, $field_type:ty)),+ ]) => (

        #[deriving(Default)]
        struct $model {
            $(
                $field_name: FieldValue<$field_type>,
            )+  
        }

        struct $table;

        #[deriving(Clone)]
        struct $table_inst {
            table_name: String,
            table_alias: Option<String>
        }

        #[allow(dead_code)]
        impl $table {

            pub fn table_name() -> &'static str {
                $table_name
            }

            pub fn from() -> deuterium::RcTable {
                $table_inst {
                    table_name: $table::table_name().to_string(),
                    table_alias: None
                }.upcast()
            }

            pub fn alias(alias: &str) -> deuterium::RcTable {
                $table_inst {
                    table_name: $table::table_name().to_string(),
                    table_alias: Some(alias.to_string())
                }.upcast()
            }

            $(
                pub fn $field_name() -> NamedField<$field_type> {
                    NamedField::<$field_type>::new(stringify!($field_name))
                }
            )+   
        }

        #[allow(dead_code)]
        impl $table_inst {
            $(
                pub fn $field_name(&self) -> NamedField<$field_type> {
                    match self.table_alias.as_ref() {
                        Some(alias) => NamedField::<$field_type>::new_qual(self.table_name.as_slice(), alias.as_slice()),
                        None => NamedField::<$field_type>::new(self.table_name.as_slice())
                    }
                }
            )+  
        }

        impl deuterium::Table for $table_inst {
            fn upcast(&self) -> RcTable {
                Arc::new(box self.clone() as BoxedTable)
            }

            fn get_table_name(&self) -> &String {
                &self.table_name
            }

            fn get_table_alias(&self) -> &Option<String> {
                &self.table_alias
            }
        } 
    )
}