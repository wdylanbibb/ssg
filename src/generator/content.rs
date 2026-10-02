use std::{fs, path::Path};

use super::{
    error::ContentError, front_matter::parse_document, plan::PlannedPage, site::SiteConfig,
};
use minijinja::{AutoEscape, Environment, path_loader};
use pulldown_cmark::{CowStr, Event, LinkType, Options, Parser, Tag, html};

pub(super) fn build_content(
    pages: &[PlannedPage],
    templates: &Path,
    output: &Path,
    site: &SiteConfig,
) -> Result<usize, ContentError> {
    let templates = create_template_environment(templates);
    let mut pages_written = 0;

    for page in pages {
        let rendered = render_page(&page.source, &templates, site)?;
        write_page(output, &page.destination, &rendered)?;

        pages_written += 1;
    }

    Ok(pages_written)
}

fn create_template_environment(templates: &Path) -> Environment<'static> {
    let mut environment = Environment::new();

    environment.set_loader(path_loader(templates));
    environment.set_auto_escape_callback(|name| {
        if name.ends_with(".html") {
            AutoEscape::Html
        } else {
            AutoEscape::None
        }
    });

    environment
}

fn render_page(
    source_path: &Path,
    templates: &Environment<'_>,
    site: &SiteConfig,
) -> Result<String, ContentError> {
    let document = fs::read_to_string(source_path).map_err(|source| ContentError::Read {
        path: source_path.to_path_buf(),
        source,
    })?;

    let (front_matter, markdown) =
        parse_document(&document).map_err(|source| ContentError::FrontMatter {
            path: source_path.to_path_buf(),
            source,
        })?;

    let template_name = front_matter.template.clone();
    let template =
        templates
            .get_template(&template_name)
            .map_err(|source| ContentError::LoadTemplate {
                page: source_path.to_path_buf(),
                template: template_name.clone(),
                source,
            })?;

    template
        .render(minijinja::context! {
            site => site,
            page => minijinja::context! {
                title => front_matter.title,
                description => front_matter.description,
                content => minijinja::Value::from_safe_string(render_markdown(markdown)),
            },
        })
        .map_err(|source| ContentError::RenderTemplate {
            page: source_path.to_path_buf(),
            template: template_name,
            source,
        })
}

fn render_markdown(markdown: &str) -> String {
    let options = Options::ENABLE_WIKILINKS | Options::ENABLE_HEADING_ATTRIBUTES;
    let parser = Parser::new_ext(markdown, options).map(|event| match event {
        Event::Start(Tag::Link {
            link_type: LinkType::WikiLink { has_pothole },
            dest_url,
            title,
            id,
        }) => {
            let dest_url = normalize_wikilink(&dest_url);

            Event::Start(Tag::Link {
                link_type: LinkType::WikiLink { has_pothole },
                dest_url,
                title,
                id,
            })
        }

        event => event,
    });

    let mut output = String::new();
    html::push_html(&mut output, parser);
    output
}

fn normalize_wikilink(target: &str) -> CowStr<'static> {
    let (page, heading) = target.split_once('#').unwrap_or((target, ""));
    let page = page.strip_suffix(".md").unwrap_or(page);

    let mut url = if page.is_empty() {
        String::new()
    } else {
        format!("/{}/", page.trim_matches('/'))
    };

    if !heading.is_empty() {
        url.push('#');
        url.push_str(&slugify_heading(heading));
    }

    url.into()
}

fn slugify_heading(heading: &str) -> String {
    heading
        .trim()
        .to_lowercase()
        .replace(' ', "-")
}

fn write_page(output: &Path, relative_path: &Path, html: &str) -> Result<(), ContentError> {
    let destination = output.join(relative_path);

    if let Some(parent) = destination.parent() {
        fs::create_dir_all(parent).map_err(|source| ContentError::CreatePageDirectory {
            path: parent.to_path_buf(),
            source,
        })?;
    }

    fs::write(&destination, html).map_err(|source| ContentError::WritePage {
        path: destination,
        source,
    })
}
