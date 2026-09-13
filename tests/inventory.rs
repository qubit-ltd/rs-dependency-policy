use camino::Utf8PathBuf;
use qubit_dependency_policy::{render_inventory_markdown, scan_projects};

#[test]
fn scans_direct_requirements_and_resolved_graph() {
    let inventory =
        scan_projects(&[Utf8PathBuf::from("tests/fixtures/num-bigint-05")]).expect("inventory");
    assert_eq!(inventory.projects.len(), 1);
    assert!(inventory.direct_requirements.contains_key("num-bigint"));
    assert!(
        inventory.projects[0]
            .resolved
            .iter()
            .any(|p| p.name == "num-bigint")
    );
    assert!(render_inventory_markdown(&inventory).contains("num-bigint"));
}

#[test]
fn reports_requirement_conflicts_across_projects() {
    let inventory = scan_projects(&[
        Utf8PathBuf::from("tests/fixtures/num-bigint-05"),
        Utf8PathBuf::from("tests/fixtures/num-bigint-04"),
    ])
    .expect("inventory");
    assert!(inventory.conflicts.contains_key("num-bigint"));
}
