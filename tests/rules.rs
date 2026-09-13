use camino::{Utf8Path, Utf8PathBuf};
use qubit_dependency_policy::{
    BaselineRef, Profile, ProjectConfig, ProjectSettings, evaluate, load_baseline,
};

fn config(project: &Utf8Path) -> ProjectConfig {
    ProjectConfig {
        format: 1,
        baseline: BaselineRef {
            source: format!(
                "file://{}",
                Utf8Path::new("tests/fixtures/policy-repo")
                    .canonicalize_utf8()
                    .expect("policy fixture")
            ),
            revision: "0123456789abcdef0123456789abcdef01234567".into(),
            release: "v2026.09.0".into(),
            name: "test".into(),
        },
        project: ProjectSettings {
            profile: if project.ends_with("application-no-lock") {
                Profile::Application
            } else {
                Profile::Library
            },
            exceptions_path: None,
        },
    }
}

#[test]
fn reports_a_direct_num_bigint_version_drift() {
    let project = Utf8PathBuf::from("tests/fixtures/num-bigint-05");
    let config = config(&project);
    let baseline =
        load_baseline(&config.baseline, Utf8Path::new("target/t3/cache")).expect("baseline");
    let evaluation = evaluate(&project, &config, &baseline).expect("evaluation");
    assert!(
        evaluation
            .violations
            .iter()
            .any(|item| item.code == "DP202")
    );
}

#[test]
fn reports_a_missing_application_lockfile() {
    let project = Utf8PathBuf::from("tests/fixtures/application-no-lock");
    let config = config(&project);
    let baseline =
        load_baseline(&config.baseline, Utf8Path::new("target/t3/cache")).expect("baseline");
    let evaluation = evaluate(&project, &config, &baseline).expect("evaluation");
    assert!(
        evaluation
            .violations
            .iter()
            .any(|item| item.code == "DP201")
    );
}
