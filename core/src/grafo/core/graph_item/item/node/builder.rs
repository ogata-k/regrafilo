//! module for Node builder

use crate::grafo::core::graph_item::item::node::NodeItemOption;
use crate::grafo::core::graph_item::node::{NodeItem, NodeItemError};
use crate::grafo::core::graph_item::GraphItemBuilderBase;
use crate::grafo::core::resolve::Resolver;
use crate::grafo::{GrafoError, NameIdError};
use crate::util::alias::{GroupId, ItemId};
use crate::util::either::Either;
use crate::util::item_base::{
    FromWithItemId, HasItemBuilderMethod, ItemBuilderBase, ItemBuilderResult,
};
use crate::util::kind::HasGraphItemKind;
use crate::util::name_type::NameType;

#[derive(Debug, Clone)]
pub struct NodeItemBuilder<Name: NameType> {
    belong_group: Option<Name>,
    name: Option<Name>,
}

impl<Name: NameType> ItemBuilderBase<Name> for NodeItemBuilder<Name> {
    type Item = NodeItem;
    type ItemError = NodeItemError<Name>;
}

impl<Name: NameType> GraphItemBuilderBase<Name> for NodeItemBuilder<Name> {
    fn set_belong_group<S: Into<Name>>(&mut self, group: S) -> &mut Self {
        self.belong_group = Some(group.into());
        self
    }

    fn set_name<S: Into<Name>>(&mut self, name: S) -> &mut Self {
        self.name = Some(name.into());
        self
    }
}

impl<Name: NameType> HasItemBuilderMethod<Name> for NodeItemBuilder<Name> {
    type ItemOption = NodeItemOption<Name>;
    fn build(
        self,
        item_id: ItemId,
        resolver: &Resolver<Name>,
    ) -> ItemBuilderResult<Name, Self::Item, Self::ItemOption> {
        let NodeItemBuilder {
            belong_group: belong_group_name,
            name,
        } = self;
        let mut errors: Vec<GrafoError<Name>> = Vec::new();
        let belong_group: Option<(GroupId, ItemId)> =
            Self::resolve_belong_group(belong_group_name, item_id, resolver, &mut errors);
        let item: Option<NodeItem> = Self::resolve_item(item_id, &mut errors, belong_group);
        let item_option: Option<NodeItemOption<Name>> =
            Self::resolve_item_option(name, item_id, resolver, &mut errors);

        match (item, item_option) {
            (Some(i), Some(o)) => (Some((i, o)), errors),
            (_, _) => (None, errors),
        }
    }
}

impl<Name: NameType> NodeItemBuilder<Name> {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            belong_group: None,
            name: None,
        }
    }

    fn resolve_belong_group(
        belong_group: Option<Name>,
        item_id: ItemId,
        resolver: &Resolver<Name>,
        errors: &mut Vec<GrafoError<Name>>,
    ) -> Option<(GroupId, ItemId)> {
        // @fixme push以外は&Sと参照を受け取るようにしたい（参照に直せたら以前のself.hoge()形式に戻す. なおselfはここで奪う）
        match resolver.get_belong_group(belong_group) {
            Ok(group) => Some(group),
            Err(Either::Left(e)) => {
                errors.push(NodeItemError::from_with_id(item_id, e).into());
                None
            }
            Err(Either::Right(e)) => {
                errors.push(e.into());
                None
            }
        }
    }

    fn resolve_item(
        item_id: ItemId,
        errors: &mut Vec<GrafoError<Name>>,
        resolved_belong_group: Option<(GroupId, ItemId)>,
    ) -> Option<NodeItem> {
        // @fixme push以外は&Sと参照を受け取るようにしたい（参照に直せたら以前のself.hoge()形式に戻す）
        let mut validate = true;
        if resolved_belong_group.is_none() {
            errors.push(NodeItemError::FailResolveBelongGroup(item_id).into());
            validate = false;
        }

        if validate {
            Some(NodeItem::new(resolved_belong_group.unwrap().1, item_id))
        } else {
            None
        }
    }

    fn resolve_item_option(
        name: Option<Name>,
        item_id: ItemId,
        resolver: &Resolver<Name>,
        errors: &mut Vec<GrafoError<Name>>,
    ) -> Option<NodeItemOption<Name>> {
        // @fixme push以外は&Sと参照を受け取るようにしたい（参照に直せたら以前のself.hoge()形式に戻す）
        if let Some(n) = name.clone() {
            if resolver.contains_name_graph_item(NodeItem::kind(), n.clone()) {
                errors.push(
                    NodeItemError::from_with_id(
                        item_id,
                        NameIdError::AlreadyExist(NodeItem::kind(), n),
                    )
                    .into(),
                );
            }
        }
        Some(NodeItemOption { name })
    }
}
