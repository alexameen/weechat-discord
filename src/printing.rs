use crate::discord::formatting;
use serenity::model::prelude::*;
use weechat::{Buffer, Weechat};

// TODO: Rework args
// TODO: Color things
pub fn print_msg(weechat: &Weechat, buffer: &Buffer, msg: &Message, notify: bool) {
    let ctx = match crate::discord::get_ctx() {
        Some(ctx) => ctx,
        _ => return,
    };
    let is_private = if let Some(channel) = msg.channel(ctx) {
        if let Channel::Private(_) = channel {
            true
        } else {
            false
        }
    } else {
        false
    };

    let self_mentioned = msg.mentions_user_id(ctx.cache.read().user.id);

    let tags = {
        let mut tags = Vec::new();
        if notify {
            if self_mentioned {
                tags.push("notify_highlight");
            } else if is_private {
                tags.push("notify_private");
            } else {
                tags.push("notify_message");
            };
        } else {
            tags.push("notify_none");
        }

        tags.join(",")
    };

    let mut msg_content = msg.content_safe(ctx);

    // TODO: Report content_safe() bug
    // TODO: Use nicknames instead of user names
    for u in &msg.mentions {
        let mut at_distinct = String::with_capacity(38);
        at_distinct.push('@');
        at_distinct.push_str(&u.name);
        at_distinct.push('#');
        let mention = u.mention().replace("<@", "<@!");
        use std::fmt::Write;
        let _ = write!(at_distinct, "{:04}", u.discriminator);
        msg_content = msg_content.replace(&mention, &at_distinct);
    }

    for attachement in &msg.attachments {
        if !msg_content.is_empty() {
            msg_content.push('\n');
        }
        msg_content.push_str(&attachement.proxy_url);
    }

    for embed in &msg.embeds {
        if !msg_content.is_empty() {
            msg_content.push('\n');
        }
        if let Some(ref author) = embed.author {
            msg_content.push_str(&author.name);
            msg_content.push('\n');
        }
        if let Some(ref title) = embed.title {
            msg_content.push_str(title);
            msg_content.push('\n');
        }
        if let Some(ref description) = embed.description {
            msg_content.push_str(description);
            msg_content.push('\n');
        }
        for field in &embed.fields {
            msg_content.push_str(&field.name);
            msg_content.push_str(&field.value);
            msg_content.push('\n');
        }
        if let Some(ref footer) = embed.footer {
            msg_content.push_str(&footer.text);
            msg_content.push('\n');
        }
    }

    let maybe_guild = buffer.get_localvar("guildid");
    let display_name = maybe_guild.and_then(|id| {
        id.parse::<u64>().ok().map(GuildId).and_then(|id| {
            ctx.cache
                .read()
                .member(id, msg.author.id)
                .map(|member| member.display_name().to_string())
        })
    });

    let author = display_name.unwrap_or_else(|| msg.author.name.to_owned());

    buffer.print_tags_dated(
        msg.timestamp.timestamp(),
        &tags,
        &format!(
            "{}\t{}",
            author,
            formatting::discord_to_weechat(weechat, &msg_content)
        ),
    );
}
