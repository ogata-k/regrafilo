//! item pool

use std::collections::btree_map::{Iter, Range};
use std::collections::BTreeMap;
use std::ops::{Bound, RangeBounds};
use std::sync::{Arc, Mutex};

use crate::grafo::graph_item::{GraphBuilderErrorBase, GraphItemBase, GraphItemBuilderBase};
use crate::grafo::GrafoError;
use crate::grafo::Resolver;
use crate::util::alias::{GraphItemId, GroupId, DEFAULT_ITEM_ID};
use crate::util::item_base::HasItemBuilderMethod;
use crate::util::kind::GraphItemKind;

/// item pool
#[derive(Debug, Clone)]
pub struct ItemArena<I> {
    pushed_index: Arc<Mutex<GraphItemId>>,
    /// (GroupId, ItemId) => Item
    arena: BTreeMap<(GroupId, GraphItemId), I>,
}

fn range_with_group(
    group_id: GroupId,
    bound: Bound<&GraphItemId>,
) -> Bound<(GroupId, GraphItemId)> {
    match bound {
        Bound::Included(item_id) => Bound::Included((group_id, *item_id)),
        Bound::Excluded(item_id) => Bound::Excluded((group_id, *item_id)),
        Bound::Unbounded => Bound::Unbounded,
    }
}

impl<I: GraphItemBase> ItemArena<I> {
    /// initialize
    pub fn new() -> Self {
        ItemArena::default()
    }

    //
    // helper
    //

    /// get the next index with increment as soon as possible
    fn get_push_index(&mut self) -> GraphItemId {
        match self.pushed_index.lock() {
            Ok(mut pushed_index) => {
                *pushed_index += 1;
                *pushed_index
            }
            Err(e) => {
                panic!("fail lock error: {}", e);
            }
        }
    }

    //
    // setter
    //

    /// push the item into arena with action for conclusion<br/>
    /// F: fn(item_kind, group_id, Result<(item_id, extension), err>)
    pub(crate) fn push<
        F,
        O,
        E: GraphBuilderErrorBase,
        B: GraphItemBuilderBase + HasItemBuilderMethod<Item = I, ItemOption = O, BuilderError = E>,
    >(
        &mut self,
        resolver: &mut Resolver,
        item_builder: B,
        action: F,
    ) -> (bool, Vec<GrafoError>)
    where
        F: FnOnce(
            &mut Resolver,
            GraphItemKind,
            GroupId,
            GraphItemId,
            B::ItemOption,
        ) -> (bool, Vec<GrafoError>),
    {
        let (item_option, mut errors) = item_builder.build(resolver);
        match item_option {
            None => (false, errors),
            Some((item, option)) => {
                let group_id = item.get_belong_group_id();
                let push_index = self.get_push_index();
                let (result, action_errors) =
                    action(resolver, item.get_kind(), group_id, push_index, option);
                errors.extend(action_errors);
                if result {
                    self.arena.insert((group_id, push_index), item);
                }
                (result, errors)
            }
        }
    }

    /// item getter
    pub fn get(&self, group_id: GroupId, index: GraphItemId) -> Option<&I> {
        self.arena.get(&(group_id, index))
    }

    /// item getter by range
    pub fn range<R: RangeBounds<GraphItemId>>(
        &self,
        group_id: GroupId,
        range: R,
    ) -> Range<(GroupId, GraphItemId), I> {
        let start = range_with_group(group_id, range.start_bound());
        let end = range_with_group(group_id, range.end_bound());
        self.arena.range((start, end))
    }

    /// iter by filtering group_id
    pub fn filter_by_group<'a>(&'a self, group_id: GroupId) -> impl Iterator + 'a {
        self.iter()
            .filter_map(move |((item_group_id, _item_id), item)| {
                if item_group_id == &group_id {
                    Some(item)
                } else {
                    None
                }
            })
    }

    //
    // reference
    //

    /// count of item
    pub fn count(&self) -> usize {
        self.arena.len()
    }

    /// item pool is empty
    pub fn is_empty(&self) -> bool {
        self.count() == 0
    }

    //
    // iter or slice
    //

    /// to iterator
    pub fn iter(&self) -> Iter<(GroupId, GraphItemId), I> {
        self.arena.iter()
    }
}

impl<I: GraphItemBase + Default> ItemArena<I> {
    fn get_default_index(&self) -> GraphItemId {
        DEFAULT_ITEM_ID
    }

