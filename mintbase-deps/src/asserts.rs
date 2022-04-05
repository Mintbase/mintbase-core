#[macro_export]
macro_rules! near_panic {
    ($msg:literal) => {
        near_sdk::env::panic_str($msg)
    };

    ($msg:literal, $($arg:expr),*) => {
        near_sdk::env::panic_str(&format!($msg, $($arg),*))
    };
}

#[macro_export]
macro_rules! near_assert {
    ($predicate:expr, $msg:literal) => {
        if !$predicate {
            $crate::near_panic!($msg)
        }
    };

    ($predicate:expr, $msg:literal, $($arg:expr),*) => {
        if !$predicate {
            $crate::near_panic!($msg, $($arg),*)
        }
    };
}

#[macro_export]
macro_rules! near_assert_eq {
    ($a:expr, $b:expr ,$msg:literal) => {
        if $a != $b {
            $crate::near_panic!($msg)
        }
    };

    ($a:expr, $b:expr ,$msg:literal, $($arg:expr),*) => {
        if $a != $b {
            $crate::near_panic!($msg, $($arg),*)
        }
    };
}

#[macro_export]
macro_rules! near_assert_ne {
    ($a:expr, $b:expr ,$msg:literal) => {
        if $a == $b {
            $crate::near_panic!($msg)
        }
    };

    ($a:expr, $b:expr ,$msg:literal, $($arg:expr),*) => {
        if $a == $b {
            $crate::near_panic!($msg, $($arg),*)
        }
    };
}

// TODO: near_assert_range

// ------------- specific asserts for mintbase smart contracts -------------- //

// Theoretically a duplicate for `near_sdk::assert_one_yocto`, but this version
// uses `near_sdk_env::panic`, where as `near_sdk::assert_one_yocto` uses rusts
// builtin `panic!` macro.
#[macro_export]
macro_rules! assert_yocto_deposit {
    () => {
        if env::attached_deposit() != 1 {
            $crate::near_panic!("Requires attached deposit of exactly 1 yoctoNEAR")
        }
    };
}

// full-macro panics which generate larger code but give src locations
// We can shave ~1% off contract size by moving these into methods on `Token`

#[macro_export]
macro_rules! assert_token_owned_by {
    ($token:expr, $account:expr) => {
        if !$token.is_owned_by($account) {
            $crate::near_panic!(
                "{} is required to own token {} ({}, {}:{})",
                $account,
                $token.id,
                file!(),
                line!(),
                column!()
            );
        }
    };
}

#[macro_export]
macro_rules! assert_token_owned_by_predecessor {
    ($token:expr) => {
        $crate::assert_token_owned_by!($token, &$crate::near_sdk::env::predecessor_account_id())
    };
}

#[macro_export]
macro_rules! assert_token_owned_or_approved {
    ($token:expr, $account:expr, $approval_id:expr) => {
        if !$token.is_owned_by($account) {
            let src = format!("{}, {}:{}", file!(), line!(), column!());
            match ($token.approvals.get($account), $approval_id) {
                (_, None) => {
                    $crate::near_panic!("Disallowing approvals without approval ID! ({})", src)
                },
                (None, _) => {
                    $crate::near_panic!(
                        "{} has no approval for token {} ({})",
                        $account,
                        $token.id,
                        src
                    )
                },
                (Some(a), Some(b)) if *a != b => {
                    $crate::near_panic!(
                        "The current approval ID is {}, but {} has been provided ({})",
                        a,
                        b,
                        src
                    )
                },
                _ => { /* everything ok */ },
            }
        }
    };
}

#[macro_export]
macro_rules! assert_token_unloaned {
    ($token:expr) => {
        if $token.is_loaned() {
            $crate::near_panic!(
                "Token {} must not be loaned ({}, {}:{})",
                $token.id,
                file!(),
                line!(),
                column!()
            );
        }
    };
}

#[macro_export]
macro_rules! assert_storage_deposit {
    ($required:expr) => {
        if env::attached_deposit() < $required {
            $crate::near_panic!(
                "Requires storage deposit of at least {} yoctoNEAR ({}, {}:{})",
                $required,
                file!(),
                line!(),
                column!()
            );
        }
    };
}

#[macro_export]
macro_rules! assert_payment_deposit {
    ($required:expr) => {
        if env::attached_deposit() < $required {
            $crate::near_panic!(
                "Requires payment of at least {} yoctoNEAR ({}, {}:{})",
                $required,
                file!(),
                line!(),
                column!()
            );
        }
    };
}

#[macro_export]
macro_rules! assert_payment_deposit_eq {
    ($required:expr) => {
        if env::attached_deposit() != $required {
            $crate::near_panic!(
                "Requires payment of exactly {} yoctoNEAR ({}, {}:{})",
                $required,
                file!(),
                line!(),
                column!()
            );
        }
    };
}
