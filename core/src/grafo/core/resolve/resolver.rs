use std::borrow::Borrow;
use std::error::Error;
use std::hash::Hash;

use crate::grafo::core::graph_item::GraphItemBase;
use crate::grafo::layout_item::LayoutItemBase;
use crate::grafo::{IdTree, IdTreeError, NameIdError, NameRefIndex};
use crate::util::alias::{GroupId, ItemId};
use crate::util::either::Either;
use crate::util::kind::{AttributeKind, GraphItemKind, LayoutItemKind, WithItemLayoutKind};
use crate::util::name_type::NameType;
use crate::util::writer::DisplayAsJson;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ResolverError {
    FailSetRootGraphId,
    NotInitialized,
    NotFindParentId(GroupId),
    AlreadyExistId(GroupId),
}

impl std::fmt::Display for ResolverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ResolverError::FailSetRootGraphId => write!(f, "fail set group as root group"),
            ResolverError::NotInitialized => write!(f, "hierarchy of group is not initialized"),
            ResolverError::NotFindParentId(group_id) => {
                write!(f, "not found parent group by id {}", group_id)
            }
            ResolverError::AlreadyExistId(group_id) => {
                write!(f, "group with id {} already exist", group_id)
            }
        }
    }
}

impl Error for ResolverError {}

impl From<IdTreeError<GroupId>> for ResolverError {
    fn from(e: IdTreeError<GroupId>) -> ResolverError {
        match e {
            IdTreeError::NotInitialized => ResolverError::NotInitialized,
            IdTreeError::NotFindParentId(id) => ResolverError::NotFindParentId(id),
            IdTreeError::AlreadyExistId(id) => ResolverError::AlreadyExistId(id),
        }
    }
}

/// reference indexes for names
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct Resolver<Name: NameType> {
    group_id_tree: IdTree<GroupId>,
    /// names reference indexes name:(group_id, item_id)
    graph_items: NameRefIndex<Name, GraphItemKind, (GroupId, ItemId)>,
    /// layout reference indexes layout_type:value
    layout_items: NameRefIndex<Name, LayoutItemKind, ItemId>,
}

impl<Name: NameType> Default for Resolver<Name> {
    fn default() -> Self {
        Self {
            group_id_tree: IdTree::None,
            graph_items: NameRefIndex::new(),
            layout_items: NameRefIndex::new(),
        }
    }
}

impl<Name: NameType> DisplayAsJson for Resolver<Name> {
    fn fmt_as_json(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{{\"group_tree\": \"{}\", \"graph_items\": ",
            self.group_id_tree
        )?;
        self.graph_items.fmt_as_json(f)?;
        write!(f, ", \"layout_items\": ")?;
        self.layout_items.fmt_as_json(f)?;
        write!(f, "}}")
    }
}

impl<Name: NameType> std::fmt::Display for Resolver<Name> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "ReferenceResolver",)?;
        self.fmt_as_json(f)
    }
}

impl<Name: NameType> Resolver<Name> {
    pub fn new() -> Self {
        Default::default()
    }

    //
    // for root group
    //
    pub(crate) fn set_root_group_id(&mut self, group_id: GroupId) -> Result<(), ResolverError> {
        if self.group_id_tree.is_some() {
            return Err(ResolverError::FailSetRootGraphId);
        }
        self.group_id_tree = IdTree::new(group_id);
        Ok(())
    }

    pub(crate) fn insert_group(
        &mut self,
        parent: GroupId,
        child: GroupId,
    ) -> Result<(), ResolverError> {
        self.group_id_tree
            .insert_id(parent, child)
            .map_err(|e| e.into())
    }

    pub(crate) fn get_root_group_id(&self) -> Result<GroupId, ResolverError> {
        match self.group_id_tree.get_root_id() {
            Ok(id) => Ok(id),
            Err(e) => Err(e.into()),
        }
    }

    pub fn get_belong_group<S: ?Sized>(
        &self,
        name: Option<&S>,
    ) -> Result<GroupId, Either<NameIdError<Name, GraphItemKind>, ResolverError>>
    where
        Name: Borrow<S>,
        S: ToOwned<Owned = Name> + Hash + Eq,
    {
        match name {
            None => {
                let root_id = self.get_root_group_id().map_err(Either::Right)?;
                Ok(root_id)
            }
            Some(n) => self
                .get_graph_item_id_pair(GraphItemKind::Group, n)
                .map(|(_, item_id)| item_id)
                .map_err(Either::Left),
        }
    }

    /// get parent and ancestors id
    pub fn get_ancestor_ids(&self, group_id: GroupId) -> Option<Vec<GroupId>> {
        self.group_id_tree.get_ancestor_ids(group_id)
    }

