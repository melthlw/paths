use crate::core::element::{BlendMode, CloneElement, Element, ElementId};
use crate::core::geometry::{Point, Rect};
use super::Document;

impl Document {
    pub fn copy_selected(&mut self) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.clipboard = self
            .elements
            .iter()
            .filter(|e| self.selected_ids.contains(&e.id()))
            .cloned()
            .collect();
    }

    pub fn cut_selected(&mut self) {
        if self.selected_ids.is_empty() {
            return;
        }
        self.copy_selected();
        self.remove_selected();
    }

    pub fn paste(&mut self, offset: Option<Point>) -> Vec<ElementId> {
        if self.clipboard.is_empty() {
            return Vec::new();
        }
        self.snapshot();
        let d = offset.unwrap_or(Point::new(15.0, 15.0));
        let mut new_ids = Vec::new();
        self.selected_ids.clear();
        for el in &self.clipboard {
            let mut new_el = el.clone_with_new_id();
            new_el.translate(d.x, d.y);
            let new_id = new_el.id();
            self.elements.push(new_el);
            self.selected_ids.insert(new_id);
            new_ids.push(new_id);
        }
        new_ids
    }

    pub fn duplicate_selected(&mut self) -> Vec<ElementId> {
        if self.selected_ids.is_empty() {
            return Vec::new();
        }
        self.snapshot();
        let mut new_elements = Vec::new();
        let mut new_ids = Vec::new();
        for el in self
            .elements
            .iter()
            .filter(|e| self.selected_ids.contains(&e.id()))
        {
            let mut new_el = el.clone_with_new_id();
            new_el.translate(15.0, 15.0);
            new_ids.push(new_el.id());
            new_elements.push(new_el);
        }
        self.selected_ids.clear();
        for id in &new_ids {
            self.selected_ids.insert(*id);
        }
        self.elements.extend(new_elements);
        new_ids
    }

    pub fn has_clones_selected(&self) -> bool {
        self.elements
            .iter()
            .any(|e| self.selected_ids.contains(&e.id()) && matches!(e, Element::Clone(_)))
    }

    pub fn has_masters_selected(&self) -> bool {
        self.elements
            .iter()
            .any(|e| matches!(e, Element::Clone(c) if self.selected_ids.contains(&c.source_id)))
    }

    pub fn clone_selected(&mut self) -> Vec<ElementId> {
        if self.selected_ids.is_empty() {
            return Vec::new();
        }
        self.snapshot();
        let mut new_elements = Vec::new();
        let mut new_ids = Vec::new();

        for el in self
            .elements
            .iter()
            .filter(|e| self.selected_ids.contains(&e.id()))
        {
            let (
                source_id,
                initial_offset,
                initial_scale,
                initial_rotation,
                initial_opacity,
                initial_blend,
                initial_blur,
                name,
            ) = match el {
                Element::Clone(c) => {
                    // Clone of a clone references the original master
                    let offset = Point::new(c.offset.x + 20.0, c.offset.y + 20.0);
                    (
                        c.source_id,
                        offset,
                        c.scale,
                        c.rotation,
                        c.opacity,
                        c.blend_mode,
                        c.blur_radius,
                        c.name.clone(),
                    )
                }
                other => {
                    let offset = Point::new(20.0, 20.0);
                    let name = Some(crate::i18n!("Clone of {}", other.name()));
                    (
                        other.id(),
                        offset,
                        Point::new(1.0, 1.0),
                        0.0,
                        1.0,
                        BlendMode::Normal,
                        0.0,
                        name,
                    )
                }
            };

            let mut clone_elem = CloneElement::new(source_id, initial_offset);
            clone_elem.scale = initial_scale;
            clone_elem.rotation = initial_rotation;
            clone_elem.opacity = initial_opacity;
            clone_elem.blend_mode = initial_blend;
            clone_elem.blur_radius = initial_blur;
            clone_elem.name = name;

            if let Some(master) = Self::find_element_recursive(&self.elements, source_id) {
                let mb = master.bounds();
                clone_elem.cached_bounds = Some(Rect::new(
                    mb.x + clone_elem.offset.x,
                    mb.y + clone_elem.offset.y,
                    mb.width * clone_elem.scale.x.abs().max(0.001),
                    mb.height * clone_elem.scale.y.abs().max(0.001),
                ));
            }

            let new_id = clone_elem.id;
            new_ids.push(new_id);
            new_elements.push(Element::Clone(clone_elem));
        }

        self.selected_ids.clear();
        for id in &new_ids {
            self.selected_ids.insert(*id);
        }
        self.elements.extend(new_elements);
        new_ids
    }

    pub(crate) fn convert_clone_to_concrete(c: &CloneElement, master: &Element) -> Element {
        let mut concrete = master.clone_with_new_id();
        concrete.translate(c.offset.x, c.offset.y);
        if (c.scale.x - 1.0).abs() > 0.001 || (c.scale.y - 1.0).abs() > 0.001 {
            let b = concrete.bounds();
            concrete.scale(
                Point::new(b.x, b.y),
                c.scale.x,
                c.scale.y,
            );
        }
        if c.rotation.abs() > 0.001 {
            let b = concrete.bounds();
            concrete.rotate(
                Point::new(b.x + b.width * 0.5, b.y + b.height * 0.5),
                c.rotation,
            );
        }
        concrete.set_opacity(c.opacity * concrete.opacity());
        if c.blend_mode != BlendMode::Normal {
            concrete.set_blend_mode(c.blend_mode);
        }
        if c.blur_radius > 0.001 {
            concrete.set_blur(c.blur_radius);
        }
        concrete
    }

    pub fn unlink_selected_clones(&mut self) -> Vec<ElementId> {
        if !self.has_clones_selected() {
            return Vec::new();
        }
        self.snapshot();
        let mut unlinked_ids = Vec::new();
        let old_elements = self.elements.clone();
        let mut new_elements = Vec::new();

        for el in self.elements.drain(..) {
            if self.selected_ids.contains(&el.id()) {
                if let Element::Clone(ref c) = el {
                    if let Some(master) = Self::find_element_recursive(&old_elements, c.source_id) {
                        let concrete = Self::convert_clone_to_concrete(c, master);
                        unlinked_ids.push(concrete.id());
                        new_elements.push(concrete);
                        continue;
                    }
                }
            }
            new_elements.push(el);
        }

        self.elements = new_elements;
        self.selected_ids.clear();
        for id in &unlinked_ids {
            self.selected_ids.insert(*id);
        }
        unlinked_ids
    }

    pub fn unlink_clone_by_id(&mut self, clone_id: ElementId) -> Option<ElementId> {
        self.snapshot();
        let old_elements = self.elements.clone();
        let mut unlinked_id = None;
        let mut new_elements = Vec::new();

        for el in self.elements.drain(..) {
            if el.id() == clone_id {
                if let Element::Clone(ref c) = el {
                    if let Some(master) = Self::find_element_recursive(&old_elements, c.source_id) {
                        let concrete = Self::convert_clone_to_concrete(c, master);
                        let nid = concrete.id();
                        unlinked_id = Some(nid);
                        new_elements.push(concrete);
                        continue;
                    }
                }
            }
            new_elements.push(el);
        }

        self.elements = new_elements;
        if let Some(nid) = unlinked_id {
            if self.selected_ids.contains(&clone_id) {
                self.selected_ids.remove(&clone_id);
                self.selected_ids.insert(nid);
            }
        }
        unlinked_id
    }

    pub fn unlink_all_clones_for_master(&mut self, master_id: ElementId) -> Vec<ElementId> {
        self.snapshot();
        let old_elements = self.elements.clone();
        let mut unlinked_ids = Vec::new();
        let mut new_elements = Vec::new();

        for el in self.elements.drain(..) {
            if let Element::Clone(ref c) = el {
                if c.source_id == master_id {
                    if let Some(master) = Self::find_element_recursive(&old_elements, c.source_id) {
                        let concrete = Self::convert_clone_to_concrete(c, master);
                        let nid = concrete.id();
                        unlinked_ids.push(nid);
                        if self.selected_ids.contains(&c.id) {
                            self.selected_ids.remove(&c.id);
                            self.selected_ids.insert(nid);
                        }
                        new_elements.push(concrete);
                        continue;
                    }
                }
            }
            new_elements.push(el);
        }

        self.elements = new_elements;
        unlinked_ids
    }

    pub fn get_clones_for_master(&self, master_id: ElementId) -> Vec<CloneElement> {
        let mut clones = Vec::new();
        for el in &self.elements {
            if let Element::Clone(c) = el {
                if c.source_id == master_id {
                    clones.push(c.clone());
                }
            }
        }
        clones
    }

    pub fn get_all_clone_relationships(&self) -> Vec<(ElementId, Vec<CloneElement>)> {
        let mut map: std::collections::BTreeMap<u64, (ElementId, Vec<CloneElement>)> = std::collections::BTreeMap::new();
        for el in &self.elements {
            if let Element::Clone(c) = el {
                map.entry(c.source_id.0)
                    .or_insert_with(|| (c.source_id, Vec::new()))
                    .1
                    .push(c.clone());
            }
        }
        map.into_values().collect()
    }

    pub fn get_master_for_clone(&self, clone_id: ElementId) -> Option<&Element> {
        for el in &self.elements {
            if let Element::Clone(c) = el {
                if c.id == clone_id {
                    return self.find_element(c.source_id);
                }
            }
        }
        None
    }

    pub fn select_original_element(&mut self) {
        for el in &self.elements {
            if self.selected_ids.contains(&el.id()) {
                if let Element::Clone(c) = el {
                    let source_id = c.source_id;
                    self.selected_ids.clear();
                    self.selected_ids.insert(source_id);
                    return;
                }
            }
        }
    }

    pub fn select_linked_clones(&mut self) {
        let master_ids: Vec<ElementId> = self.selected_ids.iter().cloned().collect();
        let mut clone_ids = Vec::new();
        for el in &self.elements {
            if let Element::Clone(c) = el {
                if master_ids.contains(&c.source_id) {
                    clone_ids.push(c.id);
                }
            }
        }
        if !clone_ids.is_empty() {
            self.selected_ids.clear();
            for id in clone_ids {
                self.selected_ids.insert(id);
            }
        }
    }

    pub fn delete_clones_for_master(&mut self, master_id: ElementId) -> usize {
        self.snapshot();
        let mut count = 0;
        self.elements.retain(|el| {
            if let Element::Clone(c) = el {
                if c.source_id == master_id {
                    self.selected_ids.remove(&c.id);
                    count += 1;
                    return false;
                }
            }
            true
        });
        count
    }

    pub fn create_tiled_clones(
        &mut self,
        master_id: ElementId,
        params: &TiledCloneParams,
    ) -> Vec<ElementId> {
        let master = match self.find_element(master_id) {
            Some(el) => el.clone(),
            None => return Vec::new(),
        };

        let actual_master_id = if let Element::Clone(ref c) = master {
            c.source_id
        } else {
            master.id()
        };

        let master_el = match self.find_element(actual_master_id) {
            Some(el) => el.clone(),
            None => return Vec::new(),
        };

        let mb = master_el.bounds();
        let mw = mb.width.max(1.0);
        let mh = mb.height.max(1.0);
        let master_name = master_el.name();

        self.snapshot();

        let rows = params.rows.max(1);
        let cols = params.cols.max(1);
        let total_items = rows * cols;
        let mut created_ids = Vec::new();

        let step_x = mw * (params.shift_x_pct / 100.0);
        let step_y = mh * (params.shift_y_pct / 100.0);

        let pseudo_rand = |seed: f32| -> f32 {
            ((seed * 12.9898 + 78.233).sin() * 43758.5453).fract() * 2.0 - 1.0
        };

        if params.symmetry == TiledCloneSymmetry::Radial {
            let radius = if params.radial_radius > 0.0 {
                params.radial_radius
            } else {
                (mw.max(mh) * 2.0).max(60.0)
            };

            for i in 1..total_items {
                let theta = (i as f32) * (std::f32::consts::TAU / (total_items as f32));
                let dx = radius * theta.cos() - radius;
                let dy = radius * theta.sin();

                let mut rot_rad = theta;
                let mut sx = 1.0f32;
                let mut sy = 1.0f32;

                let seed = i as f32 * 17.13;
                let r_rot = pseudo_rand(seed) * params.rotate_rand_deg;
                rot_rad += r_rot.to_radians();

                let r_scale = 1.0 + (pseudo_rand(seed + 1.0) * (params.scale_rand_pct / 100.0));
                sx *= r_scale.max(0.05);
                sy *= r_scale.max(0.05);

                let op = (1.0
                    - (i as f32 / total_items as f32)
                        * (params.opacity_delta_per_row_pct / 100.0))
                    .clamp(0.05, 1.0);

                let mut clone = CloneElement::new(actual_master_id, Point::new(dx, dy));
                clone.scale = Point::new(sx, sy);
                clone.rotation = rot_rad;
                clone.opacity = op;
                clone.name = Some(format!("{} (Radial {})", master_name, i));

                let mut cb = mb;
                cb.x += dx;
                cb.y += dy;
                cb.width *= sx.abs();
                cb.height *= sy.abs();
                clone.cached_bounds = Some(cb);

                let cid = clone.id;
                self.elements.push(Element::Clone(clone));
                created_ids.push(cid);
            }
        } else {
            for r in 0..rows {
                for c in 0..cols {
                    if r == 0 && c == 0 {
                        continue;
                    }

                    let mut dx = (c as f32) * step_x;
                    let mut dy = (r as f32) * step_y;

                    let mut sym_sx = 1.0f32;
                    let mut sym_sy = 1.0f32;
                    let mut sym_rot_rad = 0.0f32;

                    match params.symmetry {
                        TiledCloneSymmetry::P1 => {}
                        TiledCloneSymmetry::P2 => {
                            if (r + c) % 2 == 1 {
                                sym_rot_rad += std::f32::consts::PI;
                            }
                        }
                        TiledCloneSymmetry::PM => {
                            if c % 2 == 1 {
                                sym_sx = -1.0;
                            }
                        }
                        TiledCloneSymmetry::PG => {
                            if c % 2 == 1 {
                                sym_sx = -1.0;
                                dy += step_y * 0.5;
                            }
                        }
                        TiledCloneSymmetry::CM => {
                            if (r + c) % 2 == 1 {
                                sym_sx = -1.0;
                            }
                        }
                        TiledCloneSymmetry::PMM => {
                            if c % 2 == 1 {
                                sym_sx = -1.0;
                            }
                            if r % 2 == 1 {
                                sym_sy = -1.0;
                            }
                        }
                        TiledCloneSymmetry::PMG => {
                            if c % 2 == 1 {
                                sym_sx = -1.0;
                            }
                            if r % 2 == 1 {
                                dx += step_x * 0.5;
                            }
                        }
                        TiledCloneSymmetry::PGG => {
                            if c % 2 == 1 {
                                sym_sx = -1.0;
                                dy += step_y * 0.5;
                            }
                            if r % 2 == 1 {
                                sym_sy = -1.0;
                                dx += step_x * 0.5;
                            }
                        }
                        TiledCloneSymmetry::P4 => {
                            let step = ((r * cols + c) % 4) as f32;
                            sym_rot_rad += step * (std::f32::consts::FRAC_PI_2);
                        }
                        TiledCloneSymmetry::P6 => {
                            let step = ((r * cols + c) % 6) as f32;
                            sym_rot_rad += step * (std::f32::consts::PI / 3.0);
                            if r % 2 == 1 {
                                dx += step_x * 0.5;
                            }
                        }
                        TiledCloneSymmetry::Radial => unreachable!(),
                    }

                    let seed = (r * 31 + c * 17) as f32;
                    let rand_x = pseudo_rand(seed) * mw * (params.shift_x_rand_pct / 100.0);
                    let rand_y = pseudo_rand(seed + 1.0) * mh * (params.shift_y_rand_pct / 100.0);
                    dx += rand_x;
                    dy += rand_y;

                    let base_sx = (1.0 + (c as f32) * (params.scale_x_pct / 100.0)).max(0.05);
                    let base_sy = (1.0 + (r as f32) * (params.scale_y_pct / 100.0)).max(0.05);
                    let rand_s =
                        1.0 + pseudo_rand(seed + 2.0) * (params.scale_rand_pct / 100.0);
                    let sx = base_sx * rand_s.max(0.05) * sym_sx;
                    let sy = base_sy * rand_s.max(0.05) * sym_sy;

                    let rot_deg = (c as f32) * params.rotate_per_col_deg
                        + (r as f32) * params.rotate_per_row_deg;
                    let rand_rot_deg = pseudo_rand(seed + 3.0) * params.rotate_rand_deg;
                    let rot_rad = (rot_deg + rand_rot_deg).to_radians() + sym_rot_rad;

                    let op = (1.0
                        - (r as f32) * (params.opacity_delta_per_row_pct / 100.0))
                        .clamp(0.05, 1.0);

                    let mut clone = CloneElement::new(actual_master_id, Point::new(dx, dy));
                    clone.scale = Point::new(sx, sy);
                    clone.rotation = rot_rad;
                    clone.opacity = op;
                    clone.name = Some(format!("{} (Tile {},{})", master_name, r + 1, c + 1));

                    let mut cb = mb;
                    cb.x += dx;
                    cb.y += dy;
                    cb.width *= sx.abs();
                    cb.height *= sy.abs();
                    clone.cached_bounds = Some(cb);

                    let cid = clone.id;
                    self.elements.push(Element::Clone(clone));
                    created_ids.push(cid);
                }
            }
        }

        if !created_ids.is_empty() {
            self.selected_ids.clear();
            for id in &created_ids {
                self.selected_ids.insert(*id);
            }
        }

        created_ids
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum TiledCloneSymmetry {
    #[default]
    P1,
    P2,
    PM,
    PG,
    CM,
    PMM,
    PMG,
    PGG,
    P4,
    P6,
    Radial,
}

#[derive(Debug, Clone, PartialEq)]
pub struct TiledCloneParams {
    pub rows: usize,
    pub cols: usize,
    pub symmetry: TiledCloneSymmetry,
    pub shift_x_pct: f32,
    pub shift_y_pct: f32,
    pub shift_x_rand_pct: f32,
    pub shift_y_rand_pct: f32,
    pub scale_x_pct: f32,
    pub scale_y_pct: f32,
    pub scale_rand_pct: f32,
    pub rotate_per_col_deg: f32,
    pub rotate_per_row_deg: f32,
    pub rotate_rand_deg: f32,
    pub opacity_delta_per_row_pct: f32,
    pub radial_radius: f32,
}

impl Default for TiledCloneParams {
    fn default() -> Self {
        Self {
            rows: 3,
            cols: 3,
            symmetry: TiledCloneSymmetry::P1,
            shift_x_pct: 100.0,
            shift_y_pct: 100.0,
            shift_x_rand_pct: 0.0,
            shift_y_rand_pct: 0.0,
            scale_x_pct: 0.0,
            scale_y_pct: 0.0,
            scale_rand_pct: 0.0,
            rotate_per_col_deg: 0.0,
            rotate_per_row_deg: 0.0,
            rotate_rand_deg: 0.0,
            opacity_delta_per_row_pct: 0.0,
            radial_radius: 0.0,
        }
    }
}
