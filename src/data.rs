use std::{
    collections::{HashMap, HashSet},
    fs,
    path::{Path, PathBuf},
    sync, thread,
};

use notify::{
    DebouncedEvent, RecommendedWatcher, RecursiveMode, Watcher,
};
use std::{fs::File, io::Write, time::Duration};

use crate::{page::Page, Result};

#[derive(
    From,
    Into,
    Eq,
    PartialEq,
    Hash,
    Default,
    Debug,
    Clone,
    Copy,
    AddAssign,
)]
pub struct PageId(u32);
pub type NarrowingTagsSet = HashMap<String, usize>;

#[derive(Default)]
pub struct State {
    pub pages_by_id: HashMap<PageId, Page>,
    pub pages_by_path: HashMap<PathBuf, PageId>,
    pub path_by_id: HashMap<PageId, PathBuf>,
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
    #[allow(unused)]
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

    #[allow(unused)]
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
        let mut location = String::from("/")
            + self.matching_tags.join("/").as_str();
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

#[derive(Debug, PartialEq, Eq, Clone)]
pub enum LookupOutcome {
    None,
    One(PageId),
    Many,
}

impl State {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn insert_from_dir(
        &mut self,
        dir_path: &Path,
    ) -> Result<()> {
        for entry in fs::read_dir(dir_path)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_file()
                || path.extension().and_then(|e| e.to_str())
                    == Some("md")
            {
                self.insert_from_file(&path)?;
            }
        }
        Ok(())
    }

    pub fn insert_from_file(
        &mut self,
        md_path: &Path,
    ) -> Result<()> {
        let page = Page::read_from_file(md_path)?;

        self.insert(page, &md_path.canonicalize()?);
        Ok(())
    }

    fn insert(&mut self, page: Page, path: &Path) -> PageId {
        let page_id = self.next_page_id;
        self.next_page_id += 1.into();
        self.all_pages.insert(page_id);

        for tag in page.tags.iter() {
            self.tag_sets
                .entry(tag.clone())
                .or_insert(Default::default())
                .insert(page_id);
        }
        self.pages_by_path.insert(path.into(), page_id);
        self.path_by_id.insert(page_id, path.into());
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
        let path = self.path_by_id.remove(&page_id).unwrap();
        self.pages_by_path.remove(&path).unwrap();
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

    pub fn lookup_exact(
        &self,
        tags: Vec<String>,
    ) -> LookupOutcome {
        let mut matches: Option<HashSet<PageId>> = None;
        for tag in tags.iter().cloned() {
            if let Some(set) = self.tag_sets.get(&tag) {
                let new_matches: HashSet<PageId> = matches
                    .as_ref()
                    .unwrap_or(&self.all_pages)
                    .intersection(set)
                    .into_iter()
                    .cloned()
                    .collect();

                if new_matches.is_empty() {
                    return LookupOutcome::None;
                }
                matches = Some(new_matches);
            } else {
                return LookupOutcome::None;
            }
        }
        let matches =
            matches.as_ref().unwrap_or(&self.all_pages);
        match matches.len() {
            0 => LookupOutcome::None,
            1 => LookupOutcome::One(
                *matches.into_iter().next().unwrap(),
            ),
            _ => LookupOutcome::Many,
        }
    }

    pub fn find_best_match(
        &self,
        tags: Vec<String>,
        prefer_exact: bool,
    ) -> Match {
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
            for tag in
                &self.pages_by_id.get(&page_id).unwrap().tags
            {
                if !matching_tags.contains(&tag) {
                    *narrowing_tags
                        .entry(tag.clone())
                        .or_insert(0) += 1;
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
                    let page_id =
                        matches.into_iter().next().unwrap();
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
                                    .all(|tag| {
                                        tags.contains(&tag)
                                    })
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
            inner: sync::Arc::new(sync::RwLock::new(
                State::new(),
            )),
        }
    }
    pub fn write<'a>(
        &'a self,
    ) -> sync::RwLockWriteGuard<'a, State> {
        self.inner.write().unwrap()
    }
    pub fn read<'a>(
        &'a self,
    ) -> sync::RwLockReadGuard<'a, State> {
        self.inner.read().unwrap()
    }

    pub fn write_new_file(
        &self,
        page: &Page,
        data_dir: &Path,
    ) -> Result<()> {
        let mut path_text = page.suggested_filename();

        println!("{}", path_text);
        let (mut tmp_file, tmp_file_path, dst_path) = loop {
            let dst_path = data_dir
                .join(path_text.clone())
                .with_extension("md");
            if dst_path.exists() {
                path_text += "_";
                continue;
            }
            // TODO: randomize
            let tmp_file_path = dst_path.with_extension("tmp");
            let res = File::create(tmp_file_path.clone());
            match res {
                Err(e) => if e.kind()
                    == ::std::io::ErrorKind::AlreadyExists
                {
                    path_text += "_";
                } else {
                    Err(e)?;
                },
                Ok(file) => {
                    break (file, tmp_file_path, dst_path)
                }
            }
        };
        tmp_file.write_all(page.md.as_bytes())?;
        tmp_file.flush()?;
        drop(tmp_file);
        fs::rename(tmp_file_path, dst_path.clone())?;
        let _ = self.handle_create(dst_path)?;

        Ok(())
    }

    pub fn replace_file(
        &self,
        path: &Path,
        page: &Page,
    ) -> Result<()> {
        // TODO: randomize
        let tmp_file_path = path.with_extension(".tmp");
        let mut tmp_file = File::create(tmp_file_path.clone())?;
        tmp_file.write_all(page.md.as_bytes())?;
        tmp_file.flush()?;
        drop(tmp_file);
        fs::rename(tmp_file_path.clone(), path)?;

        let _ = self.handle_rename(tmp_file_path, path.into())?;
        Ok(())
    }

    fn handle_create(&self, path: PathBuf) -> Result<()> {
        let new_page = Page::read_from_file(&*path)?;

        let mut inner = self.inner.write().unwrap();
        if let Some(id) =
            inner.pages_by_path.get(path.as_path()).cloned()
        {
            inner.remove(id);
        }
        inner.insert(new_page, &path.canonicalize()?);

        Ok(())
    }

    fn handle_remove(&self, path: PathBuf) -> Result<()> {
        let mut inner = self.inner.write().unwrap();
        if let Some(id) =
            inner.pages_by_path.get(path.as_path()).cloned()
        {
            inner.remove(id);
        }

        Ok(())
    }
    fn handle_rename(
        &self,
        src: PathBuf,
        dst: PathBuf,
    ) -> Result<()> {
        let new_page = Page::read_from_file(&*dst)?;

        let mut inner = self.inner.write().unwrap();
        if let Some(id) =
            inner.pages_by_path.get(dst.as_path()).cloned()
        {
            inner.remove(id);
        }
        if let Some(id) =
            inner.pages_by_path.get(src.as_path()).cloned()
        {
            inner.remove(id);
        }
        inner.insert(new_page, &dst.canonicalize()?);

        Ok(())
    }
}

