//! Enable proc-macro diagnostics by default when toolchain is set on nightly!

use warning::warn_build;
fn main() {
    warn_build();
}
