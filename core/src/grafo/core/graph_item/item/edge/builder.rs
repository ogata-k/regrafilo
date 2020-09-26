//! module for Edge item's builder

use crate::grafo::core::graph_item::edge::{EdgeItem, EdgeItemError, EdgeItemOption};
use crate::grafo::core::graph_item::GraphItemBuilderBase;
use crate::grafo::core::resolve::Resolver;
use crate::grafo::graph_item::edge::{EdgeItemStyle, Endpoint};
use crate::grafo::{GrafoError, NameIdError};
use crate::util::alias::{GroupId, ItemId};
use crate::util::either::Either;
use crate::util::item_base::{
    FromWithItemId, HasItemBuilderMethod, ItemBuilderBase, ItemBuilderResult,
};
use crate::util::kind::{GraphItemKind, HasGraphItemKind};
use crate::util::name_type::NameType;

/// builder for Group item
#[derive(Debug, Clone)]
pub struct EdgeItemBuilder<Name: NameType> {
    belong_group: Option<Name>,
    name: Option<Name>,
    label: Option<String>,
    style: Option<EdgeItemStyle>,
    start: Option<(GraphItemKind, Name)>,
    end: Option<(GraphItemKind, Name)>,
}

impl<Name: NameType> ItemBuilderBase<Name> for EdgeItemBuilder<Name> {
    type Item = EdgeItem;
    type ItemError = EdgeItemError<Name>;
}

impl<Name: NameType> GraphItemBuilderBase<Name> for EdgeItemBuilder<Name> {
    type ItemStyle = EdgeItemStyle;

    fn set_belong_group<S: Into<Name>>(&mut self, group: S) -> &mut Self {
        self.belong_group = Some(group.into());
        self
    }

    fn set_name<S: Into<Name>>(&mut self, name: S) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    fn set_label<S: Into<String>>(&mut self, label: S) -> &mut Self {
        self.label = Some(label.into());
        self
    }

    fn set_item_style(&mut self, style: Self::ItemStyle) -> &mut Self {
        self.style = Some(style);
        self
    }
}

impl<Name: NameType> HasItemBuilderMethod<Name> for EdgeItemBuilder<Name> {
    type ItemOption = EdgeItemOption<Name>;
    fn build(
        self,
        item_id: ItemId,
        resolver: &Resolver<Name>,
    ) -> ItemBuilderResult<Name, Self::Item, Self::ItemOption> {
        let mut errors: Vec<GrafoError<Name>> = Vec::new();
        let belong_group: Option<GroupId> =
            self.resolve_belong_group(item_id, resolver, &mut errors);
        let start: Option<(GraphItemKind, (GroupId, ItemId))> = if let Some(bg) = belong_group {
            self.resolve_endpoint(bg, item_id, &self.start, resolver, &mut errors, |item_id| {
                EdgeItemError::NotSpecifyStartEndpoint(
                    item_id,
                    self.name.clone(),
                    self.start.clone(),
                )
            })
        } else {
            None
        };
        let end: Option<(GraphItemKind, (GroupId, ItemId))> = if let Some(bg) = belong_group {
            self.resolve_endpoint(bg, item_id, &self.end, resolver, &mut errors, |item_id| {
                EdgeItemError::NotSpecifyEndEndpoint(item_id, self.name.clone(), self.end.clone())
            })
        } else {
            None
        };
        let (item, option): (Option<EdgeItem>, EdgeItemOption<Name>) =
            self.resolve_item(item_id, resolver, &mut errors, belong_group, start, end);

        match item {
            Some(i) => (Some((i, option)), errors),
            None => (None, errors),
        }
    }
}

// resolver
impl<Name: NameType> EdgeItemBuilder<Name> {
    /// resolve belong group from builder's parameter
    fn resolve_belong_group(
        &self,
        item_id: ItemId,
        resolver: &Resolver<Name>,
        errors: &mut Vec<GrafoError<Name>>,
    ) -> Option<ItemId> {
        match resolver.get_belong_group(self.belong_group.as_ref()) {
            Ok(group) => Some(group),
            Err(Either::Left(e)) => {
                errors.push(EdgeItemError::from_with_id(item_id, self.name.clone(), e).into());
                None
            }
            Err(Either::Right(e)) => {
                errors.push(EdgeItemError::from_with_id(item_id, self.name.clone(), e).into());
                None
            }
        }
    }

