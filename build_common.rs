// The following code is modified from embassy-stm32
// https://github.com/embassy-rs/embassy/tree/main/embassy-stm32
// Special thanks to the Embassy Project and its contributors for their work!

// NOTE: this file is copy-pasted between several Embassy crates, because there is no
// straightforward way to share this code:
// - it cannot be placed into the root of the repo and linked from each build.rs using `#[path =
// "../build_common.rs"]`, because `cargo publish` requires that all files published with a crate
// reside in the crate's directory,
// - it cannot be symlinked from `embassy-xxx/build_common.rs` to `../build_common.rs`, because
// symlinks don't work on Windows.

use std::collections::HashSet;
use std::env;

/// Helper for emitting cargo instruction for enabling configs (`cargo:rustc-cfg=X`) and declaring
/// them (`cargo:rust-check-cfg=cfg(X)`).
#[derive(Debug)]
pub struct CfgSet {
    enabled: HashSet<String>,
    declared: HashSet<String>,
}

impl CfgSet {
    pub fn new() -> Self {
        Self {
            enabled: HashSet::new(),
            declared: HashSet::new(),
        }
    }

    /// Enable a config, which can then be used in `#[cfg(...)]` for conditional compilation.
    ///
    /// All configs that can potentially be enabled should be unconditionally declared using
    /// [`Self::declare()`].
    pub fn enable(&mut self, cfg: impl AsRef<str>) {
        if self.enabled.insert(cfg.as_ref().to_owned()) {
            println!("cargo:rustc-cfg={}", cfg.as_ref());
        }
    }

    pub fn enable_all(&mut self, cfgs: &[impl AsRef<str>]) {
        for cfg in cfgs.iter() {
            self.enable(cfg.as_ref());
        }
    }

    /// Declare a valid config for conditional compilation, without enabling it.
    ///
    /// This enables rustc to check that the configs in `#[cfg(...)]` attributes are valid.
    pub fn declare(&mut self, cfg: impl AsRef<str>) {
        if self.declared.insert(cfg.as_ref().to_owned()) {
            println!("cargo:rustc-check-cfg=cfg({})", cfg.as_ref());
        }
    }

    pub fn declare_all(&mut self, cfgs: &[impl AsRef<str>]) {
        for cfg in cfgs.iter() {
            self.declare(cfg.as_ref());
        }
    }

    pub fn set(&mut self, cfg: impl Into<String>, enable: bool) {
        let cfg = cfg.into();
        if enable {
            self.enable(cfg.clone());
        }
        self.declare(cfg);
    }
}

/// Sets configs that describe the target platform.
pub fn set_target_cfgs(cfgs: &mut CfgSet) {
    let target = env::var("TARGET").unwrap();

    if target.starts_with("thumbv6m-") {
        cfgs.enable_all(&["cortex_m", "armv6m"]);
    } else if target.starts_with("thumbv7m-") {
        cfgs.enable_all(&["cortex_m", "armv7m"]);
    } else if target.starts_with("thumbv7em-") {
        cfgs.enable_all(&["cortex_m", "armv7m", "armv7em"]);
    } else if target.starts_with("thumbv8m.base") {
        cfgs.enable_all(&["cortex_m", "armv8m", "armv8m_base"]);
    } else if target.starts_with("thumbv8m.main") {
        cfgs.enable_all(&["cortex_m", "armv8m", "armv8m_main"]);
    }
    cfgs.declare_all(&[
        "cortex_m",
        "armv6m",
        "armv7m",
        "armv7em",
        "armv8m",
        "armv8m_base",
        "armv8m_main",
    ]);

    cfgs.set("has_fpu", target.ends_with("-eabihf"));
}
