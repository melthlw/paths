pub mod boolean;
pub mod clipboard;
pub mod history;
pub mod nodes;
pub mod ordering;
pub mod selection;
pub mod text_style;
pub mod transform;

pub use transform::TransformOptions;

use std::collections::HashSet;

use crate::core::color::Color;
use crate::core::element::{BlendMode, Element, ElementId};
use crate::core::geometry::{Point, Rect};
use crate::core::page::{Page, PageId};
use crate::core::ruler::{Guide, GuideOrientation};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BooleanOperation {
    Union,
    Difference,
    Intersection,
    Exclusion,
    Division,
    Cut,
}

#[derive(Debug, Clone)]
pub struct Document {
    pub elements: Vec<Element>,
    pub selected_ids: HashSet<ElementId>,
    pub pages: Vec<Page>,
    pub active_page_id: Option<PageId>,
    pub guides: Vec<Guide>,
    pub clipboard: Vec<Element>,
    pub unit: crate::core::Unit,
    undo_stack: Vec<(
        Vec<Element>,
        HashSet<ElementId>,
        Vec<Page>,
        Option<PageId>,
        Vec<Guide>,
    )>,
    redo_stack: Vec<(
        Vec<Element>,
        HashSet<ElementId>,
        Vec<Page>,
        Option<PageId>,
        Vec<Guide>,
    )>,
}

impl Default for Document {
    fn default() -> Self {
        let initial_page = Page::default_a4();
        let page_id = initial_page.id;
        Self {
            elements: Vec::new(),
            selected_ids: HashSet::new(),
            pages: vec![initial_page],
            active_page_id: Some(page_id),
            guides: Vec::new(),
            clipboard: Vec::new(),
            unit: crate::core::Unit::Px,
            undo_stack: Vec::new(),
            redo_stack: Vec::new(),
        }
    }
}

impl Document {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn active_page(&self) -> Option<&Page> {
        if let Some(id) = self.active_page_id {
            self.pages.iter().find(|p| p.id == id)
        } else {
            self.pages.first()
        }
    }

    pub fn select_page(&mut self, id: PageId) {
        if self.pages.iter().any(|p| p.id == id) {
            self.active_page_id = Some(id);
        }
    }

    pub fn add_page(&mut self, name: Option<String>, rect: Rect) -> PageId {
        self.snapshot();
        let page_idx = self.pages.len() + 1;
        let page_name = name.unwrap_or_else(|| crate::i18n!("Page {}", page_idx));
        let page = Page::new(page_name, rect);
        let id = page.id;
        self.pages.push(page);
        self.active_page_id = Some(id);
        id
    }

    pub fn add_next_page(&mut self) -> PageId {
        let last_rect = self
            .pages
            .last()
            .map(|p| p.rect)
            .unwrap_or_else(|| Page::default_a4().rect);
        let spacing = 60.0;
        let new_rect = Rect::new(
            last_rect.x + last_rect.width + spacing,
            last_rect.y,
            last_rect.width,
            last_rect.height,
        );
        let num = self.pages.len() + 1;
        let page = Page::new(format!("Página {}", num), new_rect);
        let id = page.id;
        self.pages.push(page);
        self.active_page_id = Some(id);
        id
    }

    pub fn remove_page(&mut self, id: PageId) -> bool {
        if self.pages.len() <= 1 {
            return false;
        }
        self.snapshot();
        self.pages.retain(|p| p.id != id);
        if self.active_page_id == Some(id) {
            self.active_page_id = self.pages.first().map(|p| p.id);
        }
        true
    }

    pub fn rename_page(&mut self, id: PageId, new_name: String) {
        self.snapshot();
        if let Some(page) = self.pages.iter_mut().find(|p| p.id == id) {
            page.name = new_name;
        }
    }

    pub fn set_page_rect(&mut self, id: PageId, new_rect: Rect) {
        self.snapshot();
        if let Some(page) = self.pages.iter_mut().find(|p| p.id == id) {
            page.rect = new_rect.round();
        }
    }

