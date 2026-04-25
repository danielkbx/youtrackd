use crate::args::ParsedArgs;
use crate::client::{HttpTransport, YtClient};
use crate::error::YtdError;
use crate::format::{self, OutputOptions};
use crate::types::User;

pub fn run<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    match args.action.as_deref() {
        Some("list") => cmd_list(client, opts),
        Some("get") => cmd_get(client, args, opts),
        _ => Err(YtdError::Input("Usage: ytd user <list|get>".into())),
    }
}

fn cmd_list<T: HttpTransport>(client: &YtClient<T>, opts: &OutputOptions) -> Result<(), YtdError> {
    let users = client.list_users()?;
    print_users(&users, opts);
    Ok(())
}

fn cmd_get<T: HttpTransport>(
    client: &YtClient<T>,
    args: &ParsedArgs,
    opts: &OutputOptions,
) -> Result<(), YtdError> {
    let id = args
        .positional
        .first()
        .ok_or_else(|| YtdError::Input("Usage: ytd user get <user-id-or-login>".into()))?;
    let user = client.resolve_user(id)?;
    print_user(&user, opts);
    Ok(())
}

fn print_users(users: &[User], opts: &OutputOptions) {
    if matches!(opts.format, format::Format::Text) {
        print!("{}", render_users_text(users, opts));
    } else {
        format::print_items(users, opts);
    }
}

fn print_user(user: &User, opts: &OutputOptions) {
    if matches!(opts.format, format::Format::Text) {
        print!("{}", render_user_text(user, opts));
    } else {
        format::print_single(user, opts);
    }
}

fn render_users_text(users: &[User], opts: &OutputOptions) -> String {
    let mut out = String::new();
    for (idx, user) in users.iter().enumerate() {
        if idx > 0 {
            out.push('\n');
        }
        out.push_str(&render_user_text(user, opts));
    }
    out
}

fn render_user_text(user: &User, opts: &OutputOptions) -> String {
    let mut out = String::new();
    out.push_str(&user_display(user));
    out.push('\n');
    if !opts.no_meta {
        out.push_str("  id: ");
        out.push_str(&user.id);
        out.push('\n');
    }
    out.push_str("  login: ");
    out.push_str(&user.login);
    out.push('\n');
    if !opts.no_meta {
        if let Some(email) = user
            .email
            .as_deref()
            .filter(|email| !email.trim().is_empty())
        {
            out.push_str("  email: ");
            out.push_str(email);
            out.push('\n');
        }
    }
    if user.banned == Some(true) {
        out.push_str("  banned: yes\n");
    }
    if user.guest == Some(true) {
        out.push_str("  guest: yes\n");
    }
    out
}

fn user_display(user: &User) -> String {
    match user.full_name.as_deref() {
        Some(full_name) if !full_name.trim().is_empty() => {
            format!("{} ({})", full_name.trim(), user.login)
        }
        _ => user.login.clone(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn user() -> User {
        User {
            id: "1-51".into(),
            login: "alice".into(),
            full_name: Some("Alice Example".into()),
            email: Some("alice@example.com".into()),
            banned: Some(false),
            guest: Some(false),
        }
    }

    #[test]
    fn text_output_includes_user_fields() {
        let rendered = render_user_text(
            &user(),
            &OutputOptions {
                format: format::Format::Text,
                no_meta: false,
            },
        );

        assert!(rendered.contains("Alice Example (alice)"));
        assert!(rendered.contains("id: 1-51"));
        assert!(rendered.contains("email: alice@example.com"));
    }

    #[test]
    fn no_meta_hides_id_and_email() {
        let rendered = render_user_text(
            &user(),
            &OutputOptions {
                format: format::Format::Text,
                no_meta: true,
            },
        );

        assert!(!rendered.contains("id:"));
        assert!(!rendered.contains("email:"));
        assert!(rendered.contains("login: alice"));
    }
}
