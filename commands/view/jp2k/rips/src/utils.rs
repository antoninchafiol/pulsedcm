macro_rules! var_args {
    ($func:path, args => [$($arg:expr,)*], opts => [], term => $term:expr) => {{
        $func($($arg,)* $term)
    }};
    ($func:path, args => [$($arg:expr,)*], opts => [($bind_k_0:pat, $bind_v_0:expr, $key_0:expr, $val_0:expr), $(($bind_k:pat, $bind_v:expr, $key:expr, $val:expr),)*], term => $term:expr) => {{
        if let Some($bind_k_0) = $val_0 {
            var_args!($func, args => [$($arg,)* $key_0, $bind_v_0,], opts => [$(($bind_k, $bind_v, $key, $val),)*], term => $term)
        } else {
            var_args!($func, args => [$($arg,)*], opts => [$(($bind_k, $bind_v, $key, $val),)*], term => $term)
        }
    }};
}