    /// resolve endpoint from builder's parameter
    fn resolve_endpoint<F>(
        &self,
        group_id: GroupId,
        item_id: ItemId,
        endpoint: &Option<(GraphItemKind, Name)>,
        resolver: &Resolver<Name>,
        errors: &mut Vec<GrafoError<Name>>,
        not_specify_error: F,
    ) -> Option<(GraphItemKind, (GroupId, ItemId))>
    where
        F: FnOnce(ItemId) -> EdgeItemError<Name>,
    {
        if let Some((kind, name)) = endpoint {
            match resolver.get_graph_item_id_pair(*kind, &name) {
                Ok((endpoint_group_id, endpoint_item_id)) => {
                    if *kind == GraphItemKind::Group {
                        let mut cannot_specify = group_id == endpoint_item_id;
                        if !cannot_specify {
                            if resolver.contains_group(group_id) {
                                cannot_specify = resolver
                                    .get_ancestor_ids(group_id)
                                    .contains(&endpoint_item_id);
                            } else {
                                // not stored graph id in id_tree
                                // usually unreachable!!
                                errors.push(
                                    EdgeItemError::from_with_id(
                                        item_id,
                                        self.name.clone(),
                                        NameIdError::NotExist(GraphItemKind::Group, name.clone()),
                                    )
                                    .into(),
                                );
                                cannot_specify = true;
                            }
                        }

                        if cannot_specify {
                            errors.push(
                                EdgeItemError::CannotSpecifyBelongGroupAsEndpoint(
                                    item_id,
                                    self.name.clone(),
                                    name.clone(),
                                )
                                .into(),
                            );
                            None
                        } else {
                            Some((*kind, (endpoint_group_id, endpoint_item_id)))
                        }
                    } else {
                        Some((*kind, (endpoint_group_id, endpoint_item_id)))
                    }
                }
                Err(e) => {
                    errors.push(EdgeItemError::from_with_id(item_id, self.name.clone(), e).into());
                    None
                }
            }
        } else {
            errors.push(not_specify_error(item_id).into());
            None
        }
    }

    /// resolve Edge item from builder's parameter
    fn resolve_item(
        self,
        item_id: ItemId,
        resolver: &Resolver<Name>,
        errors: &mut Vec<GrafoError<Name>>,
        resolved_belong_group: Option<ItemId>,
        resolved_start: Option<(GraphItemKind, (GroupId, ItemId))>,
        resolved_end: Option<(GraphItemKind, (GroupId, ItemId))>,
    ) -> (Option<EdgeItem>, EdgeItemOption<Name>) {
        let EdgeItemBuilder {
            belong_group,
            name,
            label,
            style,
            start,
            end,
        } = self;

        // todo?? if self use outer file, check file exist. but not fail build.

        let item = match (resolved_belong_group, resolved_start, resolved_end) {
            (None, _, _) => {
                errors.push(
                    EdgeItemError::FailResolveBelongGroup(item_id, name.clone(), belong_group)
                        .into(),
                );
                None
            }
            (Some(gid), Some(start), Some(end)) => {
                let (s_kind, (s_belong_group, s_item_id)) = start;
                let (e_kind, (e_belong_group, e_item_id)) = end;

                // you use endpoint's ancestors group id for item's belong group
                if (gid == s_belong_group
                    || resolver.get_ancestor_ids(s_belong_group).contains(&gid))
                    && (gid == e_belong_group
                        || resolver.get_ancestor_ids(e_belong_group).contains(&gid))
                {
                    Some(EdgeItem::new(
                        gid,
                        item_id,
                        Endpoint::new(s_kind, s_belong_group, s_item_id),
                        Endpoint::new(e_kind, e_belong_group, e_item_id),
                        label,
                        style.unwrap_or_default(),
                    ))
                } else {
                    errors.push(
                        EdgeItemError::InappropriateGroup(item_id, name.clone(), belong_group)
                            .into(),
                    );
                    None
                }
            }
            (Some(_), None, None) => {
                errors.extend(vec![
                    EdgeItemError::FailResolveStartEndpoint(item_id, name.clone(), start).into(),
                    EdgeItemError::FailResolveEndEndpoint(item_id, name.clone(), end).into(),
                ]);
                None
            }
            (Some(_), None, Some(_)) => {
                errors.push(
                    EdgeItemError::FailResolveStartEndpoint(item_id, name.clone(), start).into(),
                );
                None
            }
            (Some(_), Some(_), None) => {
                errors
                    .push(EdgeItemError::FailResolveEndEndpoint(item_id, name.clone(), end).into());
                None
            }
        };

        // option
        if let Some(n) = &name {
            if resolver.is_usable_graph_item_name(EdgeItem::kind(), n) {
                errors.push(
                    EdgeItemError::from_with_id(
                        item_id,
                        Some(n.clone()),
                        NameIdError::AlreadyExist(EdgeItem::kind(), n.clone()),
                    )
                    .into(),
                );
            }
        }
        (item, EdgeItemOption { name })
    }
}

impl<Name: NameType> EdgeItemBuilder<Name> {
    /// initializer for Edge item's builder
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            belong_group: None,
            name: None,
            label: None,
            style: None,
            start: None,
            end: None,
        }
    }

    /// setter for start endpoint
    pub fn set_start_endpoint<S: Into<Name>>(&mut self, kind: GraphItemKind, name: S) -> &mut Self {
        self.start = Some((kind, name.into()));
        self
    }

    /// setter for end endpoint
    pub fn set_end_endpoint<S: Into<Name>>(&mut self, kind: GraphItemKind, name: S) -> &mut Self {
        self.end = Some((kind, name.into()));
        self
    }
}
