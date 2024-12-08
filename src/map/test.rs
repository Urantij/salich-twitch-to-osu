use super::*;

#[test]
fn parse_full() {
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
fn parse_min() {
    let test = "https://osu.ppy.sh/beatmapsets/2233561";

    let set = OsuMapSet::try_parse(test);

    assert!(set.is_some());

    let set = set.unwrap();

    assert_eq!(set.set_id, 2233561);
    assert!(set.map.is_none());
}
