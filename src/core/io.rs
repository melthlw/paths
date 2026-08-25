use crate::core::document::Document;
use crate::core::geometry::Rect;
use crate::core::page::Page;
use crate::core::svg_export::{DocumentProjectData, export_document_to_svg};
use crate::core::svg_import::parse_svg;
use std::path::Path;

/// Saves a document to a file in SVG format with embedded lossless Paths metadata
pub fn save_document_to_file(doc: &Document, path: &Path) -> Result<(), String> {
    let svg_content = export_document_to_svg(doc);
    std::fs::write(path, svg_content)
        .map_err(|e| crate::i18n!("Error saving file {}: {}", path.display(), e))?;
    Ok(())
}

/// Loads a document from an SVG string (either a Paths project SVG or standard external SVG)
pub fn load_document_from_svg(content: &str) -> Result<Document, String> {
    // 1. Check for Paths lossless project metadata in <metadata>
    let meta_tag_start = content
        .find("<paths:project")
        .or_else(|| content.find("<gnomepaths:project"));
    if let Some(meta_start) = meta_tag_start {
        let close_tag = if content[meta_start..].starts_with("<paths:project") {
            "</paths:project>"
        } else {
            "</gnomepaths:project>"
        };
        if let Some(tag_end) = content[meta_start..].find('>') {
            let body_start = meta_start + tag_end + 1;
            if let Some(body_end) = content[body_start..].find(close_tag) {
                let mut json_str = &content[body_start..body_start + body_end];
                // Strip CDATA wrapper if present
                if let Some(cdata_start) = json_str.find("<![CDATA[") {
                    let after_cdata = &json_str[cdata_start + 9..];
                    if let Some(cdata_end) = after_cdata.find("]]>") {
                        json_str = &after_cdata[..cdata_end];
                    }
                }
                let json_trimmed = json_str.trim();
                if let Ok(project_data) = serde_json::from_str::<DocumentProjectData>(json_trimmed)
                {
                    return Ok(project_data.to_document());
                }
            }
        }
    }

    // 2. Fallback: Parse standard vector SVG XML
    match parse_svg(content) {
        Ok(import_res) => {
            let mut doc = Document::default();
            doc.elements = import_res.elements;
            let page_rect = Rect::new(
                0.0,
                0.0,
                import_res.width.max(100.0),
                import_res.height.max(100.0),
            );
            let page = Page::new(crate::core::gettext("Page 1"), page_rect);
            let pid = page.id;
            doc.pages = vec![page];
            doc.active_page_id = Some(pid);
            Ok(doc)
        }
        Err(e) => Err(crate::i18n!("Failed to parse SVG: {}", e)),
    }
}

