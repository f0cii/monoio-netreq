fn main() {
    let transports_patch = cfg!(feature = "transports-patch");
    let hyper_patch = cfg!(feature = "hyper-patch");
    let native_tls_patch = cfg!(feature = "native-tls-patch");

    let default_crate = cfg!(feature = "default-crate");
    let hyper = cfg!(feature = "hyper");
    let native_tls = cfg!(feature = "native-tls");

    if hyper && !default_crate {
        panic!("The 'hyper' feature requires the 'default-crate' flag to be enabled or use default-features");
    }

    if native_tls && !default_crate {
        panic!("The 'native-tls' feature requires the 'default-crate' flag to be enabled or use default-features");
    }

    if hyper_patch && !transports_patch {
        panic!("The 'hyper-patch' feature requires the 'transports-patch' feature to be enabled and default-features to be disabled");
    }

    if native_tls_patch && !transports_patch {
        panic!("The 'native-tls-patch' feature requires the 'transports-patch' feature to be enabled and default-features to be disabled");
    }

    if !transports_patch && !default_crate {
        panic!("atleast one of 'transports-patch'/[default-crate] flag or default-features should be enabled")
    }

    if transports_patch && default_crate {
        panic!("disable default-features when using 'transports-patch' feature to avoid any potential issues")
    }
}
