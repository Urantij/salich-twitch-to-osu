use super::*;

#[test]
fn parse_set_full() {
    let test = "https://osu.ppy.sh/beatmapsets/2233561#osu/4743148";

    let set = OsuMapSet::try_parse(test);

    assert!(set.is_some());

    let set = set.unwrap();

    let map = set.map.unwrap();

    assert_eq!(set.set_id, 2233561);
    assert_eq!(map.game_mode, "osu");
    assert_eq!(map.id, 4743148u64);
}

#[test]
fn parse_set_min() {
    let test = "https://osu.ppy.sh/beatmapsets/2233561";

    let set = OsuMapSet::try_parse(test);

    assert!(set.is_some());

    let set = set.unwrap();

    assert_eq!(set.set_id, 2233561);
    assert!(set.map.is_none());
}

#[test]
fn parse_map() {
    let test = "https://osu.ppy.sh/beatmaps/492285";

    let map = OsuMap::try_parse(test);

    assert!(map.is_some());

    let map = map.unwrap();

    assert_eq!(map.id, 492285);
}
