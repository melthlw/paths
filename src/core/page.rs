use super::geometry::{Point, Rect};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_PAGE_ID: AtomicU64 = AtomicU64::new(1);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize)]
pub struct PageId(pub u64);

impl PageId {
    pub fn new() -> Self {
        Self(NEXT_PAGE_ID.fetch_add(1, Ordering::Relaxed))
    }
}

impl Default for PageId {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
pub struct Page {
    pub id: PageId,
    pub name: String,
    pub rect: Rect,
}

impl Page {
    pub fn new(name: impl Into<String>, rect: Rect) -> Self {
        Self {
            id: PageId::new(),
            name: name.into(),
            rect: rect.round(),
        }
    }

    pub fn default_a4() -> Self {
        Self::new(
            "Page 1",
            Rect::new(0.0, 0.0, 794.0, 1123.0),
        )
    }

    pub fn bounds(&self) -> Rect {
        self.rect.normalize()
    }

    pub fn hit_test(&self, point: Point) -> bool {
        self.bounds().contains(point)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::Document;

    #[test]
    fn test_page_creation_and_bounds() {
        let page = Page::new("Página Teste", Rect::new(10.0, 20.0, 800.0, 600.0));
        assert_eq!(page.name, "Página Teste");
        assert_eq!(page.rect, Rect::new(10.0, 20.0, 800.0, 600.0));
        assert!(page.hit_test(Point::new(100.0, 100.0)));
        assert!(!page.hit_test(Point::new(5.0, 5.0)));
    }

    #[test]
    fn test_document_multi_page_lifecycle() {
        let mut doc = Document::default();
        assert_eq!(doc.pages.len(), 1);
        let p1_id = doc.pages[0].id;
        assert_eq!(doc.active_page_id, Some(p1_id));

        // Add next page
        let p2_id = doc.add_next_page();
        assert_eq!(doc.pages.len(), 2);
        assert_eq!(doc.active_page_id, Some(p2_id));
        assert_eq!(doc.pages[1].name, "Página 2");

        // Rename active page
        doc.rename_page(p2_id, "Capa".to_string());
        assert_eq!(doc.active_page().unwrap().name, "Capa");

        // Resize page
        doc.set_page_rect(p2_id, Rect::new(100.0, 100.0, 1920.0, 1080.0));
        assert_eq!(doc.active_page().unwrap().rect.width, 1920.0);

        // Undo resize & rename
        doc.undo(); // undo resize
        doc.undo(); // undo rename
        assert_eq!(doc.active_page().unwrap().name, "Página 2");

        // Redo
        doc.redo();
        assert_eq!(doc.active_page().unwrap().name, "Capa");

        // Remove page
        assert!(doc.remove_page(p2_id));
        assert_eq!(doc.pages.len(), 1);
        assert_eq!(doc.active_page_id, Some(p1_id));
    }
}