    /// push the item into arena with action for conclusion
    pub(crate) fn push_default<F, O: Default>(&mut self, resolver: &mut Resolver, action: F)
    where
        F: FnOnce(&mut Resolver, GraphItemKind, GroupId, GraphItemId, O) -> (bool, Vec<GrafoError>),
    {
        let item = I::default();
        let group_id = item.get_belong_group_id();
        let push_index = self.get_default_index();
        let (result, errors) = action(
            resolver,
            item.get_kind(),
            group_id,
            push_index,
            O::default(),
        );
        if !result || !errors.is_empty() {
            let errors_str: Vec<String> = errors.into_iter().map(|e| format!("{}", e)).collect();
            panic!("{}", errors_str.as_slice().join("\n"));
        }

        self.arena.insert((group_id, push_index), item);
    }

    /// push the item into arena with action for conclusion<br/>
    pub(crate) fn push_user_item_as_default<
        F,
        O,
        E: GraphBuilderErrorBase,
        B: GraphItemBuilderBase + HasItemBuilderMethod<Item = I, ItemOption = O, BuilderError = E>,
    >(
        &mut self,
        resolver: &mut Resolver,
        item_builder: B,
        action: F,
    ) -> (bool, Vec<GrafoError>)
    where
        F: FnOnce(
            &mut Resolver,
            GraphItemKind,
            GroupId,
            GraphItemId,
            B::ItemOption,
        ) -> (bool, Vec<GrafoError>),
    {
        let (item_option, mut errors) = item_builder.build(resolver);
        match item_option {
            None => (false, errors),
            Some((item, option)) => {
                let group_id = item.get_belong_group_id();
                let push_index = self.get_default_index();
                let (result, action_errors) =
                    action(resolver, item.get_kind(), group_id, push_index, option);
                errors.extend(action_errors);
                if result {
                    self.arena.insert((group_id, push_index), item);
                }
                (result, errors)
            }
        }
    }

    /// item getter
    pub(crate) fn get_default(&self, group_id: GroupId) -> Option<&I> {
        self.arena.get(&(group_id, self.get_default_index()))
    }
}

impl<I> Default for ItemArena<I> {
    /// initialize without log
    fn default() -> Self {
        ItemArena {
            pushed_index: Arc::new(Mutex::new(DEFAULT_ITEM_ID)),
            arena: Default::default(),
        }
    }
}

#[cfg(test)]
mod test {
    use std::fmt::{Display, Formatter};

    use crate::grafo::core::graph_item::{
        GraphBuilderErrorBase, GraphItemBase, GraphItemBuilderBase, ItemArena,
    };
    use crate::grafo::core::{NameIdError, Resolver};
    use crate::grafo::GrafoError;
    use crate::util::alias::{GraphItemId, GroupId};
    use crate::util::item_base::{
        HasItemBuilderMethod, ItemBase, ItemBuilderBase, ItemBuilderErrorBase, ItemBuilderResult,
    };
    use crate::util::kind::test::graph_item_check_list;
    use crate::util::kind::{GraphItemKind, HasGraphItemKind};
    use std::error::Error;

    const ITERATE_COUNT: usize = 10;
    const TARGET_KIND: GraphItemKind = GraphItemKind::Node;

    #[derive(Debug, Eq, PartialEq, Clone)]
    struct TargetItemBuilder {
        belong_group: Option<String>,
        name: Option<String>,
    }

    #[derive(Debug, Eq, PartialEq, Clone)]
    struct TargetItem {
        belong_group_id: GraphItemId,
    }

    #[derive(Debug, Eq, PartialEq, Clone)]
    struct TargetItemOption {
        belong_group_id: GraphItemId,
        name: Option<String>,
    }

    #[derive(Debug, Eq, PartialEq, Clone)]
    enum TargetBuilderError {
        BuildFail,
        NotFindGroup,
    }

    impl Into<GrafoError> for TargetBuilderError {
        fn into(self) -> GrafoError {
            unimplemented!()
        }
    }

    impl HasGraphItemKind for TargetItem {
        fn kind() -> GraphItemKind {
            TARGET_KIND
        }
    }

    impl HasGraphItemKind for TargetBuilderError {
        fn kind() -> GraphItemKind {
            TARGET_KIND
        }
    }

    impl ItemBuilderBase for TargetItemBuilder {
        type Item = TargetItem;
        type ItemOption = TargetItemOption;
        type BuilderError = TargetBuilderError;
    }

