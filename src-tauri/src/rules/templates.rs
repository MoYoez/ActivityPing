use crate::platform::MediaInfo;

pub(super) struct DiscordTemplateValues<'a> {
    activity: &'a str,
    context: &'a str,
    app: &'a str,
    title: &'a str,
    rule: &'a str,
    media: String,
    song: &'a str,
    artist: &'a str,
    album: &'a str,
    source: &'a str,
}

impl<'a> DiscordTemplateValues<'a> {
    pub(super) fn new(
        activity: &'a str,
        context: Option<&'a str>,
        app: &'a str,
        title: Option<&'a str>,
        rule: Option<&'a str>,
        media: &'a MediaInfo,
        source: Option<&'a str>,
        media_visible: bool,
    ) -> Self {
        Self {
            activity,
            context: context.unwrap_or(""),
            app,
            title: title.unwrap_or(""),
            rule: rule.unwrap_or(""),
            media: if media_visible {
                media.summary()
            } else {
                String::new()
            },
            song: if media_visible {
                media.title.as_str()
            } else {
                ""
            },
            artist: if media_visible {
                media.artist.as_str()
            } else {
                ""
            },
            album: if media_visible {
                media.album.as_str()
            } else {
                ""
            },
            source: source.unwrap_or(""),
        }
    }

    fn value(&self, key: &str) -> Option<&str> {
        match key {
            "activity" => Some(self.activity),
            "context" => Some(self.context),
            "app" | "process" => Some(self.app),
            "title" => Some(self.title),
            "rule" => Some(self.rule),
            "media" => Some(self.media.as_str()),
            "song" => Some(self.song),
            "artist" => Some(self.artist),
            "album" => Some(self.album),
            "source" => Some(self.source),
            _ => None,
        }
    }
}

pub(super) fn render_discord_template(
    template: &str,
    values: &DiscordTemplateValues<'_>,
) -> Option<String> {
    let template = template.trim();
    if template.is_empty() {
        return None;
    }

    let mut output = String::new();
    let mut chars = template.chars().peekable();
    while let Some(ch) = chars.next() {
        if ch != '{' {
            output.push(ch);
            continue;
        }

        let mut key = String::new();
        let mut closed = false;
        while let Some(next) = chars.next() {
            if next == '}' {
                closed = true;
                break;
            }
            key.push(next);
        }

        if closed {
            if let Some(value) = values.value(key.trim()) {
                output.push_str(value);
            } else {
                output.push('{');
                output.push_str(&key);
                output.push('}');
            }
        } else {
            output.push('{');
            output.push_str(&key);
        }
    }

    Some(clean_rendered_text(&output))
}

fn clean_rendered_text(value: &str) -> String {
    let collapsed = value.split_whitespace().collect::<Vec<_>>().join(" ");
    collapsed
        .trim_matches(|ch: char| {
            ch.is_whitespace() || matches!(ch, '|' | '-' | '·' | '/' | '\\' | ',' | ':' | ';')
        })
        .trim()
        .to_string()
}
