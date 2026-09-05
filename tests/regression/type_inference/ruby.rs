//! Ruby type inference tests.

use cce_relation::index::{FileIndexOps, RelationQueryOps};

use crate::helper::{TestFixture, init_minimal_logging};

use super::common::{run_index, snapshot_for};

#[test]
fn test_ruby_constructors_snapshot() {
    use cce_e2e_tests::type_inference_assert::assert_variable_has_type;

    init_minimal_logging();
    let fixture =
        TestFixture::ruby_type_inference_constructors().expect("Failed to load ruby fixture");
    let bindings = snapshot_for(&fixture);

    assert_variable_has_type(&bindings, "user", "User");
    assert_variable_has_type(&bindings, "calc", "Calculator");
}

#[tokio::test]
async fn test_ruby_cross_file_type_inference() {
    init_minimal_logging();
    let fixture = TestFixture::ruby_type_inference_cross_file()
        .expect("Failed to load ruby cross_file fixture");
    let orchestrator = run_index(fixture, vec!["rb".to_string()]).await;
    let relation_index = orchestrator
        .get_relation_index()
        .expect("Relation index should be available");
    assert!(relation_index.file_count() >= 1);
    assert!(relation_index.resolved_relation_count() >= 1);
}
