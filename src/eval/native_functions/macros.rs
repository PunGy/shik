#[macro_export]
macro_rules! native_op {
    ($name:ident, $fn_title:tt, [$($arg:ident),* $(,)?] $(, $ctx:ident)? , $body:block) => {
        #[derive(Debug)]
        pub struct $name;

        impl NativeFn for $name {
            #[allow(unused_variables)]
            fn exec(&self, args: &Vec<ValueRef>, __native_ctx: &NativeContext) -> EvalResult {
                let args_required = $crate::count_args!($($arg),*);
                if args.len() != args_required {
                    return Err(RuntimeError::invalid_application(format!("({:?}) wrong number of arguments. Expected {}", $fn_title, args_required)));
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
                    )));
                    $crate::native_op!(@define_titles env, val, $fn_title);
                }
                pub fn define_help(env: &EnvRef, msg: &str) {
                    $crate::native_op!(@define_help env, "native-lambda", msg, $fn_title);
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

    // Define help for multiple names
    (@define_help $env:ident, $kind:expr, $msg:expr, [$($title:expr),+ $(,)?]) => {
        {
            let names: Vec<&str> = vec![$($title),+];
            let names_str = names.join(", ");
            let formatted = format!("{}: {}\n{}", $kind, names_str, $msg);
            $(
                $env.define_help(($title).to_string(), formatted.clone());
            )+
        }
    };

    // Define help for single name
    (@define_help $env:ident, $kind:expr, $msg:expr, $title:expr) => {
        {
            let formatted = format!("{}: {}\n{}", $kind, $title, $msg);
            $env.define_help(($title).to_string(), formatted);
        }
    };
}

#[macro_export]
macro_rules! special_b_op {
    ($name:ident, $fn_title:tt, [$($arg:ident),* $(,)?] $(, $ctx:ident)? , $body:block) => {
        #[derive(Debug)]
        pub struct $name;

        impl SpecialFn for $name {
            #[allow(unused_variables)]
            fn exec(&self, args: &Vec<Expression>, __native_ctx: &NativeContext) -> EvalResult {
                let args_required = $crate::count_args!($($arg),*);
                if args.len() != args_required {
                    return Err(RuntimeError::invalid_application(format!("({:?}) wrong number of arguments. Expected {}", $fn_title, args_required)));
                }

                #[allow(unused_mut)]
                let mut iter = args.iter();
                $(let $arg: &Expression = iter.next().unwrap();)*

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
                    $($arg: &Expression),*
                    $(, $ctx: &NativeContext)?
                ) -> EvalResult {
                    $body
                }

                pub fn define(env: &EnvRef, inter: Rc<Interpretator>) {
                    let val = Rc::new(Value::SpecialBoundForm(SpecialBoundClosure::new(
                        $crate::count_args!($($arg),*),
                        Rc::new($name),
                        inter,
                    )));
                    $crate::special_b_op!(@define_titles env, val, $fn_title);
                }

                pub fn define_help(env: &EnvRef, msg: &str) {
                    $crate::special_b_op!(@define_help env, "special-lambda", msg, $fn_title);
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

    // Define help for multiple names
    (@define_help $env:ident, $kind:expr, $msg:expr, [$($title:expr),+ $(,)?]) => {
        {
            let names: Vec<&str> = vec![$($title),+];
            let names_str = names.join(", ");
            let formatted = format!("{}: {}\n{}", $kind, names_str, $msg);
            $(
                $env.define_help(($title).to_string(), formatted.clone());
            )+
        }
    };

    // Define help for single name
    (@define_help $env:ident, $kind:expr, $msg:expr, $title:expr) => {
        {
            let formatted = format!("{}: {}\n{}", $kind, $title, $msg);
            $env.define_help(($title).to_string(), formatted);
        }
    };
}

#[macro_export]
macro_rules! special_op {
    // Single name variant
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
                    ))),
                );
            }

            pub fn define_help(env: &EnvRef, msg: &str) {
                let formatted = format!("{}: {}\n{}", "special-form", $fn_title, msg);
                env.define_help(($fn_title).to_string(), formatted);
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
macro_rules! define_help {
    ($name:ident, $env:ident, $msg:expr) => {
        $name::define_help(&$env, $msg);
    };
}

#[macro_export]
macro_rules! count_args {
    () => { 0 };
    ($head:ident $(, $tail:ident)*) => { 1 + count_args!($($tail),*) };
}
