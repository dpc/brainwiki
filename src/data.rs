use page::Page;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::{Path, PathBuf};
use std::{sync, thread};

use notify::{DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher};
use std::time::Duration;

use Result;

pub type PageId = u32;
pub type NarrowingTagsSet = HashMap<String, usize>;

#[derive(Default)]
pub struct State {
    pub pages_by_id: HashMap<PageId, Page>,
    pub pages_by_path: HashMap<PathBuf, PageId>,
    tag_sets: HashMap<String, HashSet<PageId>>,
    next_page_id: PageId,
    all_pages: HashSet<PageId>,
}

#[derive(Debug, PartialEq, Eq)]
pub struct Match {
    pub type_: MatchType,
    pub matching_tags: Vec<String>,
    pub unmatched_tags: Vec<String>,
    pub narrowing_tags: NarrowingTagsSet,
}

impl Match {
    fn is_none(&self) -> bool {
        self.type_ == MatchType::None
    }

    pub fn is_one(&self) -> bool {
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

    pub fn has_unmatched_tags(&self) -> bool {
        !self.unmatched_tags.is_empty()
    }

    pub fn to_precise_url(&self, prefer_exact: bool) -> String {
        let mut location = String::from("/") + self.matching_tags.join("/").as_str();
        if !prefer_exact {
            location += "/";
        }

        location
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum MatchType {
    None,
    One(PageId),
    Many(Vec<PageId>),
}

impl State {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn insert_from_dir(&mut self, dir_path: &Path) -> Result<()> {
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file() {
                self.insert_from_file(&path)?;
            }
        }
        Ok(())
    }

    pub fn insert_from_file(&mut self, md_path: &Path) -> ::Result<()> {
        let page = Page::read_from_file(md_path)?;

        self.insert(page);
        Ok(())
    }

    fn insert(&mut self, page: Page) -> PageId {
        debug_assert_eq!(page.fs_path, page.fs_path.canonicalize().unwrap());
        let page_id = self.next_page_id;
        self.next_page_id += 1;
        self.all_pages.insert(page_id);

        for tag in page.tags.iter() {
            self.tag_sets
                .entry(tag.clone())
                .or_insert(Default::default())
                .insert(page_id);
        }
        self.pages_by_path.insert(page.fs_path.clone(), page_id);
        self.pages_by_id.insert(page_id, page);
        page_id
    }

    fn remove(&mut self, page_id: PageId) {
        let page = self.pages_by_id.remove(&page_id).unwrap();
        for tag in page.tags.iter() {
            self.tag_sets
                .get_mut(&tag.clone())
                .unwrap()
                .remove(&page_id);
        }
        self.all_pages.remove(&page_id);
        self.pages_by_path.remove(&page.fs_path).unwrap();
    }

    pub fn lookup(&self, tags: Vec<String>) -> Result<PageId> {
        match self.find_best_match(tags, true).type_ {
            MatchType::Many(_) => {
                bail!("Multiple pages matching");
            }
            MatchType::One(id) => Ok(id),
            MatchType::None => {
                bail!("Not found");
            }
        }
    }

    pub fn find_best_match(&self, tags: Vec<String>, prefer_exact: bool) -> Match {
        let mut matches: Option<HashSet<PageId>> = None;
        let mut matching_tags = vec![];
        let mut unmatched_tags = vec![];

        for tag in tags.iter().cloned() {
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
                        unmatched_tags.push(tag);
                    }
                    1 => {
                        matching_tags.push(tag);
                        matches = Some(new_matches);
                    }
                    _ => {
                        matching_tags.push(tag);
                        matches = Some(new_matches);
                    }
                }
            } else {
                unmatched_tags.push(tag);
            }
        }

        let matches: Vec<PageId> = matches
            .as_ref()
            .unwrap_or(&self.all_pages)
            .iter()
            .take(1000)
            .cloned()
            .collect();

        let mut narrowing_tags = HashMap::new();

        for page_id in &matches {
            for tag in &self.pages_by_id.get(&page_id).unwrap().tags {
                if !matching_tags.contains(&tag) {
                    *narrowing_tags.entry(tag.clone()).or_insert(0) += 1;
                }
            }
        }

        Match {
            unmatched_tags: unmatched_tags,
            matching_tags: matching_tags,
            narrowing_tags,
            type_: match matches.len() {
                0 => MatchType::None,
                1 => {
                    let page_id = matches.into_iter().next().unwrap();
                    MatchType::One(page_id)
                }
                _ => {
                    if prefer_exact {
                        let id = matches
                            .iter()
                            .find(|id| {
                                self.pages_by_id
                                    .get(&id)
                                    .unwrap()
                                    .tags
                                    .iter()
                                    .all(|tag| tags.contains(&tag))
                            })
                            .cloned();
                        if let Some(id) = id {
                            MatchType::One(id)
                        } else {
                            MatchType::Many(matches)
                        }
                    } else {
                        MatchType::Many(matches)
                    }
                }
            },
        }
    }
}

