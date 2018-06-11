use failure;
use std;
use std::collections::{hash_map::Entry,
                       {HashMap, HashSet}};
use std::fs;
use std::path::{Path, PathBuf};

type Result<T> = std::result::Result<T, failure::Error>;
type PageId = u32;

struct Page {
    rendered: String,
    path: PathBuf,
}

#[derive(Default)]
pub struct State {
    pages_by_id: HashMap<PageId, Page>,
    tag_sets: HashMap<String, HashSet<PageId>>,
    next_page_id: PageId,
    all_pages: HashSet<PageId>,
}

impl State {
    fn insert_from_dir(dir_path: &Path) -> Result<State> {
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

    fn insert_from_file(&mut self, md_path: &Path) -> Result<()> {
        let md = fs::read_to_string(md_path)?;
        let (tags, rendered) = ::markdown::parse_markdown(&md);

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

    fn find_best_match(&self, mut tags: Vec<String>) -> BestMatch {
        let mut matches: Option<HashSet<PageId>> = None;
        let mut used_tags = vec![];

        for (i, tag) in tags.iter().cloned().enumerate() {
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
                        break;
                    }
                    1 => {
                        used_tags.push(tag);
                        matches = Some(new_matches);
                        break;
                    }
                    _ => {
                        used_tags.push(tag);
                        matches = Some(new_matches);
                    }
                }
            }
        }

        let matches: Vec<PageId> = matches
            .as_ref()
            .unwrap_or(&self.all_pages)
            .iter()
            .take(10)
            .cloned()
            .collect();

        match matches.len() {
            0 => BestMatch::NotFound,
            1 => {
                let page_id = matches.into_iter().next().unwrap();
                BestMatch::Found {
                    page_id: page_id,
                    skipped_tags: tags[used_tags.len()..].to_vec(),
                    used_tags: used_tags,
                }
            }
            _ => BestMatch::Ambigous {
                page_ids: matches,
                skipped_tags: tags[used_tags.len()..].to_vec(),
                used_tags: used_tags,
            },
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
enum BestMatch {
    NotFound,
    Found {
        page_id: PageId,
        used_tags: Vec<String>,
        skipped_tags: Vec<String>,
    },
    Ambigous {
        page_ids: Vec<PageId>,
        used_tags: Vec<String>,
        skipped_tags: Vec<String>,
    },
}

#[test]
fn simple() {
    let mut state: State = Default::default();

    assert_eq!(state.find_best_match(vec![]), BestMatch::NotFound,);

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

    let empty : Vec<String> = vec![];
    let m = state.find_best_match(empty.clone());
    if let BestMatch::Ambigous {
        page_ids,
        used_tags,
        skipped_tags
    } = m {
        assert_eq!(page_ids.len(), 2);
        assert_eq!(used_tags, empty);
        assert_eq!(skipped_tags, empty);
    } else {
        panic!(m);
    }


    let tags = vec!["x".into()];
    let m = state.find_best_match(tags.clone());
    if let BestMatch::Ambigous {
        page_ids,
        used_tags,
        skipped_tags
    } = m {
        assert_eq!(page_ids.len(), 2);
        assert_eq!(used_tags, empty);
        assert_eq!(skipped_tags, tags);
    } else {
        panic!(m);
    }


    let tags = vec!["a".into()];
    let m = state.find_best_match(tags.clone());
    if let BestMatch::Ambigous {
        page_ids,
        used_tags,
        skipped_tags
    } = m {
        assert_eq!(page_ids.len(), 2);
        assert_eq!(used_tags, tags);
        assert_eq!(skipped_tags, empty);
    } else {
        panic!(m);
    }


    let tags = vec!["a".into(), "x".into()];
    let m = state.find_best_match(tags.clone());
    if let BestMatch::Ambigous {
        page_ids,
        used_tags,
        skipped_tags
    } = m {
        assert_eq!(page_ids.len(), 2);
        assert_eq!(used_tags, vec!["a".to_string()]);
        assert_eq!(skipped_tags, vec!["x".to_string()]);
    } else {
        panic!(m);
    }


    let tags = vec!["a".into(), "b".into()];
    let m = state.find_best_match(tags.clone());
    if let BestMatch::Found {
        page_id,
        used_tags,
        skipped_tags
    } = m {
        assert_eq!(used_tags, vec!["a".to_string(), "b".into()]);
        assert_eq!(skipped_tags, empty);
    } else {
        panic!(m);
    }


    let tags = vec!["a".into(), "b".into()];
    let m = state.find_best_match(tags.clone());
    if let BestMatch::Found {
        page_id,
        used_tags,
        skipped_tags
    } = m {
        assert_eq!(used_tags, vec!["a".to_string(), "b".into()]);
        assert_eq!(skipped_tags, empty);
    } else {
        panic!(m);
    }


    let tags = vec!["a".to_string(), "x".into(), "b".into()];
    let m = state.find_best_match(tags.clone());
    if let BestMatch::Found {
        page_id,
        used_tags,
        skipped_tags
    } = m {
        assert_eq!(used_tags, vec!["a".to_string(), "b".into()]);
        assert_eq!(skipped_tags, vec!["x".to_string]);
    } else {
        panic!(m);
    }
}
