#![expect(clippy::unwrap_used, reason = "Unwrap allowed in tests")]

use deadlock_api::model::Rank;
use deadlock_api::rank::{Badge, BadgeRange, MAX_SUBRANK, MAX_TIER, badge_label};

const RANKS: &str = include_str!("fixtures/ranks.json");

fn ranks() -> Vec<Rank> {
    serde_json::from_str(RANKS).unwrap()
}

#[test]
fn new_enforces_the_tier_and_subrank_bounds() {
    assert_eq!(Badge::new(5, 3).unwrap().get(), 53);
    assert_eq!(Badge::new(MAX_TIER, MAX_SUBRANK).unwrap().get(), 116);
    assert!(Badge::new(MAX_TIER + 1, 1).is_none());
    assert!(Badge::new(5, 0).is_none());
    assert!(Badge::new(5, MAX_SUBRANK + 1).is_none());
}

#[test]
fn from_raw_accepts_only_decodable_badges() {
    assert_eq!(Badge::from_raw(0), Some(Badge::UNRANKED));
    assert!(!Badge::UNRANKED.is_ranked());
    assert!(Badge::from_raw(57).is_none());
    assert!(Badge::from_raw(120).is_none());

    let top = Badge::from_raw(116).unwrap();
    assert_eq!((top.tier(), top.subrank()), (11, 6));
    assert!(top.is_ranked());
}

#[test]
fn a_tier_range_spans_its_six_subranks() {
    let range = BadgeRange::tier(7).unwrap();

    assert_eq!((range.min, range.max), (70, 76));
    assert!(BadgeRange::tier(MAX_TIER + 1).is_none());
}

#[test]
fn labels_use_the_tier_name_when_known() {
    let ranks = ranks();

    assert_eq!(badge_label(&ranks, Badge::UNRANKED), "Unranked");
    assert_eq!(badge_label(&ranks, Badge::new(1, 4).unwrap()), "Initiate 4");
}

#[test]
fn labels_fall_back_to_the_tier_number() {
    assert_eq!(badge_label(&[], Badge::new(9, 2).unwrap()), "Tier 9 2");
}
