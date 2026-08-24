use grpc_scylladb_starter::application::{Action, role_allows};

#[test]
fn rbac_policy_is_easy_to_read() {
    let cases = [
        ("reader", Action::Read, true, "reader dapat membaca channel"),
        (
            "reader",
            Action::Delete,
            false,
            "reader tidak dapat menghapus channel",
        ),
        (
            "writer",
            Action::Create,
            true,
            "writer dapat membuat channel",
        ),
        (
            "writer",
            Action::Delete,
            false,
            "writer tidak dapat menghapus channel",
        ),
        (
            "admin",
            Action::Delete,
            true,
            "admin dapat menghapus channel",
        ),
        ("unknown", Action::Read, false, "subject tanpa role ditolak"),
    ];

    for (role, action, expected, explanation) in cases {
        let actual = role_allows(role, action);
        println!(
            "[RBAC] role={role:<7} action={action:?} result={actual:<5} expected={expected:<5} | {explanation}"
        );
        assert_eq!(actual, expected, "{explanation}");
    }
}