#[derive(Clone)]
pub struct SyncState {
    inner: sync::Arc<sync::RwLock<State>>,
}

impl SyncState {
    pub fn new() -> Self {
        SyncState {
            inner: sync::Arc::new(sync::RwLock::new(State::new())),
        }
    }
    pub fn write<'a>(&'a self) -> sync::RwLockWriteGuard<'a, State> {
        self.inner.write().unwrap()
    }
    pub fn read<'a>(&'a self) -> sync::RwLockReadGuard<'a, State> {
        self.inner.read().unwrap()
    }

    fn handle_create(&self, path: PathBuf) -> Result<()> {
        let new_page = Page::read_from_file(&*path)?;

        let mut inner = self.inner.write().unwrap();
        if let Some(id) = inner.pages_by_path.get(path.as_path()).cloned() {
            inner.remove(id);
        }
        inner.insert(new_page);

        Ok(())
    }

    fn handle_remove(&self, path: PathBuf) -> Result<()> {
        let mut inner = self.inner.write().unwrap();
        if let Some(id) = inner.pages_by_path.get(path.as_path()).cloned() {
            inner.remove(id);
        }

        Ok(())
    }
    fn handle_rename(&self, src: PathBuf, dst: PathBuf) -> Result<()> {
        let new_page = Page::read_from_file(&*dst)?;

        let mut inner = self.inner.write().unwrap();
        if let Some(id) = inner.pages_by_path.get(dst.as_path()).cloned() {
            inner.remove(id);
        }
        if let Some(id) = inner.pages_by_path.get(src.as_path()).cloned() {
            inner.remove(id);
        }
        inner.insert(new_page);

        Ok(())
    }
}

pub struct FsWatcher {
    join_handle: thread::JoinHandle<Result<()>>,
}

impl FsWatcher {
    pub fn new(dir: PathBuf, state: SyncState) -> ::Result<Self> {
        let (tx, rx) = sync::mpsc::channel();
        let mut watcher: RecommendedWatcher = Watcher::new(tx, Duration::from_secs(1))?;

        watcher.watch(dir, RecursiveMode::Recursive)?;

        let join_handle = thread::spawn(move || {
            let _watcher = watcher;
            loop {
                // Intentionally ignore Fs errors and keep going
                match rx.recv().unwrap() {
                    DebouncedEvent::Create(path) => {
                        let _ = state.handle_create(path)?;
                    }
                    DebouncedEvent::Remove(path) => {
                        let _ = state.handle_create(path)?;
                    }
                    DebouncedEvent::Rename(src, dst) => {
                        state.handle_rename(src, dst)?;
                    }
                    _ => {}
                }
            }
        });

        Ok(FsWatcher { join_handle })
    }
}

#[test]
fn simple() {
    let mut state: State = Default::default();

    assert!(state.find_best_match(vec![], false).is_none());

    let p1 = state.insert(Page {
        html: "".into(),
        fs_path: "".into(),
        tags: vec!["a".into(), "b".into()],
        title: "".into(),
        md: "".into(),
    });

    let p2 = state.insert(Page {
        html: "".into(),
        fs_path: "".into(),
        tags: vec!["a".into(), "c".into()],
        title: "".into(),
        md: "".into(),
    });

    let empty: Vec<String> = vec![];
    let m = state.find_best_match(empty.clone(), false);
    assert!(m.is_many());
    assert_eq!(m.matching_tags, empty);
    assert_eq!(m.unmatched_tags, empty);

    let tags = vec!["x".into()];
    let m = state.find_best_match(tags.clone(), false);
    assert!(m.is_many());
    assert_eq!(m.matching_tags, empty);
    assert_eq!(m.unmatched_tags, tags);

    let tags = vec!["a".into()];
    let m = state.find_best_match(tags.clone(), false);
    assert!(m.is_many());
    assert_eq!(m.matching_tags, tags);
    assert_eq!(m.unmatched_tags, empty);

    let tags = vec!["a".into(), "x".into()];
    let m = state.find_best_match(tags.clone(), false);
    assert!(m.is_many());
    assert_eq!(m.matching_tags, vec!["a".to_string()]);
    assert_eq!(m.unmatched_tags, vec!["x".to_string()]);

    let tags = vec!["a".into(), "b".into()];
    let m = state.find_best_match(tags.clone(), false);
    assert!(m.is_one());
    assert_eq!(m.matching_tags, vec!["a".to_string(), "b".into()]);
    assert_eq!(m.unmatched_tags, empty);

    let tags = vec!["a".into(), "b".into()];
    let m = state.find_best_match(tags.clone(), false);
    assert!(m.is_one());
    assert_eq!(m.matching_tags, vec!["a".to_string(), "b".into()]);
    assert_eq!(m.unmatched_tags, empty);

    let tags = vec!["a".to_string(), "x".into(), "b".into()];
    let m = state.find_best_match(tags.clone(), false);
    assert!(m.is_one());
    assert_eq!(m.matching_tags, vec!["a".to_string(), "b".into()]);
    assert_eq!(m.unmatched_tags, vec!["x".to_string()]);
}