/// Loads a document from a file path
pub fn load_document_from_file(path: &Path) -> Result<Document, String> {
    let bytes = std::fs::read(path)
        .map_err(|e| crate::i18n!("Error reading file {}: {}", path.display(), e))?;

    if let Ok(text) = std::str::from_utf8(&bytes) {
        if text.contains("<svg") || text.contains("<?xml") {
            return load_document_from_svg(text);
        }
    }

    // If it's a raster image or other format, attempt to import into a clean document
    if let Some(image) = skia_safe::Image::from_encoded(skia_safe::Data::new_copy(&bytes)) {
        let iw = image.width() as f32;
        let ih = image.height() as f32;
        let filename = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("Image")
            .to_string();
        let rect = Rect::new(0.0, 0.0, iw, ih);
        let img_elem = crate::core::element::ImageElement::new(rect, bytes, Some(filename.clone()));
        let mut doc = Document::default();
        let page = Page::new(crate::core::gettext("Page 1"), rect);
        let pid = page.id;
        doc.pages = vec![page];
        doc.active_page_id = Some(pid);
        doc.elements = vec![crate::core::Element::Image(img_elem)];
        return Ok(doc);
    }

    Err(format!(
        "Formato de arquivo não suportado para carregamento de documento: {:?}",
        path
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::color::Color;
    use crate::core::element::{PathElement, PathNode, RectElement};
    use crate::core::geometry::Point;

    #[test]
    fn test_svg_roundtrip_lossless() {
        let mut doc = Document::default();
        let rect_elem = RectElement::new(
            Rect::new(50.0, 60.0, 200.0, 100.0),
            Some(Color::new(1.0, 0.0, 0.0, 0.8)),
            Some(Color::BLACK),
        );
        doc.add_element(crate::core::Element::Rect(rect_elem));

        let nodes = vec![
            PathNode::new(Point::new(0.0, 0.0)),
            PathNode::with_handles(
                Point::new(100.0, 0.0),
                Some(Point::new(50.0, -20.0)),
                Some(Point::new(80.0, 30.0)),
            ),
            PathNode::new(Point::new(100.0, 100.0)),
        ];
        let path_elem = PathElement::new(nodes, true, Some(Color::BLUE), None, 2.0);
        doc.add_element(crate::core::Element::Path(path_elem));

        let svg_str = export_document_to_svg(&doc);
        assert!(svg_str.contains("<svg"));
        assert!(svg_str.contains("<metadata"));
        assert!(svg_str.contains("<rect"));
        assert!(svg_str.contains("<path"));

        let loaded_doc = load_document_from_svg(&svg_str).expect("Failed to load SVG");
        assert_eq!(loaded_doc.elements.len(), 2);
        match &loaded_doc.elements[0] {
            crate::core::Element::Rect(r) => {
                assert_eq!(r.rect.x, 50.0);
                assert_eq!(r.rect.width, 200.0);
            }
            _ => panic!("Expected RectElement"),
        }
        match &loaded_doc.elements[1] {
            crate::core::Element::Path(p) => {
                assert_eq!(p.nodes.len(), 3);
                assert!(p.is_closed);
            }
            _ => panic!("Expected PathElement"),
        }
    }

    #[test]
    fn test_svg_export_and_import_text_brush_group() {
        let mut doc = Document::default();
        let text_elem = crate::core::element::TextElement::new(
            Point::new(100.0, 150.0),
            "Paths Vector".to_string(),
            24.0,
            Color::BLACK,
        );
        doc.add_element(crate::core::Element::Text(text_elem));

        let brush = crate::core::element::BrushStroke::new(
            vec![
                Point::new(10.0, 10.0),
                Point::new(20.0, 30.0),
                Point::new(40.0, 50.0),
            ],
            Color::BLUE,
            4.0,
        );
        doc.add_element(crate::core::Element::Brush(brush));

        let group = crate::core::element::GroupElement::new(vec![crate::core::Element::Rect(
            RectElement::new(Rect::new(0.0, 0.0, 50.0, 50.0), Some(Color::WHITE), None),
        )]);
        doc.add_element(crate::core::Element::Group(group));

        let svg_str = export_document_to_svg(&doc);
        assert!(svg_str.contains("<text"));
        assert!(svg_str.contains("Paths Vector"));
        assert!(svg_str.contains("<g"));

        let loaded = load_document_from_svg(&svg_str).expect("Should parse");
        assert_eq!(loaded.elements.len(), 3);
    }

    #[test]
    fn test_svg_export_multipage_and_guides() {
        let mut doc = Document::default();
        let p2_id = doc.add_next_page();
        doc.rename_page(p2_id, "Artboard 2".to_string());
        doc.add_guide(crate::core::ruler::Guide::new(
            crate::core::ruler::GuideOrientation::Horizontal,
            250.0,
        ));
        doc.add_guide(crate::core::ruler::Guide::new(
            crate::core::ruler::GuideOrientation::Vertical,
            400.0,
        ));

        let svg_str = export_document_to_svg(&doc);
        let loaded = load_document_from_svg(&svg_str).expect("Should load multipage doc");
        assert_eq!(loaded.pages.len(), 2);
        assert_eq!(loaded.pages[1].name, "Artboard 2");
        assert_eq!(loaded.guides.len(), 2);
        assert_eq!(loaded.guides[0].position, 250.0);
        assert_eq!(loaded.guides[1].position, 400.0);
    }

    #[test]
    fn test_standard_svg_fallback_loading() {
        let standard_svg = r##"<svg xmlns="http://www.w3.org/2000/svg" width="400" height="300">
            <rect x="20" y="30" width="100" height="80" fill="#ff0000" stroke="#000000" stroke-width="2"/>
            <circle cx="200" cy="150" r="50" fill="#0000ff"/>
        </svg>"##;

        let loaded =
            load_document_from_svg(standard_svg).expect("Should parse standard SVG fallback");
        assert_eq!(loaded.elements.len(), 2);
        assert_eq!(loaded.pages.len(), 1);
        assert_eq!(loaded.pages[0].rect.width, 400.0);
        assert_eq!(loaded.pages[0].rect.height, 300.0);
    }

    #[test]
    fn test_export_document_svg_format() {
        let doc = Document::default();
        let mut cfg = crate::core::renderer::ExportConfig::default();
        cfg.format = crate::core::renderer::ExportFormat::Svg;
        cfg.base_filename = "test_export".to_string();

        let results = crate::core::renderer::export_document(&doc, &cfg);
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].0, "test_export.svg");
        let content_str = String::from_utf8(results[0].1.clone()).expect("Valid UTF-8 SVG string");
        assert!(content_str.contains("<svg"));
        assert!(content_str.contains("</svg>"));
    }

    #[test]
    fn test_svg_export_and_import_linked_clone() {
        let mut doc = Document::default();
        let rect = crate::core::RectElement::new(
            Rect::new(10.0, 10.0, 100.0, 50.0),
            Some(Color::RED),
            None,
        );
        let master_id = rect.id;
        doc.add_element(crate::core::Element::Rect(rect));
        doc.select(master_id, false);
        doc.clone_selected();

        let svg_str = export_document_to_svg(&doc);
        assert!(svg_str.contains("<use"));
        assert!(svg_str.contains(&format!("href=\"#elem_{}\"", master_id.0)));

        let loaded = load_document_from_svg(&svg_str).expect("Should parse SVG with clone");
        assert_eq!(loaded.elements.len(), 2);
        assert!(matches!(loaded.elements[0], crate::core::Element::Rect(_)));
        assert!(matches!(loaded.elements[1], crate::core::Element::Clone(_)));
    }
}
