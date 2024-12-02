macro_rules! inner_set_env {
    ($builder:ident) => {
        $builder
    };

    ($builder:ident, $key:ident:$value:expr $(,$key_tail:ident:$value_tail:expr)*) => {
        {
           $builder.context.$key = $value.try_into().unwrap();
           inner_set_env!($builder $(,$key_tail:$value_tail)*)
        }
    };
}

macro_rules! set_env {
    ($($key:ident:$value:expr),* $(,)?) => {
        let mut builder = near_sdk::test_utils::VMContextBuilder::new();
        let mut builder = &mut builder;
        builder = inner_set_env!(builder, $($key: $value),*);
        near_sdk::testing_env!(builder.build());
    };
}
