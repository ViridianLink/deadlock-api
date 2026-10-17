#![expect(clippy::unwrap_used, reason = "Unwrap allowed in tests")]

use deadlock_api::model::{
    AbilityOrderStats,
    AccountRank,
    Build,
    Hero,
    HeroCounterStats,
    Item,
    ItemFlowStats,
    ItemKind,
    LaneMatchupStats,
    LaneSoulCurve,
    MatchMetadata,
    PatchNote,
    Rank,
    StatValue,
};
use serde::Serialize;
use serde::de::DeserializeOwned;

const HEROES: &str = include_str!("fixtures/heroes.json");
const ITEMS: &str = include_str!("fixtures/items.json");
const RANKS: &str = include_str!("fixtures/ranks.json");
const PATCHES: &str = include_str!("fixtures/patches_v2.json");
const ITEM_FLOW: &str = include_str!("fixtures/item_flow.json");
const ABILITY_ORDER: &str = include_str!("fixtures/ability_order.json");
const HERO_COUNTER: &str = include_str!("fixtures/hero_counter.json");
const LANE_MATCHUP: &str = include_str!("fixtures/lane_matchup.json");
const LANE_SOUL_CURVE: &str = include_str!("fixtures/lane_soul_curve.json");
const BUILDS: &str = include_str!("fixtures/builds.json");
const RANK_BATCH: &str = include_str!("fixtures/rank_batch.json");
const METADATA_BULK: &str = include_str!("fixtures/metadata_bulk.json");

const HERO_KIT: &str = include_str!("fixtures/hero_kit.json");

const FLAME_DASH: u32 = 3_516_947_824;
const INFERNUS_WEAPON: u32 = 556_186_193;
const SOUL_URN: u32 = 2_521_299_219;
const KNOWN_PLAYER: u32 = 35_803_007;

fn decode<T: DeserializeOwned>(json: &str) -> T {
    serde_json::from_str(json).unwrap()
}

fn assert_round_trips<T: Serialize + DeserializeOwned>(json: &str) {
    let first: T = decode(json);
    let encoded = serde_json::to_string(&first).unwrap();
    let second: T = decode(&encoded);

    assert_eq!(serde_json::to_string(&second).unwrap(), encoded);
}

#[test]
fn heroes_decode_with_their_flags_and_images() {
    let heroes: Vec<Hero> = decode(HEROES);

    let infernus = heroes.iter().find(|hero| hero.id == 1).unwrap();
    assert_eq!(infernus.name, "Infernus");
    assert_eq!(infernus.class_name, "hero_inferno");
    assert!(infernus.player_selectable && !infernus.disabled);
    assert!(infernus.images.icon_hero_card.is_some());
}

#[test]
fn hero_kits_decode_stats_and_slots() {
    let heroes: Vec<Hero> = decode(HEROES);

    let infernus = heroes.iter().find(|hero| hero.id == 1).unwrap();
    let health = infernus.starting_stats.get("max_health").unwrap();
    assert!(matches!(health.value, StatValue::Number(_)));
    assert_eq!(
        infernus.items.get("signature2").map(String::as_str),
        Some("ability_flame_dash")
    );
    assert!(!infernus.standard_level_up_upgrades.is_empty());
}

#[test]
fn abilities_decode_properties_upgrades_and_owner() {
    let items: Vec<Item> = decode(HERO_KIT);

    let dash = items.iter().find(|item| item.id == FLAME_DASH).unwrap();
    assert_eq!(dash.hero, Some(1));
    assert_eq!(dash.kind, ItemKind::Ability);
    assert_eq!(
        dash.properties.get("AbilityCooldown").unwrap().label.as_deref(),
        Some("Cooldown")
    );
    assert_eq!(
        dash.properties.get("DashSpeed").unwrap().value,
        Some(StatValue::Text("12m".to_owned()))
    );
    assert_eq!(dash.upgrades.len(), 3);
    let first = dash.upgrades.first().unwrap().property_upgrades.first().unwrap();
    assert_eq!(first.name, "AbilityCooldown");
    assert_eq!(first.bonus, Some(StatValue::Number(-12.0)));
}

#[test]
fn weapons_decode_curated_stats() {
    let items: Vec<Item> = decode(HERO_KIT);

    let weapon = items.iter().find(|item| item.id == INFERNUS_WEAPON).unwrap();
    assert_eq!(weapon.weapon_info.clip_size, Some(27.0));
    assert_eq!(weapon.weapon_info.bullet_damage, Some(5.5));
}

#[test]
fn stat_values_parse_numbers_from_text() {
    assert_eq!(StatValue::Text("20m".to_owned()).as_f64(), Some(20.0));
    assert_eq!(StatValue::Text("0.1".to_owned()).as_f64(), Some(0.1));
    assert_eq!(StatValue::Text("12%".to_owned()).as_f64(), Some(12.0));
    assert_eq!(StatValue::Number(3.0).as_f64(), Some(3.0));
    assert_eq!(StatValue::Text("abc".to_owned()).as_f64(), None);
    assert_eq!(StatValue::Number(25.0).to_string(), "25");
}

#[test]
fn items_decode_with_their_kinds() {
    let items: Vec<Item> = decode(ITEMS);

    let urn = items.iter().find(|item| item.id == SOUL_URN).unwrap();
    assert_eq!(urn.kind, ItemKind::Ability);

    let magazine = items.iter().find(|item| item.id == 1_548_066_885).unwrap();
    assert_eq!(magazine.kind, ItemKind::Upgrade);
    assert_eq!(magazine.cost, Some(800));
    assert_eq!(magazine.item_tier, Some(1));
    assert!(magazine.shopable);
}