pub struct FsWatcher {
    // TODO
    _join_handle: thread::JoinHandle<Result<()>>,
}

impl FsWatcher {
    pub fn new(dir: PathBuf, state: SyncState) -> Result<Self> {
        let (tx, rx) = sync::mpsc::channel();
        let mut watcher: RecommendedWatcher =
            Watcher::new(tx, Duration::from_millis(10))?;

        watcher.watch(dir, RecursiveMode::Recursive)?;

        let join_handle = thread::spawn(move || {
            let _watcher = watcher;
            loop {
                let event = rx.recv().unwrap();
                println!("{:?}", event);
                match event {
                    DebouncedEvent::Create(path) => {
                        if path
                            .extension()
                            .and_then(|e| e.to_str())
                            == Some("md")
                        {
                            let _ = state.handle_create(path)?;
                        }
                    }
                    DebouncedEvent::Remove(path) => {
                        if path
                            .extension()
                            .and_then(|e| e.to_str())
                            == Some("md")
                        {
                            let _ = state.handle_remove(path)?;
                        }
                    }
                    DebouncedEvent::Rename(src, dst) => {
                        if dst
                            .extension()
                            .and_then(|e| e.to_str())
                            == Some("md")
                        {
                            state.handle_rename(src, dst)?;
                        }
                    }
                    _ => {}
                }
            }
        });

        Ok(FsWatcher {
            _join_handle: join_handle,
        })
    }
}

#[test]
fn simple() {
    let mut state: State = Default::default();

    assert!(state.find_best_match(vec![], false).is_none());

    let _p1 = state.insert(
        Page {
            html: "".into(),
            tags: vec!["a".into(), "b".into()],
            title: "".into(),
            md: "".into(),
        },
        Path::new(""),
    );

    let _p2 = state.insert(
        Page {
            html: "".into(),
            tags: vec!["a".into(), "c".into()],
            title: "".into(),
            md: "".into(),
        },
        Path::new(""),
    );

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
    assert_eq!(
        m.matching_tags,
        vec!["a".to_string(), "b".into()]
    );
    assert_eq!(m.unmatched_tags, empty);

    let tags = vec!["a".into(), "b".into()];
    let m = state.find_best_match(tags.clone(), false);
    assert!(m.is_one());
    assert_eq!(
        m.matching_tags,
        vec!["a".to_string(), "b".into()]
    );
    assert_eq!(m.unmatched_tags, empty);

    let tags = vec!["a".to_string(), "x".into(), "b".into()];
    let m = state.find_best_match(tags.clone(), false);
    assert!(m.is_one());
    assert_eq!(
        m.matching_tags,
        vec!["a".to_string(), "b".into()]
    );
    assert_eq!(m.unmatched_tags, vec!["x".to_string()]);
}