    impl GraphItemBuilderBase for TargetItemBuilder {
        fn set_belong_group<S: Into<String>>(&mut self, group: S) -> &mut Self {
            self.belong_group = Some(group.into());
            self
        }

        fn set_name<S: Into<String>>(&mut self, name: S) -> &mut Self {
            self.name = Some(name.into());
            self
        }
    }

    impl TargetItemBuilder {
        fn new() -> Self {
            TargetItemBuilder {
                belong_group: None,
                name: None,
            }
        }
        fn get_belong_group(
            &self,
            resolver: &Resolver,
            errors: &mut Vec<GrafoError>,
            belong_group: Option<&str>,
        ) -> Option<GroupId> {
            match belong_group {
                None => Some(resolver.get_root_group_id()),
                Some(belong_group_name) => {
                    let belong_group_result =
                        resolver.get_item_id_pair(GraphItemKind::Group, &belong_group_name);
                    match belong_group_result {
                        Ok((_belong_group_id, item_id)) => Some(item_id),
                        Err(err) => {
                            errors.push(TargetBuilderError::from(err).into());
                            None
                        }
                    }
                }
            }
        }
    }

    impl HasItemBuilderMethod for TargetItemBuilder {
        fn build(self, resolver: &Resolver) -> ItemBuilderResult<TargetItem, TargetItemOption> {
            assert_ne!(TARGET_KIND, GraphItemKind::Group);
            let mut errors: Vec<GrafoError> = Vec::new();

            let group_id =
                (&self).get_belong_group(&resolver, &mut errors, self.belong_group.as_deref());
            if group_id.is_none() {
                errors.push(TargetBuilderError::NotFindGroup.into());
                return (None, errors);
            }
            let group_id = group_id.unwrap();

            let TargetItemBuilder {
                belong_group: _,
                name,
            } = self;
            if errors.is_empty() {
                (
                    Some((
                        TargetItem {
                            belong_group_id: group_id,
                        },
                        TargetItemOption {
                            belong_group_id: group_id,
                            name,
                        },
                    )),
                    errors,
                )
            } else {
                (None, errors)
            }
        }
    }

    impl ItemBase for TargetItem {}

    impl GraphItemBase for TargetItem {
        fn get_belong_group_id(&self) -> usize {
            self.belong_group_id
        }
    }

