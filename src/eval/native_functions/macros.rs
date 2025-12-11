#[macro_export]
macro_rules! native_op {
    ($name:ident, $fn_title:expr, [$($arg:ident),*], $ctx:ident, $body:block) => {
        #[derive(Debug)]
        pub struct $name;

        impl NativeFn for $name {
            #[allow(unused_variables)]
            fn exec(&self, args: &Vec<ValueRef>, $ctx: &NativeContext) -> EvalResult {
                if args.len() != count_args!($($arg),*) {
                    return Err(RuntimeError::InvalidApplication)
                }
                #[allow(unused_mut)]
                let mut iter = args.iter();
                $(let $arg = iter.next().unwrap();)*

                $body
            }
        }

        impl $name {
            pub fn define(env: &EnvRef, inter: Rc<Interpretator>) {
                env.define(
                    ($fn_title).to_string(),
                    Rc::new(Value::NativeLambda(NativeClosure::new(
                        count_args!($($arg),*),
                        Rc::new($name),
                        inter,
                        Rc::clone(env),
                    ))),
                );
            }
        }
    };
    // Pattern without ctx for simple functions that don't need it
    // Pattern: name, [arg1, arg2...], body
    ($name:ident, $fn_title:expr, [$($arg:ident),*], $body:block) => {
        #[derive(Debug)]
        pub struct $name;

        impl NativeFn for $name {
            #[allow(unused_variables)]
            fn exec(&self, args: &Vec<ValueRef>, ctx: &NativeContext) -> EvalResult {
                if args.len() != count_args!($($arg),*) {
                    return Err(RuntimeError::InvalidApplication)
                }
                #[allow(unused_mut)]
                let mut iter = args.iter();
                $(let $arg = iter.next().unwrap();)*

                $body
            }
        }

        impl $name {
            pub fn define(env: &EnvRef, inter: Rc<Interpretator>) {
                env.define(
                    ($fn_title).to_string(),
                    Rc::new(Value::NativeLambda(NativeClosure::new(
                        count_args!($($arg),*),
                        Rc::new($name),
                        inter,
                        Rc::clone(env),
                    ))),
                );
            }
        }
    };
}

#[macro_export]
macro_rules! special_op {
    ($name:ident, $fn_title:expr, $args:ident, $ctx:ident, $body:block) => {
        #[derive(Debug)]
        pub struct $name;

        impl SpecialFn for $name {
            fn exec(&self, $args: &Vec<Expression>, $ctx: &NativeContext) -> EvalResult {
                $body
            }
        }

        impl $name {
            pub fn define(env: &EnvRef, inter: Rc<Interpretator>) {
                env.define(
                    ($fn_title).to_string(),
                    Rc::new(Value::SpecialForm(SpecialClosure::new(
                        Rc::new($name),
                        inter,
                        Rc::clone(env),
                    ))),
                );
            }
        }
    };
}

#[macro_export]
macro_rules! define_native {
    ($name:ident, $env:ident, $inter:ident) => {
        $name::define(&$env, Rc::clone(&$inter));
    };
}

#[macro_export]
macro_rules! count_args {
    () => { 0 };
    ($head:ident $(, $tail:ident)*) => { 1 + count_args!($($tail),*) };
}
