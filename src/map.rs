use regex::Regex;
use std::sync::LazyLock;

static MAP_REGEX: LazyLock<Regex> = LazyLock::new(|| {
    Regex::new(
        r"^https://osu.ppy.sh/beatmapsets/(?<set_id>\d+)(#(?<game_mode>\w+)/(?<map_id>\d+))?",
    )
    .unwrap()
});

#[cfg(test)]
mod test;

pub struct OsuMap {
    pub game_mode: String,
    pub id: u64,
}

pub struct OsuMapSet {
    pub set_id: u64,
    pub map: Option<OsuMap>,
}

impl OsuMapSet {
    pub fn format_to_link(&self) -> String {
        if let Some(map) = self.map.as_ref() {
            return format!(
                "https://osu.ppy.sh/beatmapsets/{}#{}/{}",
                self.set_id, map.game_mode, map.id
            );
        }

        format!("https://osu.ppy.sh/beatmapsets/{}", self.set_id)
    }

    pub fn try_parse(content: &str) -> Option<OsuMapSet> {
        let Some(caps) = MAP_REGEX.captures(content) else {
            return None;
        };

        let set_id: u64 = caps["set_id"].parse().unwrap();

        let map = match caps.name("map_id") {
            Some(m) => {
                let id = m.as_str().parse().unwrap();
                let game_mode: String = caps.name("game_mode").unwrap().as_str().to_owned();

                Some(OsuMap { id, game_mode })
            }
            _ => None,
        };

        Some(OsuMapSet { set_id, map })
    }
}