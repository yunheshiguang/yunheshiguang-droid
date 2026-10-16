use std::collections::HashMap;
use std::fs;
use std::sync::Arc;

use yunheshiguang_sdk::documents::FolderDocumentStore;
use yunheshiguang_sdk::vendor::DeviceIdentity;

use super::book::{
    GROUPS_COLLECTION, Inventory, PROFILES_COLLECTION, ProfileBook, profile_document_id,
};
use super::groups::ProfileGroups;
use super::keys::device_keys;
use super::patch::{CheckedPatch, MAX_NUMBER, MAX_TAGS, UNGROUPED_ID, check_group};
use super::section::ProfilesSection;
use crate::workspace::adb::DeviceIdentities;
use crate::workspace::contract::{
    DeviceGroupDefinition, DeviceProfile, DeviceProfilePatch, DeviceRecord, DeviceStatus,
    GroupTone, PiiClass,
};
use crate::workspace::governance::GroupDirectory;
use crate::workspace::messages::*;
use crate::workspace::snapshot::{SnapshotDraft, SnapshotSection};
use crate::workspace::text::ui_text;

fn identity(serial_number: &str) -> DeviceIdentity {
    DeviceIdentity::from_properties([("ro.serialno", serial_number)])
}

fn serials(values: &[&str]) -> Vec<String> {
    values.iter().map(ToString::to_string).collect()
}

#[test]
fn keys_devices_by_a_unique_hardware_serial_and_falls_back_otherwise() {
    let identities = HashMap::from([
        ("192.168.1.8:5555".to_string(), identity("R58M20ABCDE")),
        ("board-a".to_string(), identity("0123456789ABCDEF")),
        ("twin-1".to_string(), identity("SAME")),
        ("twin-2".to_string(), identity("SAME")),
    ]);

    let keys = device_keys(
        &serials(&[
            "192.168.1.8:5555",
            "board-a",
            "twin-1",
            "twin-2",
            "unauthorized",
        ]),
        &identities,
    );

    assert_eq!(keys["192.168.1.8:5555"], "R58M20ABCDE");
    assert_eq!(keys["board-a"], "board-a");
    assert_eq!(keys["twin-1"], "twin-1");
    assert_eq!(keys["twin-2"], "twin-2");
    assert_eq!(keys["unauthorized"], "unauthorized");
}

fn patch(configure: impl FnOnce(&mut DeviceProfilePatch)) -> DeviceProfilePatch {
    let mut patch = DeviceProfilePatch::default();
    configure(&mut patch);
    patch
}

#[test]
fn numbers_devices_in_request_order_and_clears_with_zero() {
    let numbered = CheckedPatch::check(&patch(|patch| patch.number = Some(7)), 3).unwrap();
    let mut profiles = vec![DeviceProfile::default(); 3];

    for (index, profile) in profiles.iter_mut().enumerate() {
        numbered.apply(profile, index).unwrap();
    }
    assert_eq!(
        profiles
            .iter()
            .map(|profile| profile.number)
            .collect::<Vec<_>>(),
        [7, 8, 9]
    );

    let cleared = CheckedPatch::check(&patch(|patch| patch.number = Some(0)), 1).unwrap();
    cleared.apply(&mut profiles[0], 0).unwrap();
    assert_eq!(profiles[0].number, 0);

    assert_eq!(
        CheckedPatch::check(&patch(|patch| patch.number = Some(MAX_NUMBER)), 2)
            .unwrap_err()
            .key,
        ORGANIZE_NUMBER_TOO_LARGE_KEY
    );
    assert_eq!(
        CheckedPatch::check(&DeviceProfilePatch::default(), 0).unwrap_err(),
        ui_text(ORGANIZE_NO_DEVICES_KEY)
    );
}

#[test]
fn tidies_text_and_keeps_tags_unique_regardless_of_case() {
    let checked = CheckedPatch::check(
        &patch(|patch| {
            patch.alias = Some("  A 区   07 ".to_string());
            patch.add_tags = serials(&["TikTok", "tiktok", " 美区 ", ""]);
            patch.remove_tags = serials(&["OLD"]);
        }),
        1,
    )
    .unwrap();
    let mut profile = DeviceProfile {
        tags: serials(&["old", "美区"]),
        ..DeviceProfile::default()
    };

    checked.apply(&mut profile, 0).unwrap();

    assert_eq!(profile.alias, "A 区 07");
    assert_eq!(profile.tags, ["美区", "TikTok"]);
}

