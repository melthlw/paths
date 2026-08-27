use std::collections::HashMap;
use skia_safe as skia;
use crate::core::document::Document;
use crate::core::element::{Element, ElementId};

#[derive(Hash, Eq, PartialEq, Clone, Copy, Debug)]
pub struct PictureCacheKey {
    pub id: ElementId,
    pub x_bits: u32,
    pub y_bits: u32,
    pub w_bits: u32,
    pub h_bits: u32,
}

#[derive(Default)]
pub struct PictureCache {
    pictures: HashMap<PictureCacheKey, skia::Picture>,
}

impl PictureCache {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn get_or_record(&mut self, element: &Element, document: &Document) -> Option<skia::Picture> {
        let bounds = element.bounds().normalize();
        let key = PictureCacheKey {
            id: element.id(),
            x_bits: bounds.x.to_bits(),
            y_bits: bounds.y.to_bits(),
            w_bits: bounds.width.to_bits(),
            h_bits: bounds.height.to_bits(),
        };

        if let Some(picture) = self.pictures.get(&key) {
            return Some(picture.clone());
        }

        let mut recorder = skia::PictureRecorder::new();
        let sk_bounds = skia::Rect::from_xywh(
            bounds.x - 100.0,
            bounds.y - 100.0,
            bounds.width + 200.0,
            bounds.height + 200.0,
        );

        let canvas = recorder.begin_recording(sk_bounds, false);
        element.render_with_doc(canvas, Some(document));

        if let Some(picture) = recorder.finish_recording_as_picture(None) {
            self.pictures.insert(key, picture.clone());
            Some(picture)
        } else {
            None
        }
    }

    #[allow(dead_code)]
    pub fn invalidate(&mut self, id: ElementId) {
        self.pictures.retain(|k, _| k.id != id);
    }

    pub fn clear(&mut self) {
        self.pictures.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::element::RectElement;
    use crate::core::geometry::Rect;

    #[test]
    fn test_picture_cache_lifecycle() {
        let mut cache = PictureCache::new();
        let doc = Document::default();
        let elem = Element::Rect(RectElement::new(Rect::new(0.0, 0.0, 100.0, 100.0), None, None));

        // First query records picture
        let pic1 = cache.get_or_record(&elem, &doc);
        assert!(pic1.is_some());

        // Invalidate single element
        cache.invalidate(elem.id());
        assert!(cache.pictures.is_empty());

        // Re-record and clear all
        let _ = cache.get_or_record(&elem, &doc);
        assert_eq!(cache.pictures.len(), 1);
        cache.clear();
        assert!(cache.pictures.is_empty());
    }
}