    pub fn add_guide(&mut self, guide: Guide) {
        self.snapshot();
        self.guides.push(guide);
    }

    pub fn remove_guide(&mut self, id: u64) {
        self.snapshot();
        self.guides.retain(|g| g.id != id);
    }

    pub fn update_guide_position(&mut self, id: u64, new_pos: f32) {
        self.snapshot();
        if let Some(g) = self.guides.iter_mut().find(|g| g.id == id) {
            g.position = new_pos;
        }
    }

    pub fn find_guide_near(&self, world_pos: Point, tolerance: f32) -> Option<&Guide> {
        for guide in &self.guides {
            match guide.orientation {
                GuideOrientation::Horizontal => {
                    if (guide.position - world_pos.y).abs() <= tolerance {
                        return Some(guide);
                    }
                }
                GuideOrientation::Vertical => {
                    if (guide.position - world_pos.x).abs() <= tolerance {
                        return Some(guide);
                    }
                }
            }
        }
        None
    }

    pub fn add_element(&mut self, element: Element) {
        self.snapshot();
        let id = element.id();
        self.elements.push(element);
        self.selected_ids.clear();
        self.selected_ids.insert(id);
    }

    pub fn remove_selected(&mut self) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.snapshot();
        let deleted_ids = self.selected_ids.clone();
        let old_elements = self.elements.clone();

        // Automatically unlink any clones whose master is being deleted
        let mut new_elements = Vec::new();
        for el in self.elements.drain(..) {
            if deleted_ids.contains(&el.id()) {
                continue;
            }
            if let Element::Clone(ref c) = el {
                if deleted_ids.contains(&c.source_id) {
                    if let Some(master) = Self::find_element_recursive(&old_elements, c.source_id) {
                        let concrete = Self::convert_clone_to_concrete(c, master);
                        new_elements.push(concrete);
                        continue;
                    }
                }
            }
            new_elements.push(el);
        }
        self.elements = new_elements;
        self.selected_ids.clear();
    }

    #[allow(dead_code)]
    pub fn clear(&mut self) {
        if self.elements.is_empty() {
            return;
        }
        self.snapshot();
        self.elements.clear();
        self.selected_ids.clear();
    }

    pub fn find_element(&self, id: ElementId) -> Option<&Element> {
        Self::find_element_recursive(&self.elements, id)
    }

    #[allow(dead_code)]
    pub fn find_element_mut(&mut self, id: ElementId) -> Option<&mut Element> {
        Self::find_element_mut_recursive(&mut self.elements, id)
    }

    pub fn element_bounds(&self, el: &Element) -> Rect {
        match el {
            Element::Clone(c) => {
                if let Some(master) = self.find_element(c.source_id) {
                    let mb = self.element_bounds(master);
                    let mut b = Rect::new(
                        mb.x + c.offset.x,
                        mb.y + c.offset.y,
                        mb.width * c.scale.x.abs().max(0.001),
                        mb.height * c.scale.y.abs().max(0.001),
                    );
                    if c.rotation.abs() > 0.001 {
                        let cx = mb.x + mb.width * 0.5 + c.offset.x;
                        let cy = mb.y + mb.height * 0.5 + c.offset.y;
                        let cos_a = c.rotation.cos();
                        let sin_a = c.rotation.sin();
                        let corners = [
                            Point::new(b.x, b.y),
                            Point::new(b.x + b.width, b.y),
                            Point::new(b.x + b.width, b.y + b.height),
                            Point::new(b.x, b.y + b.height),
                        ];
                        let mut min_x = f32::MAX;
                        let mut min_y = f32::MAX;
                        let mut max_x = f32::MIN;
                        let mut max_y = f32::MIN;
                        for cp in corners {
                            let dx = cp.x - cx;
                            let dy = cp.y - cy;
                            let rx = cx + (dx * cos_a - dy * sin_a);
                            let ry = cy + (dx * sin_a + dy * cos_a);
                            min_x = min_x.min(rx);
                            min_y = min_y.min(ry);
                            max_x = max_x.max(rx);
                            max_y = max_y.max(ry);
                        }
                        b = Rect::new(min_x, min_y, max_x - min_x, max_y - min_y);
                    }
                    b
                } else {
                    c.bounds()
                }
            }
            other => other.bounds(),
        }
    }

    pub fn element_hit_test(&self, el: &Element, p: Point) -> bool {
        match el {
            Element::Clone(c) => {
                if let Some(master) = self.find_element(c.source_id) {
                    let mut lp = Point::new(p.x - c.offset.x, p.y - c.offset.y);
                    if c.rotation.abs() > 0.001 {
                        let mb = self.element_bounds(master);
                        let cx = mb.x + mb.width * 0.5;
                        let cy = mb.y + mb.height * 0.5;
                        let dx = lp.x - cx;
                        let dy = lp.y - cy;
                        let cos_a = (-c.rotation).cos();
                        let sin_a = (-c.rotation).sin();
                        lp.x = cx + (dx * cos_a - dy * sin_a);
                        lp.y = cy + (dx * sin_a + dy * cos_a);
                    }
                    if (c.scale.x - 1.0).abs() > 0.001 || (c.scale.y - 1.0).abs() > 0.001 {
                        let mb = self.element_bounds(master);
                        lp.x = mb.x + (lp.x - mb.x) / c.scale.x.abs().max(0.001);
                        lp.y = mb.y + (lp.y - mb.y) / c.scale.y.abs().max(0.001);
                    }
                    master.hit_test(lp)
                } else {
                    c.hit_test(p)
                }
            }
            other => other.hit_test(p),
        }
    }

    pub fn get_document_colors(&self) -> Vec<Option<Color>> {
        let mut colors: Vec<Option<Color>> = vec![None];
        for el in &self.elements {
            el.collect_colors(&mut colors);
        }
        let mut unique = Vec::new();
        for c in colors {
            if !unique.contains(&c) {
                unique.push(c);
            }
        }
        unique
    }
}

