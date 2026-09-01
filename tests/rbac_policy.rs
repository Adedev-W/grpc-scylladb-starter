use grpc_scylladb_starter::application::{Action, role_allows};

#[test]
fn rbac_policy_is_easy_to_read() {
    let cases = [
        ("reader", Action::Read, true, "reader can read channels"),
        (
            "reader",
            Action::Delete,
            false,
            "reader cannot delete channels",
        ),
        ("writer", Action::Create, true, "writer can create channels"),
        (
            "writer",
            Action::Delete,
            false,
            "writer cannot delete channels",
        ),
        ("admin", Action::Delete, true, "admin can delete channels"),
        (
            "unknown",
            Action::Read,
            false,
            "subjects without a role are denied",
        ),
    ];

    for (role, action, expected, explanation) in cases {
        let actual = role_allows(role, action);
        println!(
            "[RBAC] role={role:<7} action={action:?} result={actual:<5} expected={expected:<5} | {explanation}"
        );
        assert_eq!(actual, expected, "{explanation}");
    }
}
