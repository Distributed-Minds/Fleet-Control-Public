//! Deterministic, offline, static preview renderer for typed catalog records.
//! Input must pass the current typed admission boundary before rendering.
//! This is NOT rights clearance, complete Draft 2020-12 schema validation,
//! hosted play, or proof that upstream links/redirects remain safe.
use free_energy_catalog::{is_public_https_url, PermissionStatus, PlayStatus, Project};

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

// Recheck the exact same conservative URL policy at the final HTML link
// sink. Typed Project values are public and can be modified after admission;
// a caller must not turn a formerly safe record into an executable or
// internal-host link simply by bypassing the manifest reader.
fn push_admitted_link(out: &mut String, url: &str, label: &str) {
    if is_public_https_url(url) {
        push_link(out, url, label);
    } else {
        out.push_str("<span>");
        out.push_str(label);
        out.push_str(" (link withheld: unsafe or unverified URL)</span>");
    }
}

fn play_label(status: &PlayStatus) -> &'static str {
    match status {
        PlayStatus::EngineOnly => {
            "Engine only — install a separately rights-checked game/content pack"
        }
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
        push_admitted_link(
            &mut out,
            &project.upstream.canonical_source_url,
            "Source upstream (external)",
        );
        out.push_str(" | ");
        push_admitted_link(
            &mut out,
            &project.upstream.contribution_url,
            "Contribute upstream (external)",
        );
        out.push_str("</p>\n");
        if let Some(download) = &project.play.upstream_download_url {
            out.push_str("<p>");
            push_admitted_link(
                &mut out,
                download,
                "Open upstream download (unverified by FREE ENERGY)",
            );
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
            out.push_str("</strong> <small>Claim status: ");
            out.push_str(&escape(&claim.status));
            out.push_str(" — ");
            out.push_str(match claim.status.as_str() {
                "OBSERVED_AT" => "Observed upstream claim, not clearance",
                "SUPERSEDED" => "Superseded, historical only",
                "RETRACTED" => "Retracted, not current",
                "UNKNOWN" => "Unknown, not verified",
                _ => "Unrecognized, do not rely on",
            });
            out.push_str("</small>: ");
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
            push_admitted_link(&mut out, &item.url, &escape(&item.evidence_id));
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
        out.push_str(
            ". Contributor-supplied reviewer metadata is not authenticated rights approval.</p>\n",
        );
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
    fn typed_records_cannot_bypass_url_admission_at_html_link_sink() {
        let mut record = validate_manifest(LUANTI).expect("valid pilot");
        // A public, mutable typed record may be changed after JSON admission.
        // Keep the final href sink fail-closed even for those callers.
        record.upstream.canonical_source_url = "javascript:alert(1)".to_owned();
        record.upstream.contribution_url = "https://127.0.0.1/private".to_owned();
        record.play.upstream_download_url = Some("https://valid.org/%2fprivate".to_owned());
        record.evidence[0].url = "https://localhost/evidence".to_owned();

        let html = render_catalog(&[record]);
        assert_eq!(
            html.matches("link withheld: unsafe or unverified URL")
                .count(),
            4,
            "every mutated outbound URL must become non-clickable"
        );
        for forbidden in [
            "href=\"javascript:",
            "href=\"https://127.0.0.1",
            "href=\"https://valid.org/%2f",
            "href=\"https://localhost",
        ] {
            assert!(
                !html.contains(forbidden),
                "unsafe rendered link: {forbidden}"
            );
        }
        assert!(html.contains("Source upstream (external) (link withheld:"));
        assert!(html.contains("Open upstream download (unverified by FREE ENERGY) (link withheld:"));
    }

    #[test]
    fn admitted_pilot_urls_remain_clickable_in_the_static_renderer() {
        let project = validate_manifest(LUANTI).expect("valid pilot");
        let html = render_catalog(&[project]);
        assert!(html.contains("<a href=\"https://"));
        assert!(!html.contains("link withheld:"));
    }

    #[test]
    fn hostile_display_strings_are_escaped() {
        let mut record = validate_manifest(LUANTI).expect("valid pilot");
        record.display_name = "<script>&\"'".to_string();
        let html = render_catalog(&[record]);
        assert!(html.contains("&lt;script&gt;&amp;&quot;&#39;"));
        assert!(!html.contains("<script>"));
    }

    #[test]
    fn authored_escaping_fixture_exercises_rendered_text_and_attribute_encoder() {
        // Fixture expectations are checked against real HTML output, not copied
        // back as a fixture verdict. This is lexical escaping conformance; it
        // does not replace browser DOM, URL admission or rights verification.
        let suite: serde_json::Value =
            serde_json::from_str(include_str!("../fixtures/renderer-escaping-v0.json"))
                .expect("valid renderer fixture JSON");
        let cases = suite["cases"].as_array().expect("renderer fixture cases");
        assert_eq!(cases.len(), 14, "all authored escaping cases must run");

        for case in cases {
            let id = case["id"].as_str().expect("fixture case ID");
            let input = case["input"].as_str().expect("untrusted fixture input");
            let expected = case["reference_escaped"]
                .as_str()
                .expect("reference escaped string");
            assert_eq!(
                case["expected_decoded_value"].as_str(),
                Some(input),
                "invalid decoded fixture contract: {id}"
            );
            assert_eq!(escape(input), expected, "escape primitive: {id}");

            match case["context"].as_str().expect("escaping context") {
                "text" => {
                    let mut project = validate_manifest(LUANTI).expect("valid pilot");
                    let surrounded = match case["source_field"].as_str().expect("text source field")
                    {
                        "display_name" => {
                            project.display_name = input.to_owned();
                            format!("<h2>{expected}</h2>")
                        }
                        "rights_claims[].statement" => {
                            project.rights_claims[0].statement = input.to_owned();
                            format!(": {expected} <small>")
                        }
                        "play.content_requirements[]" => {
                            project.play.content_requirements = vec![input.to_owned()];
                            format!("<li>{expected}</li>")
                        }
                        other => panic!("unhandled text sink in {id}: {other}"),
                    };
                    let html = render_catalog(&[project]);
                    assert!(
                        html.contains(&surrounded),
                        "input not encoded in its actual rendered text sink: {id}"
                    );
                    if input.contains('<') {
                        assert!(
                            !html.contains(input),
                            "untrusted markup escaped incorrectly in rendered page: {id}"
                        );
                    }
                }
                "double_quoted_attribute" => {
                    // The v0 page has no manifest-provided aria-label. Its only
                    // dynamic attribute sink is an href populated by push_link;
                    // URL admission takes place before this rendering boundary.
                    let mut html = String::new();
                    push_link(&mut html, input, "fixture link");
                    assert_eq!(
                        html,
                        format!(
                            "<a href=\"{expected}\" rel=\"noopener noreferrer\">fixture link</a>"
                        ),
                        "attribute breakout or incorrect encoding: {id}"
                    );
                    assert_eq!(html.matches(" href=").count(), 1, "{id}");
                    assert_eq!(html.matches(" rel=").count(), 1, "{id}");
                    if input.contains('"') {
                        assert_ne!(
                            html,
                            format!(
                                "<a href=\"{input}\" rel=\"noopener noreferrer\">fixture link</a>"
                            ),
                            "naive unescaped attribute output: {id}"
                        );
                    }
                }
                other => panic!("unhandled renderer context in {id}: {other}"),
            }
        }
    }
}

#[cfg(test)]
mod claim_state_regression {
    use super::render_catalog;
    use free_energy_catalog::validate_manifest;

    const LUANTI: &str = include_str!("../projects/luanti.json");

    #[test]
    fn retracted_and_unknown_claims_are_visibly_not_current() {
        let mut record = validate_manifest(LUANTI).expect("valid pilot");
        record.rights_claims[0].status = "RETRACTED".to_owned();
        record.rights_claims[0].statement = "Hypothetical positive claim".to_owned();
        record.rights_claims[1].status = "UNKNOWN".to_owned();
        let html = render_catalog(&[record]);
        assert!(
            html.contains("Claim status: RETRACTED — Retracted, not current"),
            "retracted claims must never look like current clearance"
        );
        assert!(
            html.contains("Claim status: UNKNOWN — Unknown, not verified"),
            "unverified rights cannot look like positive permission"
        );
        assert!(html.contains("Rights review required"));
        assert!(!html.contains("Claim status: RETRACTED</small>:"));
    }

    #[test]
    fn status_metadata_is_escaped_when_renderer_gets_untrusted_values() {
        let mut record = validate_manifest(LUANTI).expect("valid pilot");
        record.rights_claims[0].status = "<img src=x onerror=alert(1)>".to_owned();
        let html = render_catalog(&[record]);
        assert!(html.contains("&lt;img src=x onerror=alert(1)&gt;"));
        assert!(!html.contains("<img src=x"));
        assert!(html.contains("Unrecognized, do not rely on"));
    }
}
