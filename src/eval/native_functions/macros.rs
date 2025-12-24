#[macro_export]
macro_rules! native_op {
    ($name:ident, $fn_title:tt, [$($arg:ident),* $(,)?] $(, $ctx:ident)? , $body:block) => {
        #[derive(Debug)]
        pub struct $name;

        impl NativeFn for $name {
            #[allow(unused_variables)]
            fn exec(&self, args: &Vec<ValueRef>, __native_ctx: &NativeContext) -> EvalResult {
                if args.len() != $crate::count_args!($($arg),*) {
                    return Err(RuntimeError::InvalidApplication);
                }

                #[allow(unused_mut)]
                let mut iter = args.iter();
                $(let $arg: &ValueRef = iter.next().unwrap();)*

                $crate::native_op!(@bind_ctx __native_ctx $(, $ctx)?);

                paste::paste! {
                    Self::run($($arg),* $(, $ctx)?)
                }
            }
        }

        paste::paste! {
            impl $name {
                // This is where the user-provided $body goes.
                pub fn run(
                    $($arg: &ValueRef),*
                    $(, $ctx: &NativeContext)?
                ) -> EvalResult {
                    $body
                }

                pub fn define(env: &EnvRef, inter: Rc<Interpretator>) {
                    let val = Rc::new(Value::NativeLambda(NativeClosure::new(
                        $crate::count_args!($($arg),*),
                        Rc::new($name),
                        inter,
                        Rc::clone(env),
                    )));
                    $crate::native_op!(@define_titles env, val, $fn_title);
                }
            }
        }
    };

    (@bind_ctx $native_ctx:ident, $ctx:ident) => { let $ctx = $native_ctx; };
    (@bind_ctx $native_ctx:ident) => {};

    (@define_titles $env:ident, $val:ident, [$($title:expr),+ $(,)?]) => {
        $(
            $env.define(($title).to_string(), Rc::clone(&$val));
        )+
    };

    // One title SECOND (more general)
    (@define_titles $env:ident, $val:ident, $title:expr) => {
        $env.define(($title).to_string(), Rc::clone(&$val));
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