#[cfg(test)]
pub mod tests {
    use super::*;
    use crate::core::color::Color;
    use crate::core::element::{FillLayer, FillStyle, RectElement, StrokeLayer, StrokeStyle};
    use crate::core::ruler::{Guide, GuideOrientation};

    #[test]
    fn test_guide_management_in_document() {
        let mut doc = Document::new();
        let g1 = Guide::new(GuideOrientation::Horizontal, 100.0);
        let id1 = g1.id;
        doc.add_guide(g1);

        assert_eq!(doc.guides.len(), 1);
        assert!(doc.find_guide_near(Point::new(50.0, 102.0), 5.0).is_some());
        assert!(doc.find_guide_near(Point::new(50.0, 120.0), 5.0).is_none());

        doc.update_guide_position(id1, 200.0);
        assert_eq!(doc.guides[0].position, 200.0);

        doc.remove_guide(id1);
        assert_eq!(doc.guides.len(), 0);
    }

    #[test]
    fn test_arrange_selected_in_grid() {
        let mut doc = Document::new();
        let r1 = crate::core::RectElement::new(Rect::new(0.0, 0.0, 50.0, 50.0), None, None);
        let r2 = crate::core::RectElement::new(Rect::new(10.0, 10.0, 50.0, 50.0), None, None);
        let r3 = crate::core::RectElement::new(Rect::new(20.0, 20.0, 50.0, 50.0), None, None);
        let r4 = crate::core::RectElement::new(Rect::new(30.0, 30.0, 50.0, 50.0), None, None);

        let id1 = r1.id;
        let id2 = r2.id;
        let id3 = r3.id;
        let id4 = r4.id;

        doc.add_element(Element::Rect(r1));
        doc.add_element(Element::Rect(r2));
        doc.add_element(Element::Rect(r3));
        doc.add_element(Element::Rect(r4));

        doc.select(id1, false);
        doc.select(id2, true);
        doc.select(id3, true);
        doc.select(id4, true);

        // Arrange in 2 rows x 2 cols with 10px gap
        doc.arrange_selected_in_grid(2, 2, 10.0, 10.0);

        let bounds: Vec<Rect> = doc.elements.iter().map(|e| e.bounds()).collect();
        assert_eq!(bounds[0].x, 0.0);
        assert_eq!(bounds[0].y, 0.0);
        assert_eq!(bounds[1].x, 60.0); // 50 + 10
        assert_eq!(bounds[1].y, 0.0);
        assert_eq!(bounds[2].x, 0.0);
        assert_eq!(bounds[2].y, 60.0);
        assert_eq!(bounds[3].x, 60.0);
        assert_eq!(bounds[3].y, 60.0);
    }

