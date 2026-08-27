use rstar::{AABB, RTree, RTreeObject};
use crate::core::element::Element;
use crate::core::geometry::Rect;

#[derive(Debug, Clone, PartialEq)]
pub struct SpatialElement {
    pub id: u64,
    pub index: usize,
    pub bounds: [f32; 4], // [min_x, min_y, max_x, max_y]
}

impl RTreeObject for SpatialElement {
    type Envelope = AABB<[f32; 2]>;

    fn envelope(&self) -> Self::Envelope {
        AABB::from_corners(
            [self.bounds[0], self.bounds[1]],
            [self.bounds[2], self.bounds[3]],
        )
    }
}

#[derive(Debug, Default, Clone)]
pub struct SpatialIndex {
    tree: RTree<SpatialElement>,
}

impl SpatialIndex {

    pub fn build(elements: &[Element]) -> Self {
        let mut items = Vec::with_capacity(elements.len());
        for (idx, elem) in elements.iter().enumerate() {
            let bounds = elem.bounds();
            let norm = bounds.normalize();
            // Add a small 5px buffer to account for stroke width and handles
            items.push(SpatialElement {
                id: elem.id().0,
                index: idx,
                bounds: [
                    norm.x - 5.0,
                    norm.y - 5.0,
                    norm.x + norm.width + 5.0,
                    norm.y + norm.height + 5.0,
                ],
            });
        }
        Self {
            tree: RTree::bulk_load(items),
        }
    }

    pub fn query_rect(&self, rect: Rect) -> Vec<usize> {
        let norm = rect.normalize();
        let envelope = AABB::from_corners(
            [norm.x, norm.y],
            [norm.x + norm.width, norm.y + norm.height],
        );
        let mut indices: Vec<usize> = self
            .tree
            .locate_in_envelope_intersecting(envelope)
            .map(|item| item.index)
            .collect();
        indices.sort_unstable();
        indices
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::element::RectElement;
    use crate::core::geometry::Rect;

    #[test]
    fn test_spatial_index_culling() {
        let elem1 = Element::Rect(RectElement::new(Rect::new(0.0, 0.0, 50.0, 50.0), None, None));
        let elem2 = Element::Rect(RectElement::new(Rect::new(1000.0, 1000.0, 50.0, 50.0), None, None));
        let elements = vec![elem1, elem2];

        let spatial_index = SpatialIndex::build(&elements);

        // Query visible area covering elem1 only
        let query_visible = spatial_index.query_rect(Rect::new(-10.0, -10.0, 100.0, 100.0));
        assert_eq!(query_visible, vec![0]);

        // Query visible area covering elem2 only
        let query_elem2 = spatial_index.query_rect(Rect::new(990.0, 990.0, 100.0, 100.0));
        assert_eq!(query_elem2, vec![1]);

        // Query area covering both
        let query_all = spatial_index.query_rect(Rect::new(-10.0, -10.0, 2000.0, 2000.0));
        assert_eq!(query_all, vec![0, 1]);
    }
}