#[test]
fn refuses_text_that_is_too_long_and_too_many_tags() {
    let long = |configure: fn(&mut DeviceProfilePatch)| {
        CheckedPatch::check(&patch(configure), 1).unwrap_err().key
    };

    assert_eq!(
        long(|patch| patch.alias = Some("名".repeat(33))),
        ORGANIZE_ALIAS_TOO_LONG_KEY
    );
    assert_eq!(
        long(|patch| patch.note = Some("字".repeat(201))),
        ORGANIZE_NOTE_TOO_LONG_KEY
    );
    assert_eq!(
        long(|patch| patch.add_tags = vec!["t".repeat(25)]),
        ORGANIZE_TAG_TOO_LONG_KEY
    );

    let many = CheckedPatch::check(
        &patch(|patch| patch.add_tags = (0..=MAX_TAGS).map(|index| format!("t{index}")).collect()),
        1,
    )
    .unwrap();
    assert_eq!(
        many.apply(&mut DeviceProfile::default(), 0)
            .unwrap_err()
            .key,
        ORGANIZE_TOO_MANY_TAGS_KEY
    );
}

fn group(id: &str, name: &str) -> DeviceGroupDefinition {
    DeviceGroupDefinition {
        id: id.to_string(),
        name: name.to_string(),
        tone: GroupTone::Sky.into(),
        ..Default::default()
    }
}

#[test]
fn moves_devices_only_into_groups_that_exist() {
    let inventory = Inventory {
        groups: vec![group("shelf-a", "A")],
        ..Inventory::default()
    };
    let into = |id: &str| {
        CheckedPatch::check(&patch(|patch| patch.group_id = Some(id.to_string())), 1)
            .unwrap()
            .check_group(&inventory)
    };

    assert_eq!(into("shelf-a"), Ok(()));
    assert_eq!(into(""), Ok(()));
    assert_eq!(into("gone"), Err(ui_text(ORGANIZE_GROUP_MISSING_KEY)));
}

#[test]
fn validates_group_names_and_ids() {
    assert_eq!(
        check_group(Some(group("shelf-a", "  货架   A ")))
            .unwrap()
            .name,
        "货架 A"
    );
    for id in ["all", UNGROUPED_ID, "a/b", ""] {
        assert_eq!(
            check_group(Some(group(id, "A"))).unwrap_err(),
            ui_text(ORGANIZE_GROUP_ID_INVALID_KEY),
            "{id}"
        );
    }
    assert_eq!(
        check_group(Some(group("shelf-a", "   "))).unwrap_err(),
        ui_text(ORGANIZE_GROUP_NAME_REQUIRED_KEY)
    );
}

fn book() -> (tempfile::TempDir, ProfileBook) {
    let root = tempfile::tempdir().unwrap();
    let book = ProfileBook::new(Arc::new(FolderDocumentStore::new(root.path())));

    (root, book)
}

fn profile(key: &str, alias: &str) -> DeviceProfile {
    DeviceProfile {
        device_key: key.to_string(),
        alias: alias.to_string(),
        ..DeviceProfile::default()
    }
}

#[test]
fn stores_one_document_per_device_without_the_serial_in_its_name() {
    let (root, book) = book();

    book.save_profiles(&[profile("R58M20ABCDE", "A-01")], &[])
        .unwrap();

    let id = profile_document_id("R58M20ABCDE");
    let file = root
        .path()
        .join(PROFILES_COLLECTION)
        .join(format!("{}.json", id.as_str()));
    assert!(!id.as_str().contains("R58M20ABCDE"));
    assert!(
        fs::read_to_string(file)
            .unwrap()
            .contains("\"deviceKey\":\"R58M20ABCDE\"")
    );
    assert_eq!(book.inventory().profiles["R58M20ABCDE"].alias, "A-01");
}

#[test]
fn deletes_blank_profiles_and_retired_keys() {
    let (root, book) = book();

    book.save_profiles(&[profile("usb-1", "A"), profile("R58M", "B")], &[])
        .unwrap();
    book.save_profiles(&[profile("usb-1", "")], &["R58M".to_string()])
        .unwrap();

    assert!(book.inventory().profiles.is_empty());
    assert_eq!(
        fs::read_dir(root.path().join(PROFILES_COLLECTION))
            .unwrap()
            .count(),
        0
    );
}

#[test]
fn reads_profiles_written_by_a_newer_version() {
    let (root, book) = book();
    let directory = root.path().join(PROFILES_COLLECTION);
    fs::create_dir_all(&directory).unwrap();
    fs::write(
        directory.join(format!("{}.json", profile_document_id("R58M").as_str())),
        r#"{"deviceKey":"R58M","alias":"A-01","colorLabel":"teal"}"#,
    )
    .unwrap();
    fs::write(directory.join("device-broken.json"), "{").unwrap();
    fs::write(
        directory.join("device-planted.json"),
        r#"{"deviceKey":"R58M20ABCDE","groupId":"shelf-b"}"#,
    )
    .unwrap();

    let inventory = book.inventory();

    assert_eq!(inventory.profiles.len(), 1);
    assert_eq!(inventory.profiles["R58M"].alias, "A-01");
}