    pub fn group_tree_to_string(&self) -> String {
        self.group_id_tree.to_string()
    }

    //
    // for whole item resolve
    //

    pub fn count_usable_whole_layout_item_names(&self) -> usize {
        self.layout_items.count_usable_names_all()
    }

    pub fn count_registered_whole_layout_names(&self) -> usize {
        self.layout_items.count_registered_names_all()
    }

    //
    // for graph item
    //

    pub(crate) fn push_graph_item_value_or_override<S: Into<Name>>(
        &mut self,
        item_kind: GraphItemKind,
        name: S,
        group_id: GroupId,
        item_id: ItemId,
    ) -> Result<(), NameIdError<Name, GraphItemKind>> {
        self.graph_items
            .push_value_or_override(item_kind, name, (group_id, item_id))
    }

    pub fn get_graph_item_id_pair<S: ?Sized>(
        &self,
        item_kind: GraphItemKind,
        name: &S,
    ) -> Result<(GroupId, ItemId), NameIdError<Name, GraphItemKind>>
    where
        Name: Borrow<S>,
        S: ToOwned<Owned = Name> + Hash + Eq,
    {
        self.graph_items
            .get_value(item_kind, name)
            .ok_or_else(|| NameIdError::NotExist(item_kind, name.to_owned()))
    }

    pub fn get_graph_item_name_by(
        &self,
        item_kind: GraphItemKind,
        group_id: GroupId,
        item_id: ItemId,
    ) -> Option<&Name> {
        self.graph_items.get_name(item_kind, (group_id, item_id))
    }

    pub fn get_graph_item_name_by_item<I: GraphItemBase>(&self, item: &I) -> Option<&Name> {
        self.graph_items.get_name(
            item.get_kind(),
            (item.get_belong_group_id(), item.get_item_id()),
        )
    }

    pub fn is_usable_graph_item_name<S: ?Sized>(&self, item_kind: GraphItemKind, name: &S) -> bool
    where
        Name: Borrow<S>,
        S: ToOwned<Owned = Name> + Hash + Eq,
    {
        self.graph_items.is_usable_name(item_kind, name)
    }

    pub fn has_registered_graph_item_name(
        &self,
        item_kind: GraphItemKind,
        group_id: GroupId,
        item_id: ItemId,
    ) -> bool {
        self.graph_items
            .has_registered_name(item_kind, (group_id, item_id))
    }

    pub fn count_usable_graph_item_names(&self) -> usize {
        self.graph_items.count_usable_names_all()
    }

    pub fn count_registered_graph_item_names(&self) -> usize {
        self.graph_items.count_registered_names_all()
    }

    pub fn count_usable_graph_item_names_by(&self, item_kind: GraphItemKind) -> usize {
        self.graph_items.count_usable_names_by(item_kind)
    }

    pub fn count_registered_graph_item_names_by(&self, item_kind: GraphItemKind) -> usize {
        self.graph_items.count_registered_names_by(item_kind)
    }

    //
    // for layout with graph item
    //

    pub(crate) fn push_graph_item_layout_value<S: Into<Name>>(
        &mut self,
        item_kind: GraphItemKind,
        layout_kind: WithItemLayoutKind,
        name: S,
        layout_item_id: ItemId,
    ) -> Result<(), NameIdError<Name, LayoutItemKind>> {
        self.layout_items.push_value_or_override(
            LayoutItemKind::new_layout(item_kind, layout_kind),
            name.into(),
            layout_item_id,
        )
    }

    pub fn get_graph_item_layout_id<S: ?Sized>(
        &self,
        item_kind: GraphItemKind,
        layout_kind: WithItemLayoutKind,
        name: &S,
    ) -> Result<ItemId, NameIdError<Name, LayoutItemKind>>
    where
        Name: Borrow<S>,
        S: ToOwned<Owned = Name> + Hash + Eq,
    {
        let kind = LayoutItemKind::new_layout(item_kind, layout_kind);
        self.layout_items
            .get_value(kind, name)
            .ok_or_else(|| NameIdError::NotExist(kind, name.to_owned()))
    }

    pub fn get_graph_item_layout_name_by(
        &self,
        item_kind: GraphItemKind,
        layout_kind: WithItemLayoutKind,
        item_id: ItemId,
    ) -> Option<&Name> {
        self.layout_items
            .get_name(LayoutItemKind::new_layout(item_kind, layout_kind), item_id)
    }

    pub fn get_graph_item_layout_name_by_item<I: GraphItemBase>(
        &self,
        layout_kind: WithItemLayoutKind,
        item: &I,
    ) -> Option<&Name> {
        self.layout_items.get_name(
            LayoutItemKind::WithItemAttribute(item.get_kind(), layout_kind),
            item.get_item_id(),
        )
    }

