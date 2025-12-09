#[macro_export]
macro_rules! native_op {
    // Pattern: name, [arg1, arg2...], body
    ($name:ident, $fn_title:expr, [$($arg:ident),*], $body:expr) => {
        #[derive(Debug)]
        pub struct $name;

        impl NativeFn for $name {
            fn exec(&self, args: &Vec<ValueRef>) -> EvalResult {
                if args.len() != count_args!($($arg),*) {
                    return Err(RuntimeError::InvalidApplication)
                }
                let mut iter = args.iter();
                $(let $arg = iter.next().unwrap();)*

                $body
            }

        }

        impl $name {
            pub fn define(env: &EnvRef) {
                env.define(
                    ($fn_title).to_string(),
                    Rc::new(Value::NativeLambda(NativeClosure::new(
                        count_args!($($arg),*),
                        Rc::new($name),
                    ))),
                );
            }
        }
    };
}

#[macro_export]
macro_rules! count_args {
    () => { 0 };
    ($head:ident $(, $tail:ident)*) => { 1 + count_args!($($tail),*) };
}