#[test]
fn lists_groups_by_name_and_sees_its_own_changes_at_once() {
    let (root, book) = book();

    book.save_group(&group("b", "货架 B")).unwrap();
    assert_eq!(book.inventory().groups.len(), 1);
    book.save_group(&group("a", "货架 A")).unwrap();
    assert_eq!(
        book.inventory()
            .groups
            .iter()
            .map(|group| group.id.as_str())
            .collect::<Vec<_>>(),
        ["a", "b"]
    );

    book.remove_group("a").unwrap();
    assert_eq!(book.inventory().groups.len(), 1);
    assert!(!root.path().join(GROUPS_COLLECTION).join("a.json").exists());
}

fn device(id: &str, status: DeviceStatus) -> DeviceRecord {
    DeviceRecord {
        id: id.to_string(),
        status: status.into(),
        ..DeviceRecord::default()
    }
}

#[test]
fn labels_devices_and_counts_every_group() {
    let inventory = Inventory {
        profiles: HashMap::from([
            (
                "R58M".to_string(),
                DeviceProfile {
                    device_key: "R58M".to_string(),
                    number: 12,
                    alias: "A-12".to_string(),
                    note: "张三的账号".to_string(),
                    tags: serials(&["美区"]),
                    group_id: "shelf-a".to_string(),
                    ..Default::default()
                },
            ),
            ("usb-2".to_string(), profile("usb-2", "B-01")),
        ]),
        groups: vec![group("shelf-a", "货架 A"), group("shelf-b", "货架 B")],
    };
    let keys = HashMap::from([
        ("10.0.0.8:5555".to_string(), "R58M".to_string()),
        ("usb-2".to_string(), "HW-2".to_string()),
    ]);
    let mut draft = SnapshotDraft::new("test", "zh-CN".to_string(), "test".to_string());
    draft.add_devices(vec![
        device("10.0.0.8:5555", DeviceStatus::Online),
        device("usb-2", DeviceStatus::Offline),
    ]);

    ProfilesSection::new(&inventory, &keys).contribute(&mut draft);
    let snapshot = draft.finish();

    let labeled = &snapshot.devices[0];
    let labels = labeled.labels.as_option().unwrap();
    assert_eq!((labels.number, labels.alias.as_str()), (12, "A-12"));
    assert_eq!(labels.tags, ["美区"]);
    let note = labels.note.as_option().unwrap();
    assert_eq!(note.redacted_value, "******");
    assert_eq!(note.pii_class, PiiClass::Note);
    assert_eq!(labeled.group_id, "shelf-a");
    assert_eq!(labeled.group.as_option().unwrap().fallback, "货架 A");

    let fallback = &snapshot.devices[1];
    assert_eq!(fallback.labels.as_option().unwrap().alias, "B-01");
    assert_eq!(fallback.group_id, UNGROUPED_ID);

    let summary = snapshot
        .groups
        .iter()
        .map(|group| (group.id.as_str(), group.total, group.online))
        .collect::<Vec<_>>();
    assert_eq!(
        summary,
        [("shelf-a", 1, 1), ("shelf-b", 0, 0), (UNGROUPED_ID, 1, 0)]
    );
}

#[test]
fn leaves_out_the_ungrouped_bucket_when_every_device_has_a_group() {
    let inventory = Inventory::default();
    let keys = HashMap::new();
    let mut draft = SnapshotDraft::new("test", "zh-CN".to_string(), "test".to_string());

    ProfilesSection::new(&inventory, &keys).contribute(&mut draft);

    assert!(draft.finish().groups.is_empty());
}

#[test]
fn tells_governance_which_groups_the_target_devices_are_in() {
    let (_root, book) = book();
    book.save_group(&group("shelf-a", "A")).unwrap();
    book.save_profiles(
        &[
            DeviceProfile {
                group_id: "shelf-a".to_string(),
                ..profile("R58M20ABCDE", "")
            },
            DeviceProfile {
                group_id: "gone".to_string(),
                ..profile("usb-2", "")
            },
        ],
        &[],
    )
    .unwrap();
    let identities = Arc::new(DeviceIdentities::default());
    identities.remember("192.168.1.8:5555", identity("R58M20ABCDE"));
    let groups = ProfileGroups {
        book: Arc::new(book),
        identities,
    };

    assert_eq!(
        groups.groups_of(&serials(&["192.168.1.8:5555"])),
        ["shelf-a"],
        "found by the hardware serial the snapshot already read"
    );
    assert_eq!(
        groups.groups_of(&serials(&["192.168.1.8:5555", "usb-2", "new"])),
        ["shelf-a", UNGROUPED_ID],
        "devices outside any existing group count as ungrouped"
    );
    assert!(groups.groups_of(&[]).is_empty());
}
