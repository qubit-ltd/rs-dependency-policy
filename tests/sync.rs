use std::collections::BTreeMap;

use camino::Utf8Path;
use qubit_dependency_policy::{DirectRule, ProfileRules, plan_sync};
use semver::VersionReq;

#[test]
fn plans_num_bigint_version_replacement_without_writing() {
    let mut direct = BTreeMap::new();
    direct.insert(
        "num-bigint".into(),
        DirectRule {
            requirement: VersionReq::parse("0.4.8").expect("valid requirement"),
        },
    );
    let plan = plan_sync(
        Utf8Path::new("tests/fixtures/sync-version-only"),
        &ProfileRules {
            direct,
            resolved: BTreeMap::new(),
        },
    )
    .expect("sync plan");
    assert_eq!(plan.manifest_edits.len(), 1);
    assert_eq!(plan.manifest_edits[0].dependency, "num-bigint");
    assert!(plan.manifest_edits[0].new.contains("0.4.8"));
}

#[test]
fn blocks_inline_dependency_declarations() {
    let mut direct = BTreeMap::new();
    direct.insert(
        "num-bigint".into(),
        DirectRule {
            requirement: VersionReq::parse("0.4.8").expect("valid requirement"),
        },
    );
    let plan = plan_sync(
        Utf8Path::new("tests/fixtures/sync-feature-conflict"),
        &ProfileRules {
            direct,
            resolved: BTreeMap::new(),
        },
    )
    .expect("sync plan");
    assert_eq!(plan.blocked[0].code, "DP301");
}