    #[test]
    fn test_boolean_operations() {
        let mut doc = Document::new();
        let r1 = crate::core::RectElement::new(Rect::new(0.0, 0.0, 100.0, 100.0), None, None);
        let r2 = crate::core::RectElement::new(Rect::new(50.0, 50.0, 100.0, 100.0), None, None);

        let id1 = r1.id;
        let id2 = r2.id;

        doc.add_element(Element::Rect(r1));
        doc.add_element(Element::Rect(r2));

        doc.select(id1, false);
        doc.select(id2, true);

        // Apply Union
        doc.apply_boolean_operation(BooleanOperation::Union);
        assert_eq!(doc.elements.len(), 1);
        let b = doc.elements[0].bounds();
        assert!(b.width >= 150.0);
        assert!(b.height >= 150.0);

        // Test Undo
        assert!(doc.undo());
        assert_eq!(doc.elements.len(), 2);
    }

    #[test]
    fn test_convert_selected_to_path() {
        let mut doc = Document::new();
        let r = crate::core::RectElement::new(Rect::new(10.0, 10.0, 100.0, 50.0), None, None);
        let id = r.id;
        doc.add_element(Element::Rect(r));
        doc.select(id, false);

        assert!(doc.convert_selected_to_path());
        assert_eq!(doc.elements.len(), 1);
        if let Element::Path(p) = &doc.elements[0] {
            assert_eq!(p.nodes.len(), 4);
        } else {
            panic!("Expected PathElement after conversion");
        }
    }

    #[test]
    fn test_layer_visibility_locking_and_reordering() {
        let mut doc = Document::new();
        let r1 = crate::core::RectElement::new(Rect::new(0.0, 0.0, 50.0, 50.0), None, None);
        let r2 = crate::core::RectElement::new(Rect::new(100.0, 100.0, 50.0, 50.0), None, None);
        let id1 = r1.id;
        let id2 = r2.id;

        doc.add_element(Element::Rect(r1));
        doc.add_element(Element::Rect(r2));

        let layers = doc.get_layers_info();
        assert_eq!(layers.len(), 2);
        // Top layer should be id2 (z_index 1)
        assert_eq!(layers[0].id, id2);
        assert_eq!(layers[1].id, id1);

        // Test visibility
        doc.set_element_visibility(id1, false);
        assert!(!doc.elements[0].visible());
        assert_eq!(doc.hit_test(Point::new(25.0, 25.0)), None);

        // Test locking
        doc.set_element_locked(id2, true);
        assert!(doc.elements[1].locked());
        assert_eq!(doc.hit_test(Point::new(125.0, 125.0)), None);

        // Test reordering / moving up
        doc.move_layer_up(id1);
        assert_eq!(doc.elements[1].id(), id1);
        assert_eq!(doc.elements[0].id(), id2);

        // Test undo
        assert!(doc.undo());
        assert_eq!(doc.elements[0].id(), id1);
    }

