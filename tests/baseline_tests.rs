// =============================================================================
//    Copyright (c) 2025 - 2026 Haixing Hu.
//
//    SPDX-License-Identifier: Apache-2.0
//
//    Licensed under the Apache License, Version 2.0.
// =============================================================================

use camino::Utf8Path;
use qubit_dependency_policy::BaselineRef;
use qubit_dependency_policy::Profile;
use qubit_dependency_policy::load_baseline;

fn fixture_source(name: &str) -> String {
    let path = Utf8Path::new("tests/fixtures").join(name);
    format!("file://{}", path.canonicalize_utf8().expect("fixture path"))
}

#[test]
fn loads_a_versioned_local_baseline() {
    let reference = BaselineRef {
        source: fixture_source("policy-repo"),
        revision: "0123456789abcdef0123456789abcdef01234567".into(),
        release: "v2026.09.0".into(),
        name: "test".into(),
    };
    let baseline = load_baseline(&reference, Utf8Path::new("target/t2/cache"))
        .expect("fixture baseline should load");
    assert_eq!(baseline.baseline.release, "v2026.09.0");
    assert!(baseline.baseline.profile(Profile::Library).is_some());
}

#[test]
fn rejects_an_unknown_release() {
    let reference = BaselineRef {
        source: fixture_source("policy-repo"),
        revision: "0123456789abcdef0123456789abcdef01234567".into(),
        release: "v2099.01.0".into(),
        name: "test".into(),
    };
    let error = load_baseline(&reference, Utf8Path::new("target/t2/cache"))
        .expect_err("unknown release should fail");
    assert_eq!(error.code(), "DP102");
}
