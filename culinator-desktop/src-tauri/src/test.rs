use super::*;
#[test]
fn bootstrap_serializes_expected_fields() {
    let value = ServiceBootstrap {
        endpoint: "http://127.0.0.1:1".into(),
        websocket_url: "ws://127.0.0.1:1/ws".into(),
        token: "secret".into(),
    };
    let json = serde_json::to_value(value).expect("serialize");
    assert_eq!(json["websocketUrl"], "ws://127.0.0.1:1/ws");
}

/// The menu spec is written by `useNativeMenu.ts` in camelCase; keep the two
/// shapes in step.
#[test]
fn menu_spec_reads_the_frontend_shape() {
    let sections: Vec<MenuSectionSpec> = serde_json::from_str(
        r#"[{
            "label": "Recipe",
            "enabled": true,
            "items": [
                { "id": "save", "label": "Save changes", "enabled": true,
                  "separatorBefore": true, "accelerator": "CmdOrCtrl+S" },
                { "id": "delete", "label": "Delete recipe…", "enabled": false,
                  "separatorBefore": false }
            ]
        }]"#,
    )
    .expect("deserialize menu spec");

    let items = &sections[0].items;
    assert_eq!(sections[0].label, "Recipe");
    assert!(items[0].separator_before);
    assert_eq!(items[0].accelerator.as_deref(), Some("CmdOrCtrl+S"));
    // A missing accelerator is the common case, not an error.
    assert_eq!(items[1].accelerator, None);
    assert!(!items[1].enabled);
}
