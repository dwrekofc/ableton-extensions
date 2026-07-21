use nucleo_matcher::{
    Config, Matcher, Utf32Str,
    pattern::{Atom, AtomKind, CaseMatching, Normalization},
};
use palette_protocol::CatalogItem;
use std::cmp::Reverse;

pub fn rank_items(query: &str, mut items: Vec<CatalogItem>, limit: usize) -> Vec<CatalogItem> {
    let normalized_query = query.trim().to_lowercase();
    if normalized_query.is_empty() {
        items.sort_by_key(|item| {
            Reverse((item.pinned as u8, item.favorite as u8, item.usage_count))
        });
        items.truncate(limit);
        return items;
    }

    let pattern = Atom::new(
        &normalized_query,
        CaseMatching::Ignore,
        Normalization::Smart,
        AtomKind::Fuzzy,
        false,
    );
    let mut matcher = Matcher::new(Config::DEFAULT);
    let mut buffer = Vec::new();
    let mut scored = Vec::new();

    for item in items {
        let name_lower = item.name.to_lowercase();
        let mut candidates = vec![name_lower.clone()];
        candidates.extend(item.aliases.iter().map(|value| value.to_lowercase()));
        candidates.extend(item.categories.iter().map(|value| value.to_lowercase()));
        candidates.extend(item.tags.iter().map(|value| value.to_lowercase()));

        let fuzzy = candidates
            .iter()
            .filter_map(|candidate| {
                pattern.score(Utf32Str::new(candidate, &mut buffer), &mut matcher)
            })
            .max();
        let Some(fuzzy) = fuzzy else { continue };

        let exact_alias = item
            .aliases
            .iter()
            .any(|alias| alias.eq_ignore_ascii_case(&normalized_query));
        let score = fuzzy as u64
            + u64::from(name_lower == normalized_query) * 5_000
            + u64::from(name_lower.starts_with(&normalized_query)) * 1_500
            + u64::from(exact_alias) * 3_000
            + u64::from(item.pinned) * 1_000
            + u64::from(item.favorite) * 500
            + item.usage_count.min(250) * 4;
        scored.push((score, item));
    }

    scored.sort_by_key(|(score, item)| (Reverse(*score), item.name.to_lowercase()));
    scored
        .into_iter()
        .take(limit)
        .map(|(_, item)| item)
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use palette_protocol::{ItemKind, ItemSource};

    fn item(id: &str, name: &str) -> CatalogItem {
        CatalogItem {
            id: id.into(),
            kind: ItemKind::NativeDevice,
            source: ItemSource::LiveBrowser,
            name: name.into(),
            aliases: vec![],
            categories: vec![],
            tags: vec![],
            browser_path: vec![],
            compatible_tracks: vec![],
            favorite: false,
            pinned: false,
            usage_count: 0,
            last_used_at: None,
            metadata: serde_json::Value::Null,
        }
    }

    #[test]
    fn aliases_and_preferences_influence_results() {
        let reverb = item("reverb", "Reverb");
        let mut filter = item("filter", "Auto Filter");
        filter.aliases.push("verb".into());
        filter.favorite = true;
        assert_eq!(rank_items("verb", vec![reverb, filter], 1)[0].id, "filter");
    }

    #[test]
    fn empty_query_prioritizes_pins() {
        let first = item("first", "Alpha");
        let mut second = item("second", "Zulu");
        second.pinned = true;
        assert_eq!(rank_items("", vec![first, second], 10)[0].id, "second");
    }
}
