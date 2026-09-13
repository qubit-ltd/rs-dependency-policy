// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use num_bigint::BigInt;

/// Returns a fixed integer used by dependency-resolution tests.
pub fn fixture_value() -> BigInt {
    1.into()
}