    #[test]
    fn test_clipboard_and_context_menu_operations() {
        let mut doc = Document::new();
        let r1 = Element::Rect(RectElement::new(
            Rect::new(0.0, 0.0, 50.0, 50.0),
            Some(Color::RED),
            None,
        ));
        let r2 = Element::Rect(RectElement::new(
            Rect::new(60.0, 0.0, 50.0, 50.0),
            Some(Color::BLUE),
            None,
        ));
        let r3 = Element::Rect(RectElement::new(
            Rect::new(120.0, 0.0, 50.0, 50.0),
            Some(Color::RED),
            None,
        ));
        let id1 = r1.id();
        let id2 = r2.id();
        let id3 = r3.id();
        doc.add_element(r1);
        doc.add_element(r2);
        doc.add_element(r3);

        // Test select_same_fill
        doc.select(id1, false);
        doc.select_same_fill();
        assert_eq!(doc.selected_ids.len(), 2);
        assert!(doc.selected_ids.contains(&id1));
        assert!(doc.selected_ids.contains(&id3));

        // Test duplicate_selected
        doc.select(id2, false);
        let dup_ids = doc.duplicate_selected();
        assert_eq!(dup_ids.len(), 1);
        assert_eq!(doc.elements.len(), 4);
        assert_eq!(doc.selected_ids.len(), 1);
        assert!(doc.selected_ids.contains(&dup_ids[0]));

        // Test copy and paste
        doc.select(id1, false);
        doc.copy_selected();
        assert_eq!(doc.clipboard.len(), 1);
        let paste_ids = doc.paste(Some(Point::new(20.0, 20.0)));
        assert_eq!(paste_ids.len(), 1);
        assert_eq!(doc.elements.len(), 5);

        // Test cut
        doc.select(id2, false);
        doc.cut_selected();
        assert_eq!(doc.elements.len(), 4);
        assert!(!doc.elements.iter().any(|e| e.id() == id2));

        // Test bring_to_front / send_to_back
        doc.select(id1, false);
        doc.bring_selected_to_front();
        assert_eq!(doc.elements.last().unwrap().id(), id1);
        doc.send_selected_to_back();
        assert_eq!(doc.elements.first().unwrap().id(), id1);

        // Test hide and lock
        doc.select(id1, false);
        doc.hide_selected();
        assert!(!doc.elements[0].visible());
        doc.select(id3, false);
        doc.lock_selected();
        assert!(doc
            .elements
            .iter()
            .find(|e| e.id() == id3)
            .unwrap()
            .locked());

        // Test group and ungroup
        let mut doc2 = Document::new();
        let el_a = Element::Rect(RectElement::new(
            Rect::new(0.0, 0.0, 40.0, 40.0),
            Some(Color::RED),
            None,
        ));
        let el_b = Element::Rect(RectElement::new(
            Rect::new(50.0, 50.0, 40.0, 40.0),
            Some(Color::BLUE),
            None,
        ));
        let ida = el_a.id();
        let idb = el_b.id();
        doc2.add_element(el_a);
        doc2.add_element(el_b);
        doc2.select(ida, false);
        doc2.select(idb, true);
        assert_eq!(doc2.selected_ids.len(), 2);
        let grp_id = doc2.group_selected().unwrap();
        assert_eq!(doc2.elements.len(), 1);
        assert_eq!(doc2.elements[0].id(), grp_id);

        // Test ungroup
        let unpacked = doc2.ungroup_selected();
        assert_eq!(unpacked.len(), 2);
        assert_eq!(doc2.elements.len(), 2);

        // Test clip group
        doc2.select(ida, false);
        doc2.select(idb, true);
        let clip_grp_id = doc2.set_clip_group_selected().unwrap();
        assert_eq!(doc2.elements.len(), 1);
        if let Element::Group(g) = &doc2.elements[0] {
            assert_eq!(g.id, clip_grp_id);
            assert!(g.clip_element.is_some());
        } else {
            panic!("Expected Group element");
        }
    }