#[test]
fn ranks_decode_with_their_images() {
    let ranks: Vec<Rank> = decode(RANKS);

    assert!(ranks.iter().all(|rank| rank.images.large.is_some()));
}

#[test]
fn patch_notes_decode() {
    let patches: Vec<PatchNote> = decode(PATCHES);

    let first = patches.first().unwrap();
    assert_eq!(first.source, "forum");
    assert_eq!(first.guid.text, "urn:xenforo:thread:166726");
}

#[test]
fn item_flow_decodes_nodes_and_summary() {
    let flow: ItemFlowStats = decode(ITEM_FLOW);

    let node = flow.nodes.first().unwrap();
    assert_eq!(node.item_id, 968_099_481);
    assert_eq!(node.matches, 18_360);
    assert!((node.win_rate() - 8_657.0 / 18_360.0).abs() < 1e-9);
    assert!(flow.summary.matches > 0);
}

#[test]
fn ability_orders_decode() {
    let orders: Vec<AbilityOrderStats> = decode(ABILITY_ORDER);

    let first = orders.first().unwrap();
    assert_eq!(first.abilities.len(), 15);
    assert_eq!(first.avg_upgrade_times_s.len(), 15);
    assert_eq!(first.matches, 2_145);
}

#[test]
fn hero_counters_decode() {
    let counters: Vec<HeroCounterStats> = decode(HERO_COUNTER);

    let first = counters.first().unwrap();
    assert_eq!((first.hero_id, first.enemy_hero_id), (1, 66));
    assert_eq!(first.matches_played, 3_252);
    assert!(first.win_rate() > 0.0 && first.win_rate() < 1.0);
}

#[test]
fn lane_matchups_decode() {
    let matchups: Vec<LaneMatchupStats> = decode(LANE_MATCHUP);

    let first = matchups.first().unwrap();
    assert_eq!(first.hero_ids, [1, 31]);
    assert_eq!(first.enemy_hero_ids, [2, 15]);
    assert_eq!(first.sample_time_s, 900);
}

#[test]
fn lane_soul_curves_decode() {
    let curves: Vec<LaneSoulCurve> = decode(LANE_SOUL_CURVE);

    let first = curves.first().unwrap();
    assert_eq!(first.sample_times_s.len(), first.sample_matches.len());
    assert_eq!(first.sample_times_s.len(), first.net_worth_diff.len());
}

#[test]
fn builds_decode_with_ability_order() {
    let builds: Vec<Build> = decode(BUILDS);

    let first = builds.first().unwrap();
    assert_eq!(first.hero_build.hero_id, 1);
    assert_eq!(first.num_favorites, Some(77_623));
    assert!(!first.hero_build.details.mod_categories.is_empty());
    let order = first.hero_build.details.ability_order.as_ref().unwrap();
    assert!(!order.currency_changes.is_empty());
}

#[test]
fn account_ranks_decode_with_and_without_a_last_match() {
    let ranks: Vec<AccountRank> = decode(RANK_BATCH);

    assert_eq!(ranks.len(), 2);
    assert_eq!(
        ranks.first().unwrap().last_match.as_ref().unwrap().match_id,
        28_588_084
    );
    assert!(ranks.get(1).unwrap().last_match.is_none());
    assert!(!ranks.first().unwrap().badge().is_ranked());
}

#[test]
fn match_metadata_decodes_the_text_start_time() {
    let matches: Vec<MatchMetadata> = decode(METADATA_BULK);

    let first = matches.first().unwrap();
    assert_eq!(first.match_id, 117_518_704);
    assert_eq!(first.start_time, "2026-10-15T18:59:34Z".parse().unwrap());
    assert_eq!(first.winning_team, "Team0");

    let player =
        first.players.iter().find(|p| p.account_id == KNOWN_PLAYER).unwrap();
    assert!(!player.items.is_empty());
    assert!(!player.stats.is_empty());
    assert!(!player.won(&first.winning_team));
}

#[test]
fn an_integer_start_time_is_accepted() {
    let json = r#"{
        "match_id": 1,
        "start_time": 1760554774,
        "winning_team": "Team1",
        "duration_s": 600
    }"#;

    let metadata: MatchMetadata = decode(json);

    assert_eq!(metadata.start_time.as_second(), 1_760_554_774);
    assert!(metadata.players.is_empty());
}

#[test]
fn cached_models_round_trip_through_json() {
    assert_round_trips::<Vec<Hero>>(HEROES);
    assert_round_trips::<Vec<Item>>(ITEMS);
    assert_round_trips::<Vec<Item>>(HERO_KIT);
    assert_round_trips::<ItemFlowStats>(ITEM_FLOW);
    assert_round_trips::<Vec<Build>>(BUILDS);
}

#[test]
fn match_metadata_round_trips_with_an_rfc_3339_start_time() {
    let original: Vec<MatchMetadata> = decode(METADATA_BULK);

    let encoded = serde_json::to_string(&original).unwrap();
    let restored: Vec<MatchMetadata> = decode(&encoded);

    let (before, after) = (original.first().unwrap(), restored.first().unwrap());
    assert_eq!(before.match_id, after.match_id);
    assert_eq!(before.start_time, after.start_time);
    assert_eq!(before.players.len(), after.players.len());
}
