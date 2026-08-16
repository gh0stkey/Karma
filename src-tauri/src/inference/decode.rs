use std::collections::HashMap;

use super::config::ViterbiBiases;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tag {
    O,
    B(usize),
    I(usize),
    E(usize),
    S(usize),
}

pub struct LabelSpace {
    pub tags: Vec<Tag>,
    pub entity_names: Vec<String>,
}

impl LabelSpace {
    pub fn from_id2label(id2label: &HashMap<String, String>) -> anyhow::Result<Self> {
        let mut entries: Vec<(usize, String)> = id2label
            .iter()
            .filter_map(|(id, name)| id.parse::<usize>().ok().map(|i| (i, name.clone())))
            .collect();
        entries.sort_by_key(|(i, _)| *i);

        let mut entity_ids: HashMap<String, usize> = HashMap::new();
        let mut entity_names: Vec<String> = Vec::new();
        let mut tags = Vec::with_capacity(entries.len());

        for (_, name) in &entries {
            let (prefix, entity) = name
                .split_once('-')
                .map(|(p, e)| (p, Some(e)))
                .unwrap_or((name.as_str(), None));
            let entity_idx = match entity {
                Some(e) => {
                    let next = entity_names.len();
                    *entity_ids.entry(e.to_string()).or_insert_with(|| {
                        entity_names.push(e.to_string());
                        next
                    })
                }
                None => usize::MAX,
            };
            let tag = match prefix {
                "O" => Tag::O,
                "B" => Tag::B(entity_idx),
                "I" => Tag::I(entity_idx),
                "E" => Tag::E(entity_idx),
                "S" => Tag::S(entity_idx),
                other => anyhow::bail!("unknown label prefix: {other}"),
            };
            tags.push(tag);
        }

        Ok(Self { tags, entity_names })
    }

    fn allowed(prev: Tag, next: Tag) -> bool {
        match next {
            Tag::O | Tag::B(_) | Tag::S(_) => matches!(prev, Tag::O | Tag::E(_) | Tag::S(_)),
            Tag::I(e) | Tag::E(e) => matches!(prev, Tag::B(p) | Tag::I(p) if p == e),
        }
    }

    fn start_allowed(tag: Tag) -> bool {
        matches!(tag, Tag::O | Tag::B(_) | Tag::S(_))
    }

    fn end_allowed(tag: Tag) -> bool {
        matches!(tag, Tag::O | Tag::E(_) | Tag::S(_))
    }