    pub fn is_usable_graph_item_layout_name<S: ?Sized>(
        &self,
        item_kind: GraphItemKind,
        layout_kind: WithItemLayoutKind,
        name: &S,
    ) -> bool
    where
        Name: Borrow<S>,
        S: ToOwned<Owned = Name> + Hash + Eq,
    {
        self.layout_items
            .is_usable_name(LayoutItemKind::new_layout(item_kind, layout_kind), name)
    }

    pub fn has_registered_graph_item_layout_name(
        &self,
        item_kind: GraphItemKind,
        layout_kind: WithItemLayoutKind,
        item_id: ItemId,
    ) -> bool {
        self.layout_items
            .has_registered_name(LayoutItemKind::new_layout(item_kind, layout_kind), item_id)
    }

    pub fn count_usable_graph_item_layout_names(&self) -> usize {
        self.layout_items
            .count_usable_names_filtered_by(|k| k.need_graph_item())
    }

    pub fn count_registered_graph_item_layout_names(&self) -> usize {
        self.layout_items
            .count_registered_names_filtered_by(|k| k.need_graph_item())
    }

    pub fn count_usable_graph_item_layout_names_by(
        &self,
        item_kind: GraphItemKind,
        layout_kind: WithItemLayoutKind,
    ) -> usize {
        self.layout_items
            .count_usable_names_by(LayoutItemKind::new_layout(item_kind, layout_kind))
    }

    pub fn count_registered_graph_item_layout_names_by(
        &self,
        item_kind: GraphItemKind,
        layout_kind: WithItemLayoutKind,
    ) -> usize {
        self.layout_items
            .count_registered_names_by(LayoutItemKind::new_layout(item_kind, layout_kind))
    }

    //
    //  for layout without graph item
    //

    pub(crate) fn push_attribute_value<S: Into<Name>>(
        &mut self,
        attribute_kind: AttributeKind,
        name: S,
        layout_item_id: ItemId,
    ) -> Result<(), NameIdError<Name, LayoutItemKind>> {
        self.layout_items.push_value_or_override(
            LayoutItemKind::new_attribute(attribute_kind),
            name,
            layout_item_id,
        )
    }

    pub fn get_attribute_item_id<S: ?Sized>(
        &self,
        attribute_kind: AttributeKind,
        name: &S,
    ) -> Result<ItemId, NameIdError<Name, LayoutItemKind>>
    where
        Name: Borrow<S>,
        S: ToOwned<Owned = Name> + Hash + Eq,
    {
        let kind = LayoutItemKind::new_attribute(attribute_kind);
        self.layout_items
            .get_value(kind, name)
            .ok_or_else(|| NameIdError::NotExist(kind, name.to_owned()))
    }

    pub fn get_attribute_name_by(
        &self,
        attribute_kind: AttributeKind,
        item_id: ItemId,
    ) -> Option<&Name> {
        self.layout_items
            .get_name(LayoutItemKind::new_attribute(attribute_kind), item_id)
    }

    pub fn get_attribute_name_by_item<I: LayoutItemBase>(&self, item: &I) -> Option<&Name> {
        self.layout_items.get_name(
            LayoutItemKind::IsolateAttribute(item.get_kind()),
            item.get_item_id(),
        )
    }

    pub fn is_usable_attribute_name<S: ?Sized>(
        &self,
        attribute_kind: AttributeKind,
        name: &S,
    ) -> bool
    where
        Name: Borrow<S>,
        S: ToOwned<Owned = Name> + Hash + Eq,
    {
        self.layout_items
            .is_usable_name(LayoutItemKind::new_attribute(attribute_kind), name)
    }

    pub fn has_registered_attribute_name(
        &self,
        attribute_kind: AttributeKind,
        item_id: ItemId,
    ) -> bool {
        self.layout_items
            .has_registered_name(LayoutItemKind::new_attribute(attribute_kind), item_id)
    }

    pub fn count_usable_whole_attribute_names(&self) -> usize {
        self.layout_items
            .count_usable_names_filtered_by(|k| k.is_attribute())
    }

    pub fn count_registered_whole_attribute_names(&self) -> usize {
        self.layout_items
            .count_registered_names_filtered_by(|k| k.is_attribute())
    }

    pub fn count_usable_attribute_names_by(&self, attribute_kind: AttributeKind) -> usize {
        self.layout_items
            .count_usable_names_by(LayoutItemKind::new_attribute(attribute_kind))
    }

    pub fn count_registered_attribute_names_by(&self, attribute_kind: AttributeKind) -> usize {
        self.layout_items
            .count_registered_names_by(LayoutItemKind::new_attribute(attribute_kind))
    }
}