    #[test]
    fn test_blend_mode_blur_and_opacity() {
        let mut doc = Document::new();
        let el = Element::Rect(RectElement::new(
            Rect::new(0.0, 0.0, 100.0, 100.0),
            Some(Color::RED),
            None,
        ));
        let id = el.id();
        doc.add_element(el);
        doc.select(id, false);

        assert_eq!(
            doc.get_selection_blend_info(),
            Some((BlendMode::Normal, 0.0, 1.0))
        );

        doc.set_selected_blend_mode(BlendMode::Multiply);
        doc.set_selected_blur(0.45);
        doc.set_selected_opacity(0.8);

        assert_eq!(
            doc.get_selection_blend_info(),
            Some((BlendMode::Multiply, 0.45, 0.8))
        );

        // Verify undo / redo
        doc.undo();
        assert_eq!(
            doc.get_selection_blend_info(),
            Some((BlendMode::Multiply, 0.45, 1.0))
        );
        doc.redo();
        assert_eq!(
            doc.get_selection_blend_info(),
            Some((BlendMode::Multiply, 0.45, 0.8))
        );
    }

    #[test]
    fn test_multi_fill_and_multi_stroke() {
        let mut doc = Document::new();
        let el = Element::Rect(RectElement::new(
            Rect::new(0.0, 0.0, 100.0, 100.0),
            Some(Color::RED),
            Some(Color::BLACK),
        ));
        let id = el.id();
        doc.add_element(el);
        doc.select(id, false);

        let (fills, strokes) = doc.get_selected_fills_and_strokes().unwrap();
        assert_eq!(fills.len(), 1);
        assert_eq!(strokes.len(), 1);

        let new_fills = vec![FillLayer::new(Color::RED), FillLayer::new(Color::BLUE)];
        let new_strokes = vec![
            StrokeLayer::new(Color::BLACK, 2.0),
            StrokeLayer::new(Color::EMERALD, 6.0),
        ];

        doc.set_selected_fills(new_fills.clone());
        doc.set_selected_strokes(new_strokes.clone());

        let (res_fills, res_strokes) = doc.get_selected_fills_and_strokes().unwrap();
        assert_eq!(res_fills.len(), 2);
        assert_eq!(res_strokes.len(), 2);

        // Undo stroke change
        doc.undo();
        let (_, strokes_after_undo) = doc.get_selected_fills_and_strokes().unwrap();
        assert_eq!(strokes_after_undo.len(), 1);

        // Redo stroke change
        doc.redo();
        let (_, strokes_after_redo) = doc.get_selected_fills_and_strokes().unwrap();
        assert_eq!(strokes_after_redo.len(), 2);
    }

    #[test]
    fn test_fill_styles_and_stroke_styles() {
        let mut doc = Document::new();
        let el = Element::Rect(crate::core::RectElement::new(
            Rect::new(0.0, 0.0, 100.0, 100.0),
            Some(Color::RED),
            None,
        ));
        let id = el.id();
        doc.add_element(el);
        doc.select(id, false);

        let solid_fill = FillLayer::new(Color::BLUE);
        let mut linear_fill = FillLayer::new(Color::RED);
        linear_fill.style = FillStyle::LinearGradient;
        linear_fill.secondary_color = Color::EMERALD;
        linear_fill.angle = 45.0;
        let mut radial_fill = FillLayer::new(Color::WHITE);
        radial_fill.style = FillStyle::RadialGradient;
        radial_fill.secondary_color = Color::BLACK;

        let solid_stroke = StrokeLayer::new(Color::BLACK, 2.0);
        let mut dashed_stroke = StrokeLayer::new(Color::WHITE, 4.0);
        dashed_stroke.style = StrokeStyle::Dashed;

        doc.set_selected_fills(vec![solid_fill, linear_fill, radial_fill]);
        doc.set_selected_strokes(vec![solid_stroke, dashed_stroke]);

        let (fills, strokes) = doc.get_selected_fills_and_strokes().unwrap();
        assert_eq!(fills.len(), 3);
        assert_eq!(fills[0].style, FillStyle::Solid);
        assert_eq!(fills[1].style, FillStyle::LinearGradient);
        assert_eq!(fills[1].angle, 45.0);
        assert_eq!(fills[2].style, FillStyle::RadialGradient);

        assert_eq!(strokes.len(), 2);
        assert_eq!(strokes[0].style, StrokeStyle::Solid);
        assert_eq!(strokes[1].style, StrokeStyle::Dashed);
        assert_eq!(strokes[1].width, 4.0);
    }

