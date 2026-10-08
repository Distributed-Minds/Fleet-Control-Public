//! Deterministic, offline, static preview renderer for typed catalog records.
//! Input must pass the current typed admission boundary before rendering.
//! This is NOT rights clearance, complete Draft 2020-12 schema validation,
//! hosted play, or proof that upstream links/redirects remain safe.
use free_energy_catalog::{PermissionStatus, PlayStatus, Project};

fn escape(value: &str) -> String {
    let mut out = String::new();
    for c in value.chars() {
        match c {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            _ => out.push(c),
        }
    }
    out
}

fn push_link(out: &mut String, url: &str, label: &str) {
    out.push_str("<a href=\"");
    out.push_str(&escape(url));
    out.push_str("\" rel=\"noopener noreferrer\">");
    out.push_str(label);
    out.push_str("</a>");
}

fn play_label(status: &PlayStatus) -> &'static str {
    match status {
        PlayStatus::EngineOnly => "Engine only — install a separately rights-checked game/content pack",
        PlayStatus::UpstreamLinkOnly => "Upstream download link only — not verified by FREE ENERGY",
        PlayStatus::FreeEnergyVerified => "FREE ENERGY verified",
        PlayStatus::Unavailable => "Unavailable according to this draft record",
        PlayStatus::Unknown => "Play status not verified",
    }
}

pub fn render_catalog(records: &[Project]) -> String {
    let mut projects: Vec<_> = records.iter().collect();
    projects.sort_by(|a, b| a.id.cmp(&b.id));
    let mut out = String::from(concat!(
        "<!doctype html>\n<html lang=\"en\">\n<head>\n",
        "<meta charset=\"utf-8\">\n",
        "<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n",
        "<title>FREE ENERGY — Project catalog draft</title>\n",
        "</head>\n<body>\n<main>\n",
        "<h1>FREE ENERGY — project catalog (draft)</h1>\n",
        "<p>Upstream links only. No game is verified or hosted by FREE ENERGY. ",
        "Rights review is required before reuse. These records are research drafts, ",
        "not permission to redistribute assets.</p>\n",
    ));
    for project in projects {
        out.push_str("<article>\n<h2>");
        out.push_str(&escape(&project.display_name));
        out.push_str("</h2>\n<p><strong>Kind:</strong> ");
        out.push_str(&escape(&project.kind));
        out.push_str(" — <strong>ID:</strong> <code>");
        out.push_str(&escape(&project.id));
        out.push_str("</code></p>\n<p><strong>Play:</strong> ");
        out.push_str(play_label(&project.play.status));
        out.push_str("</p>\n<p>");
        push_link(&mut out, &project.upstream.canonical_source_url, "Source upstream (external)");
        out.push_str(" | ");
        push_link(&mut out, &project.upstream.contribution_url, "Contribute upstream (external)");
        out.push_str("</p>\n");
        if let Some(download) = &project.play.upstream_download_url {
            out.push_str("<p>");
            push_link(&mut out, download, "Open upstream download (unverified by FREE ENERGY)");
            out.push_str("</p>\n");
        }
        if !project.play.content_requirements.is_empty() {
            out.push_str("<h3>Requirements and limitations</h3>\n<ul>\n");
            for requirement in &project.play.content_requirements {
                out.push_str("<li>");
                out.push_str(&escape(requirement));
                out.push_str("</li>\n");
            }
            out.push_str("</ul>\n");
        }

        out.push_str("<h3>Rights claims — not reuse clearance</h3>\n<ul>\n");
        let mut claims: Vec<_> = project.rights_claims.iter().collect();
        claims.sort_by(|a, b| a.claim_id.cmp(&b.claim_id));
        for claim in claims {
            out.push_str("<li><strong>");
            out.push_str(&escape(&claim.component));
            out.push_str(" / ");
            out.push_str(&escape(&claim.scope));
            out.push_str("</strong>: ");
            out.push_str(&escape(&claim.statement));
            out.push_str(" <small>Evidence IDs: ");
            let mut ids = claim.evidence_ids.clone();
            ids.sort();
            out.push_str(&escape(&ids.join(", ")));
            out.push_str("</small></li>\n");
        }
        out.push_str("</ul>\n<h3>Permission decisions</h3>\n<ul>\n");
        let mut permissions: Vec<_> = project.permission_decisions.iter().collect();
        permissions.sort_by(|a, b| {
            (&a.component, &a.scope, &a.use_kind).cmp(&(&b.component, &b.scope, &b.use_kind))
        });
        for permission in permissions {
            out.push_str("<li>");
            out.push_str(&escape(&permission.component));
            out.push_str(" / ");
            out.push_str(&escape(&permission.scope));
            out.push_str(" / ");
            out.push_str(&escape(&permission.use_kind));
            out.push_str(": ");
            out.push_str(match permission.decision {
                PermissionStatus::ReviewRequired => "Rights review required",
                PermissionStatus::NotAuthorized => "Not authorized",
            });
            out.push_str("</li>\n");
        }

        out.push_str("</ul>\n<h3>Evidence snapshots</h3>\n<ul>\n");
        let mut evidence: Vec<_> = project.evidence.iter().collect();
        evidence.sort_by(|a, b| a.evidence_id.cmp(&b.evidence_id));
        for item in evidence {
            out.push_str("<li>");
            push_link(&mut out, &item.url, &escape(&item.evidence_id));
            out.push_str(" — observed ");
            out.push_str(&escape(&item.observed_at));
            out.push_str(" — ");
            out.push_str(&escape(&item.currentness));
            out.push_str("</li>\n");
        }
        out.push_str("</ul>\n<p><strong>Record review:</strong> ");
        out.push_str(&escape(&project.review.reviewed_at));
        out.push_str(" — ");
        out.push_str(&escape(&project.review.record_status));
        out.push_str(". Contributor-supplied reviewer metadata is not authenticated rights approval.</p>\n");
        out.push_str("</article>\n");
    }
    out.push_str("</main>\n</body>\n</html>\n");
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use free_energy_catalog::validate_manifest;

    const LUANTI: &str = include_str!("../projects/luanti.json");
    const OPENRA: &str = include_str!("../projects/openra.json");
    const VELOREN: &str = include_str!("../projects/veloren.json");

    #[test]
    fn committed_page_is_exact_deterministic_output() {
        let records: Vec<_> = [VELOREN, LUANTI, OPENRA]
            .iter()
            .map(|json| validate_manifest(json).expect("valid pilot manifest"))
            .collect();
        assert_eq!(render_catalog(&records), include_str!("../site/index.html"));
    }

    #[test]
    fn hostile_display_strings_are_escaped() {
        let mut record = validate_manifest(LUANTI).expect("valid pilot");
        record.display_name = "<script>&\"'".to_string();
        let html = render_catalog(&[record]);
        assert!(html.contains("&lt;script&gt;&amp;&quot;&#39;"));
        assert!(!html.contains("<script>"));
    }
}
