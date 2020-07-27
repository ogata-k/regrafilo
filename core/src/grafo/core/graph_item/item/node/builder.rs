//! module for Node builder

use crate::grafo::core::graph_item::item::node::NodeItemOption;
use crate::grafo::core::graph_item::node::{NodeItem, NodeItemError};
use crate::grafo::core::graph_item::GraphItemBuilderBase;
use crate::grafo::core::resolve::Resolver;
use crate::grafo::{GrafoError, NameIdError};
use crate::util::alias::{GroupId, ItemId};
use crate::util::item_base::{
    FromWithItemId, HasItemBuilderMethod, ItemBuilderBase, ItemBuilderResult,
};
use crate::util::kind::{HasGraphItemKind, NameKind};

#[derive(Debug, Clone)]
pub struct NodeItemBuilder {
    belong_group: Option<String>,
    name: Option<String>,
}

impl ItemBuilderBase for NodeItemBuilder {
    type Item = NodeItem;
    type ItemError = NodeItemError;
}

impl GraphItemBuilderBase for NodeItemBuilder {
    fn set_belong_group<S: Into<String>>(&mut self, group: S) -> &mut Self {
        self.belong_group = Some(group.into());
        self
    }

    fn set_name<S: Into<String>>(&mut self, name: S) -> &mut Self {
        self.name = Some(name.into());
        self
    }
}

impl HasItemBuilderMethod for NodeItemBuilder {
    type ItemOption = NodeItemOption;
    fn build(
        self,
        item_id: ItemId,
        resolver: &Resolver,
    ) -> ItemBuilderResult<Self::Item, Self::ItemOption> {
        let mut errors: Vec<GrafoError> = Vec::new();
        let belong_group: Option<(GroupId, ItemId)> =
            self.resolve_belong_group(item_id, resolver, &mut errors);
        let item: Option<NodeItem> = self.resolve_item(item_id, &mut errors, belong_group);
        let item_option: Option<NodeItemOption> =
            self.resolve_item_option(item_id, resolver, &mut errors);

        match (item, item_option) {
            (Some(i), Some(o)) => (Some((i, o)), errors),
            (_, _) => (None, errors),
        }
    }
}

impl NodeItemBuilder {
    pub fn new() -> Self {
        Self {
            belong_group: None,
            name: None,
        }
    }

    fn resolve_belong_group(
        &self,
        item_id: ItemId,
        resolver: &Resolver,
        errors: &mut Vec<GrafoError>,
    ) -> Option<(GroupId, ItemId)> {
        match resolver.get_belong_group(self.belong_group.as_deref()) {
            Ok(group) => Some(group),
            Err(e) => {
                errors.push(NodeItemError::from_with_id(item_id, e).into());
                None
            }
        }
    }

    fn resolve_item_option(
        self,
        item_id: ItemId,
        resolver: &Resolver,
        errors: &mut Vec<GrafoError>,
    ) -> Option<NodeItemOption> {
        let NodeItemBuilder {
            belong_group: _,
            name,
        } = self;
        if let Some(n) = &name {
            if resolver.contains_name_graph_item(NodeItem::kind(), n) {
                errors.push(
                    NodeItemError::from_with_id(
                        item_id,
                        NameIdError::AlreadyExist(
                            NodeItem::kind(),
                            NameKind::ItemName,
                            n.to_string(),
                        ),
                    )
                    .into(),
                );
            }
        }
        Some(NodeItemOption { name })
    }

    fn resolve_item(
        &self,
        item_id: ItemId,
        errors: &mut Vec<GrafoError>,
        resolved_belong_group: Option<(GroupId, ItemId)>,
    ) -> Option<NodeItem> {
        let mut validate = true;
        if resolved_belong_group.is_none() {
            errors.push(NodeItemError::FailResolveBelongGroup(item_id).into());
            validate = false;
        }

        if validate {
            Some(NodeItem::new(resolved_belong_group.unwrap().1))
        } else {
            None
        }
    }
}