    #[test]
    fn test_linked_clone_lifecycle_and_transformations() {
        let mut doc = Document::new();
        let master = Element::Rect(crate::core::RectElement::new(
            Rect::new(50.0, 50.0, 100.0, 80.0),
            Some(Color::RED),
            None,
        ));
        let master_id = master.id();
        doc.add_element(master);
        doc.select(master_id, false);

        // 1. Create linked clone
        let clone_ids = doc.clone_selected();
        assert_eq!(clone_ids.len(), 1);
        let clone_id = clone_ids[0];
        assert_eq!(doc.elements.len(), 2);
        assert!(doc.has_clones_selected());
        assert!(!doc.has_masters_selected());

        // Clone bounds should follow master + initial offset (20.0, 20.0)
        let clone_el = doc.find_element(clone_id).unwrap();
        let bounds = doc.element_bounds(clone_el);
        assert_eq!(bounds.x, 70.0);
        assert_eq!(bounds.y, 70.0);
        assert_eq!(bounds.width, 100.0);
        assert_eq!(bounds.height, 80.0);

        // 2. Hit-test on clone
        assert!(doc.element_hit_test(clone_el, Point::new(80.0, 80.0)));
        assert!(!doc.element_hit_test(clone_el, Point::new(10.0, 10.0)));

        // 3. Move master: spatial transformations are decoupled so unselected clone stays at (70.0, 70.0)
        doc.select(master_id, false);
        assert!(!doc.has_clones_selected());
        assert!(doc.has_masters_selected());
        doc.translate_selected(30.0, 10.0);
        let clone_el_updated = doc.find_element(clone_id).unwrap();
        let updated_bounds = doc.element_bounds(clone_el_updated);
        assert_eq!(updated_bounds.x, 70.0);
        assert_eq!(updated_bounds.y, 70.0);

        // 4. Select master from clone
        doc.select(clone_id, false);
        doc.select_original_element();
        assert!(doc.selected_ids.contains(&master_id));

        // 5. Select clones from master
        doc.select_linked_clones();
        assert!(doc.selected_ids.contains(&clone_id));

        // 6. Unlink clone
        let unlinked_ids = doc.unlink_selected_clones();
        assert_eq!(unlinked_ids.len(), 1);
        assert_eq!(doc.elements.len(), 2);
        assert!(!doc.has_clones_selected());

        // The unlinked element is now an independent Rect at (70.0, 70.0)
        let unlinked_el = doc.find_element(unlinked_ids[0]).unwrap();
        assert!(matches!(unlinked_el, Element::Rect(_)));
        assert_eq!(unlinked_el.bounds().x, 70.0);
        assert_eq!(unlinked_el.bounds().y, 70.0);
    }

    #[test]
    fn test_linked_clone_auto_unlink_on_master_deletion() {
        let mut doc = Document::new();
        let master = Element::Rect(crate::core::RectElement::new(
            Rect::new(10.0, 10.0, 50.0, 50.0),
            Some(Color::BLUE),
            None,
        ));
        let master_id = master.id();
        doc.add_element(master);
        doc.select(master_id, false);

        let clone_ids = doc.clone_selected();
        assert_eq!(clone_ids.len(), 1);

        // Delete the master element
        doc.select(master_id, false);
        doc.remove_selected();

        // The clone was preserved and automatically converted into an independent element
        assert_eq!(doc.elements.len(), 1);
        assert!(!matches!(doc.elements[0], Element::Clone(_)));
        assert_eq!(doc.elements[0].bounds().x, 30.0);
        assert_eq!(doc.elements[0].bounds().y, 30.0);
    }

