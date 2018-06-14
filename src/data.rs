use failure;
use std;
use std::collections::{hash_map::Entry,
                       {HashMap, HashSet}};
use std::fs;
use std::path::{Path, PathBuf};
use markdown;

type Result<T> = std::result::Result<T, failure::Error>;
type PageId = u32;

pub struct Page {
    pub rendered: String,
    path: PathBuf,
}

#[derive(Default)]
pub struct State {
    pub pages_by_id: HashMap<PageId, Page>,
    tag_sets: HashMap<String, HashSet<PageId>>,
    next_page_id: PageId,
    all_pages: HashSet<PageId>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Match {
    pub type_: MatchType,
    used_tags: Vec<String>,
    skipped_tags: Vec<String>,
}

impl Match {
    fn is_none(&self) -> bool {
        self.type_ == MatchType::None
    }

    fn is_one(&self) -> bool {
        if let MatchType::One(_) = self.type_ {
            true
        } else {
            false
        }
    }
    fn is_many(&self) -> bool {
        if let MatchType::Many(_) = self.type_ {
            true
        } else {
            false
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum MatchType {
    None,
    One(PageId),
    Many(Vec<PageId>),
}

impl State {
    pub fn insert_from_dir(dir_path: &Path) -> Result<State> {
        let mut state: State = Default::default();
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                state.insert_from_file(&path)?;
            }
        }

        Ok(state)
    }

    pub fn insert_from_file(&mut self, md_path: &Path) -> Result<()> {
        let md = fs::read_to_string(md_path)?;
        let (tags, rendered) = markdown::parse_markdown(&md);

        let page_id = self.next_page_id;

        let page = Page {
            path: md_path.into(),
            rendered: rendered,
        };

        self.insert(page, tags);
        Ok(())
    }

    fn insert(&mut self, page: Page, tags: Vec<String>) -> PageId {
        let page_id = self.next_page_id;
        self.next_page_id += 1;
        self.all_pages.insert(page_id);
        for tag in tags.into_iter() {
            self.tag_sets
                .entry(tag)
                .or_insert(Default::default())
                .insert(page_id);
        }
        self.pages_by_id.insert(page_id, page);
        page_id
    }

    pub fn find_best_match(&self, mut tags: Vec<String>) -> Match {
        let mut matches: Option<HashSet<PageId>> = None;
        let mut used_tags = vec![];
        let mut skipped_tags = vec![];

        for (i, tag) in tags.into_iter().enumerate() {
            if let Some(set) = self.tag_sets.get(&tag) {
                let new_matches: HashSet<PageId> = matches
                    .as_ref()
                    .unwrap_or(&self.all_pages)
                    .intersection(set)
                    .into_iter()
                    .cloned()
                    .collect();

                match new_matches.len() {
                    0 => {
                        skipped_tags.push(tag);
                    }
                    1 => {
                        used_tags.push(tag);
                        matches = Some(new_matches);
                    }
                    _ => {
                        used_tags.push(tag);
                        matches = Some(new_matches);
                    }
                }
            } else {
                skipped_tags.push(tag);
            }
        }

        let matches: Vec<PageId> = matches
            .as_ref()
            .unwrap_or(&self.all_pages)
            .iter()
            .take(10)
            .cloned()
            .collect();

        Match {
            skipped_tags: skipped_tags,
            used_tags: used_tags,
            type_: match matches.len() {
                0 => MatchType::None,
                1 => {
                    let page_id = matches.into_iter().next().unwrap();
                    MatchType::One(page_id)
                }
                _ => MatchType::Many(matches),
            },
        }
    }
}

#[test]
fn simple() {
    let mut state: State = Default::default();

    assert!(state.find_best_match(vec![]).is_none());

    let p1 = state.insert(
        Page {
            rendered: "".into(),
            path: "".into(),
        },
        vec!["a".into(), "b".into()],
    );

    let p2 = state.insert(
        Page {
            rendered: "".into(),
            path: "".into(),
        },
        vec!["a".into(), "c".into()],
    );

    let empty: Vec<String> = vec![];
    let m = state.find_best_match(empty.clone());
    assert!(m.is_many());
    assert_eq!(m.used_tags, empty);
    assert_eq!(m.skipped_tags, empty);

    let tags = vec!["x".into()];
    let m = state.find_best_match(tags.clone());
    assert!(m.is_many());
    assert_eq!(m.used_tags, empty);
    assert_eq!(m.skipped_tags, tags);

    let tags = vec!["a".into()];
    let m = state.find_best_match(tags.clone());
    assert!(m.is_many());
    assert_eq!(m.used_tags, tags);
    assert_eq!(m.skipped_tags, empty);

    let tags = vec!["a".into(), "x".into()];
    let m = state.find_best_match(tags.clone());
    assert!(m.is_many());
    assert_eq!(m.used_tags, vec!["a".to_string()]);
    assert_eq!(m.skipped_tags, vec!["x".to_string()]);

    let tags = vec!["a".into(), "b".into()];
    let m = state.find_best_match(tags.clone());
    assert!(m.is_one());
    assert_eq!(m.used_tags, vec!["a".to_string(), "b".into()]);
    assert_eq!(m.skipped_tags, empty);

    let tags = vec!["a".into(), "b".into()];
    let m = state.find_best_match(tags.clone());
    assert!(m.is_one());
    assert_eq!(m.used_tags, vec!["a".to_string(), "b".into()]);
    assert_eq!(m.skipped_tags, empty);

    let tags = vec!["a".to_string(), "x".into(), "b".into()];
    let m = state.find_best_match(tags.clone());
    assert!(m.is_one());
    assert_eq!(m.used_tags, vec!["a".to_string(), "b".into()]);
    assert_eq!(m.skipped_tags, vec!["x".to_string()]);
}
