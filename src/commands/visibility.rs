use crate::args::ParsedArgs;
use crate::client::{HttpTransport, YtClient};
use crate::config;
use crate::error::YtdError;
use crate::types::{LimitedVisibilityInput, ResolvedVisibilityGroup, UserGroupInput};

pub fn build_create_visibility_input<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
) -> Result<Option<LimitedVisibilityInput>, YtdError> {
    build_visibility_input(client, args, false, true)
}

pub fn build_explicit_update_visibility_input<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
) -> Result<Option<LimitedVisibilityInput>, YtdError> {
    build_visibility_input(client, args, true, false)
}

pub fn build_comment_update_visibility_input<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
) -> Result<Option<LimitedVisibilityInput>, YtdError> {
    build_explicit_update_visibility_input(client, args)
}

fn build_visibility_input<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    clear_on_no_visibility_group: bool,
    include_defaults: bool,
) -> Result<Option<LimitedVisibilityInput>, YtdError> {
    let group = if include_defaults {
        config::resolve_visibility_group(
            args.flags.get("visibility-group").map(|s| s.as_str()),
            args.flags.contains_key("no-visibility-group"),
        )?
    } else {
        resolve_explicit_visibility_group(args)?
    };

    match group {
        ResolvedVisibilityGroup::Group(group) => Ok(Some(LimitedVisibilityInput {
            visibility_type: "LimitedVisibility",
            permitted_groups: vec![UserGroupInput {
                id: resolve_group_id(client, &group)?,
            }],
        })),
        ResolvedVisibilityGroup::Clear if clear_on_no_visibility_group => {
            Ok(Some(LimitedVisibilityInput {
                visibility_type: "LimitedVisibility",
                permitted_groups: vec![],
            }))
        }
        ResolvedVisibilityGroup::Clear | ResolvedVisibilityGroup::None => Ok(None),
    }
}

fn resolve_explicit_visibility_group(
    args: &ParsedArgs,
) -> Result<ResolvedVisibilityGroup, YtdError> {
    let group = args.flags.get("visibility-group").map(|s| s.as_str());
    let clear = args.flags.contains_key("no-visibility-group");

    if group.is_some() && clear {
        return Err(YtdError::Input(
            "--visibility-group cannot be combined with --no-visibility-group".into(),
        ));
    }

    if clear {
        return Ok(ResolvedVisibilityGroup::Clear);
    }

    match group {
        Some(group) if group.trim().is_empty() => {
            Err(YtdError::Input("visibility-group cannot be empty".into()))
        }
        Some(group) => Ok(ResolvedVisibilityGroup::Group(group.to_string())),
        None => Ok(ResolvedVisibilityGroup::None),
    }
}

fn resolve_group_id<T: HttpTransport>(
    client: &YtClient<T>,
    group_name: &str,
) -> Result<String, YtdError> {
    let groups = client.list_groups()?;

    if let Some(group) = groups.iter().find(|group| group.name == group_name) {
        return Ok(group.id.clone());
    }

    if let Some(group) = groups
        .iter()
        .find(|group| group.name.eq_ignore_ascii_case(group_name))
    {
        return Ok(group.id.clone());
    }

    Err(YtdError::Input(format!(
        "Visibility group not found: {group_name}"
    )))
}