    impl Display for TargetBuilderError {
        fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
            use TargetBuilderError::*;
            match &self {
                BuildFail => write!(f, "fail build item"),
                NotFindGroup => write!(f, "fail found belong group"),
            }
        }
    }

    impl Error for TargetBuilderError {}

    impl ItemBuilderErrorBase for TargetBuilderError {}

    impl From<NameIdError<GraphItemKind>> for TargetBuilderError {
        fn from(error: NameIdError<GraphItemKind>) -> Self {
            unimplemented!()
        }
    }

    impl GraphBuilderErrorBase for TargetBuilderError {}

    #[test]
    fn is_empty() {
        assert!(ItemArena::<TargetItem>::new().is_empty());
    }

    #[test]
    fn with_name_count() {
        let mut arena_mut = ItemArena::<TargetItem>::new();
        let mut resolver = Resolver::default();
        resolver.set_root_group_id(0);
        for i in 0..ITERATE_COUNT {
            let mut builder = TargetItemBuilder::new();
            builder.set_name(format!("{}", i));
            let (result, errors) = arena_mut.push(
                &mut resolver,
                builder,
                |resolver, kind, group_id, item_id, option| {
                    let mut errors: Vec<GrafoError> = Vec::new();
                    if let TargetItemOption {
                        belong_group_id: _,
                        name: Some(name),
                    } = option
                    {
                        if let Err(err) = resolver.push_item_name(kind, name, group_id, item_id) {
                            errors.push(TargetBuilderError::from(err).into());
                        }
                    }
                    (errors.is_empty(), errors)
                },
            );
            assert_eq!(Vec::<GrafoError>::new(), errors);
            assert!(result);
        }
        let arena = arena_mut;
        assert_eq!(arena.count(), ITERATE_COUNT);
        for target in graph_item_check_list() {
            assert_eq!(
                resolver.item_name_count_by(target),
                if target == TARGET_KIND {
                    ITERATE_COUNT
                } else {
                    0
                }
            );
        }
    }

    #[test]
    fn with_name_each_eq() {
        let mut arena_mut = ItemArena::<TargetItem>::new();
        let mut resolver = Resolver::default();
        resolver.set_root_group_id(0);

        for i in 1..=ITERATE_COUNT {
            let mut builder = TargetItemBuilder::new();
            builder.set_name(format!("{}", i));
            let (result, errors) = arena_mut.push(
                &mut resolver,
                builder,
                |resolver, kind, group_id, item_id, option| {
                    let mut errors: Vec<GrafoError> = Vec::new();
                    if let TargetItemOption {
                        belong_group_id: _,
                        name: Some(name),
                    } = option
                    {
                        if let Err(err) = resolver.push_item_name(kind, name, group_id, item_id) {
                            errors.push(TargetBuilderError::from(err).into());
                        }
                    }

                    (errors.is_empty(), errors)
                },
            );
            assert_eq!(Vec::<GrafoError>::new(), errors);
            assert!(result);
        }
        let arena = arena_mut;
        for (index, item) in (&arena).iter() {
            for kind in graph_item_check_list() {
                let name = format!("{}", index.1);
                let ref_result = resolver.get_item_id_pair(kind, &name);
                if let Ok(success) = ref_result {
                    // デフォルトがitem_id = 0占有
                    assert_eq!(success, *index);
                } else {
                    assert_eq!(
                        ref_result,
                        Err(NameIdError::NotExist(kind, format!("{}", index.1)))
                    );
                }
            }
        }
    }

    #[test]
    fn mixed_count() {
        let mut arena_mut = ItemArena::<TargetItem>::new();
        let mut resolver = Resolver::default();
        resolver.set_root_group_id(0);
        for i in 1..=2 * ITERATE_COUNT {
            let mut builder = TargetItemBuilder::new();
            if i <= ITERATE_COUNT {
                builder.set_name(format!("{}", i));
            }
            let (result, errors) = arena_mut.push(
                &mut resolver,
                builder,
                |resolver, kind, group_id, item_id, option| {
                    let mut errors: Vec<GrafoError> = Vec::new();
                    if let TargetItemOption {
                        belong_group_id: _,
                        name: Some(name),
                    } = option
                    {
                        if let Err(err) = resolver.push_item_name(kind, name, group_id, item_id) {
                            errors.push(TargetBuilderError::from(err).into());
                        }
                    }

                    (errors.is_empty(), errors)
                },
            );
            assert_eq!(Vec::<GrafoError>::new(), errors);
            assert!(result)
        }
        let arena = arena_mut;
        assert_eq!(arena.count(), 2 * ITERATE_COUNT);
        for target in graph_item_check_list() {
            assert_eq!(
                resolver.item_name_count_by(target),
                if target == TARGET_KIND {
                    ITERATE_COUNT
                } else {
                    0
                }
            );
        }
    }

    #[test]
    fn mixed_each_eq() {
        let mut arena_mut = ItemArena::<TargetItem>::new();
        let mut resolver = Resolver::default();
        resolver.set_root_group_id(0);
        for i in 1..=2 * ITERATE_COUNT {
            let mut builder = TargetItemBuilder::new();
            if i <= ITERATE_COUNT {
                builder.set_name(format!("{}", i));
            }
            let (result, errors) = arena_mut.push(
                &mut resolver,
                builder,
                |resolver, kind, group_id, item_id, option| {
                    let mut errors: Vec<GrafoError> = Vec::new();
                    if let TargetItemOption {
                        belong_group_id: _,
                        name: Some(name),
                    } = option
                    {
                        if let Err(err) = resolver.push_item_name(kind, name, group_id, item_id) {
                            errors.push(TargetBuilderError::from(err).into());
                        }
                    }

                    (errors.is_empty(), errors)
                },
            );
            assert_eq!(Vec::<GrafoError>::new(), errors);
            assert!(result);
        }
        let arena = arena_mut;
        for (index, item) in (&arena).iter() {
            for kind in graph_item_check_list() {
                let name = format!("{}", index.1);
                let ref_result = resolver.get_item_id_pair(kind, &name);
                if index.1 <= ITERATE_COUNT && kind == TARGET_KIND {
                    if let Ok(success) = &ref_result {
                        // デフォルトがitem_id = 0占有
                        assert_eq!(success, index);
                    } else {
                        unreachable!("over count and not exist the name \"{}\"", name)
                    }
                } else {
                    assert_eq!(
                        ref_result,
                        Err(NameIdError::NotExist(kind, format!("{}", index.1)))
                    );
                }
            }
        }
    }
}