//! Agent-oriented documentation embedded in the binary, so it always matches the CLI version.

pub struct Topic {
    pub name: &'static str,
    pub summary: &'static str,
    pub body: &'static str,
}

pub const TOPICS: &[Topic] = &[
    Topic {
        name: "overview",
        summary: "How the CLI behaves, output contract, exit codes, command map",
        body: include_str!("../guide/overview.md"),
    },
    Topic {
        name: "build-a-site",
        summary: "End-to-end recipe: theme, content model, pages, menus, go live",
        body: include_str!("../guide/build-a-site.md"),
    },
    Topic {
        name: "themes",
        summary: "Theme directory layout, pull/push workflow, media",
        body: include_str!("../guide/themes.md"),
    },
    Topic {
        name: "liquid",
        summary: "Liquid templating in Raytha: layouts, variables, filters, patterns",
        body: include_str!("../guide/liquid.md"),
    },
    Topic {
        name: "widgets",
        summary: "Widget templates, field definitions, and placing widgets on pages",
        body: include_str!("../guide/widgets.md"),
    },
    Topic {
        name: "content-types",
        summary: "Modelling content: content types, fields, views, content items",
        body: include_str!("../guide/content-types.md"),
    },
    Topic {
        name: "site-pages",
        summary: "Site pages, sections, widgets, draft vs published, home page",
        body: include_str!("../guide/site-pages.md"),
    },
    Topic {
        name: "media",
        summary: "Uploading and referencing images and files",
        body: include_str!("../guide/media.md"),
    },
    Topic {
        name: "functions",
        summary: "Raytha Functions: JavaScript for HTTP requests, Liquid and content events",
        body: include_str!("../guide/functions.md"),
    },
    Topic {
        name: "errors",
        summary: "Error codes, what they mean, and how to recover",
        body: include_str!("../guide/errors.md"),
    },
];

pub fn find(name: &str) -> Option<&'static Topic> {
    TOPICS.iter().find(|t| t.name == name)
}

/// Topic index shown when no topic is given.
pub fn index() -> String {
    let mut s = String::from(
        "# raytha guide\n\nRun `raytha guide <topic>` for any of these. Start with `overview`, \
         then `build-a-site`.\n\n",
    );
    for t in TOPICS {
        s.push_str(&format!("- {}: {}\n", t.name, t.summary));
    }
    s
}