    #[test]
    fn test_linked_clones_spatial_decoupling() {
        let mut doc = Document::new();
        let master = Element::Rect(crate::core::RectElement::new(
            Rect::new(100.0, 100.0, 60.0, 40.0),
            Some(Color::RED),
            None,
        ));
        let master_id = master.id();
        doc.add_element(master);
        doc.select(master_id, false);

        // Create two clones
        let clone_ids1 = doc.clone_selected();
        let clone1_id = clone_ids1[0];

        doc.select(master_id, false);
        let clone_ids2 = doc.clone_selected();
        let clone2_id = clone_ids2[0];

        // Move clone2 independently to offset (150, 80)
        doc.select(clone2_id, false);
        doc.translate_selected(130.0, 60.0);

        let b_master = doc.find_element(master_id).unwrap().bounds();
        let b_clone1 = doc.element_bounds(doc.find_element(clone1_id).unwrap());
        let b_clone2 = doc.element_bounds(doc.find_element(clone2_id).unwrap());

        assert_eq!(b_master, Rect::new(100.0, 100.0, 60.0, 40.0));
        assert_eq!(b_clone1, Rect::new(120.0, 120.0, 60.0, 40.0));
        assert_eq!(b_clone2, Rect::new(250.0, 180.0, 60.0, 40.0));

        // Now move master by (50, 30) - clones should remain decoupled and stay at (120, 120) and (250, 180)
        doc.select(master_id, false);
        doc.translate_selected(50.0, 30.0);

        let b_master_after = doc.find_element(master_id).unwrap().bounds();
        let b_clone1_after = doc.element_bounds(doc.find_element(clone1_id).unwrap());
        let b_clone2_after = doc.element_bounds(doc.find_element(clone2_id).unwrap());

        assert_eq!(b_master_after, Rect::new(150.0, 130.0, 60.0, 40.0));
        assert_eq!(b_clone1_after, Rect::new(120.0, 120.0, 60.0, 40.0));
        assert_eq!(b_clone2_after, Rect::new(250.0, 180.0, 60.0, 40.0));
    }

    #[test]
    fn test_clone_management_queries_and_selective_unlinking() {
        let mut doc = Document::new();
        let master1 = Element::Rect(crate::core::RectElement::new(
            Rect::new(0.0, 0.0, 50.0, 50.0),
            Some(Color::EMERALD),
            None,
        ));
        let master1_id = master1.id();
        doc.add_element(master1);

        doc.select(master1_id, false);
        let c1_ids = doc.clone_selected();
        let c1 = c1_ids[0];

        doc.select(master1_id, false);
        let c2_ids = doc.clone_selected();
        let _c2 = c2_ids[0];

        // Verify relationships
        let clones = doc.get_clones_for_master(master1_id);
        assert_eq!(clones.len(), 2);

        let all_rels = doc.get_all_clone_relationships();
        assert_eq!(all_rels.len(), 1);
        assert_eq!(all_rels[0].0, master1_id);
        assert_eq!(all_rels[0].1.len(), 2);

        let master_of_c1 = doc.get_master_for_clone(c1).unwrap();
        assert_eq!(master_of_c1.id(), master1_id);

        // Unlink single clone c1
        let unlinked = doc.unlink_clone_by_id(c1);
        assert!(unlinked.is_some());
        let unlinked_id = unlinked.unwrap();

        // c1 is now a concrete element and no longer in get_clones_for_master
        assert_eq!(doc.get_clones_for_master(master1_id).len(), 1);
        let concrete = doc.find_element(unlinked_id).unwrap();
        assert!(matches!(concrete, Element::Rect(_)));

        // Unlink all remaining clones for master1
        let remaining_unlinked = doc.unlink_all_clones_for_master(master1_id);
        assert_eq!(remaining_unlinked.len(), 1);
        assert!(doc.get_clones_for_master(master1_id).is_empty());
        assert!(doc.get_all_clone_relationships().is_empty());
    }
}