    fn transition_bias(&self, prev: Tag, next: Tag, biases: &ViterbiBiases) -> f32 {
        match (prev, next) {
            (Tag::O, Tag::O) => biases.background_stay,
            (Tag::O, Tag::B(_) | Tag::S(_)) => biases.background_to_start,
            (Tag::E(_) | Tag::S(_), Tag::O) => biases.end_to_background,
            (Tag::E(_) | Tag::S(_), Tag::B(_) | Tag::S(_)) => biases.end_to_start,
            (Tag::B(p) | Tag::I(p), Tag::I(e)) if p == e => biases.inside_to_continue,
            (Tag::B(p) | Tag::I(p), Tag::E(e)) if p == e => biases.inside_to_end,
            _ => 0.0,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct RawSpan {
    pub entity: usize,
    pub start_token: usize,
    pub end_token: usize,
}

pub fn viterbi_decode(
    logits: &[Vec<f32>],
    space: &LabelSpace,
    biases: &ViterbiBiases,
) -> Vec<usize> {
    let n_labels = space.tags.len();
    let n_tokens = logits.len();
    if n_tokens == 0 {
        return Vec::new();
    }

    let neg_inf = f32::NEG_INFINITY;
    let mut dp = vec![vec![neg_inf; n_labels]; n_tokens];
    let mut back = vec![vec![0usize; n_labels]; n_tokens];

    for (l, tag) in space.tags.iter().enumerate() {
        if LabelSpace::start_allowed(*tag) {
            dp[0][l] = logits[0][l];
        }
    }

    for t in 1..n_tokens {
        for (cur, &cur_tag) in space.tags.iter().enumerate() {
            let mut best = neg_inf;
            let mut best_prev = 0usize;
            for (prev, &prev_tag) in space.tags.iter().enumerate() {
                if dp[t - 1][prev] == neg_inf || !LabelSpace::allowed(prev_tag, cur_tag) {
                    continue;
                }
                let score = dp[t - 1][prev] + space.transition_bias(prev_tag, cur_tag, biases);
                if score > best {
                    best = score;
                    best_prev = prev;
                }
            }
            if best > neg_inf {
                dp[t][cur] = best + logits[t][cur];
                back[t][cur] = best_prev;
            }
        }
    }

    let mut best_end = 0usize;
    let mut best_score = neg_inf;
    for (l, tag) in space.tags.iter().enumerate() {
        if LabelSpace::end_allowed(*tag) && dp[n_tokens - 1][l] > best_score {
            best_score = dp[n_tokens - 1][l];
            best_end = l;
        }
    }

    let mut path = vec![0usize; n_tokens];
    path[n_tokens - 1] = best_end;
    for t in (1..n_tokens).rev() {
        path[t - 1] = back[t][path[t]];
    }
    path
}

pub fn extract_spans(path: &[usize], space: &LabelSpace) -> Vec<RawSpan> {
    let mut spans = Vec::new();
    let mut t = 0;
    while t < path.len() {
        match space.tags[path[t]] {
            Tag::S(e) => {
                spans.push(RawSpan {
                    entity: e,
                    start_token: t,
                    end_token: t,
                });
                t += 1;
            }
            Tag::B(e) => {
                let mut end = t;
                let mut u = t + 1;
                while u < path.len() {
                    match space.tags[path[u]] {
                        Tag::I(x) if x == e => {
                            end = u;
                            u += 1;
                        }
                        Tag::E(x) if x == e => {
                            end = u;
                            break;
                        }
                        _ => break,
                    }
                }
                spans.push(RawSpan {
                    entity: e,
                    start_token: t,
                    end_token: end,
                });
                t = end + 1;
            }
            _ => {
                t += 1;
            }
        }
    }
    spans
}

#[cfg(test)]
mod tests {
    use super::*;

    fn space() -> LabelSpace {
        let id2label: HashMap<String, String> = [
            ("0", "O"),
            ("1", "B-private_url"),
            ("2", "I-private_url"),
            ("3", "E-private_url"),
            ("4", "S-private_url"),
            ("5", "B-private_email"),
            ("6", "I-private_email"),
            ("7", "E-private_email"),
            ("8", "S-private_email"),
        ]
        .into_iter()
        .map(|(k, v)| (k.to_string(), v.to_string()))
        .collect();
        LabelSpace::from_id2label(&id2label).unwrap()
    }

    fn logits_from_tags(tags: &[&str], space: &LabelSpace) -> Vec<Vec<f32>> {
        tags.iter()
            .map(|name| {
                space
                    .tags
                    .iter()
                    .enumerate()
                    .map(|(i, _)| {
                        let matches = match *name {
                            "O" => space.tags[i] == Tag::O,
                            "B-0" => space.tags[i] == Tag::B(0),
                            "I-0" => space.tags[i] == Tag::I(0),
                            "E-0" => space.tags[i] == Tag::E(0),
                            "S-0" => space.tags[i] == Tag::S(0),
                            "B-1" => space.tags[i] == Tag::B(1),
                            "I-1" => space.tags[i] == Tag::I(1),
                            "E-1" => space.tags[i] == Tag::E(1),
                            "S-1" => space.tags[i] == Tag::S(1),
                            _ => false,
                        };
                        if matches { 10.0 } else { 0.0 }
                    })
                    .collect()
            })
            .collect()
    }

    #[test]
    fn viterbi_recovers_full_url_span() {
        let s = space();
        let logits = logits_from_tags(&["O", "B-0", "I-0", "I-0", "I-0", "E-0", "O", "O"], &s);
        let path = viterbi_decode(&logits, &s, &ViterbiBiases::default());
        let spans = extract_spans(&path, &s);
        assert_eq!(
            spans,
            vec![RawSpan {
                entity: 0,
                start_token: 1,
                end_token: 5
            }]
        );
    }

    #[test]
    fn viterbi_rejects_inside_without_begin() {
        let s = space();
        let logits = logits_from_tags(&["O", "I-0", "I-0", "O"], &s);
        let path = viterbi_decode(&logits, &s, &ViterbiBiases::default());
        let spans = extract_spans(&path, &s);
        assert!(spans.is_empty());
    }

    #[test]
    fn viterbi_forces_span_closure_at_end() {
        let s = space();
        let logits = logits_from_tags(&["O", "B-0", "I-0", "I-0"], &s);
        let path = viterbi_decode(&logits, &s, &ViterbiBiases::default());
        let spans = extract_spans(&path, &s);
        assert_eq!(
            spans,
            vec![RawSpan {
                entity: 0,
                start_token: 1,
                end_token: 3
            }]
        );
    }

    #[test]
    fn viterbi_handles_adjacent_entities() {
        let s = space();
        let logits = logits_from_tags(&["O", "B-0", "E-0", "B-1", "I-1", "E-1", "O"], &s);
        let path = viterbi_decode(&logits, &s, &ViterbiBiases::default());
        let spans = extract_spans(&path, &s);
        assert_eq!(
            spans,
            vec![
                RawSpan {
                    entity: 0,
                    start_token: 1,
                    end_token: 2
                },
                RawSpan {
                    entity: 1,
                    start_token: 3,
                    end_token: 5
                }
            ]
        );
    }
}
