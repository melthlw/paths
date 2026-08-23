use super::element::ElementId;

#[derive(Debug, Clone, PartialEq)]
pub struct LayerItemInfo {
    pub id: ElementId,
    pub name: String,
    pub icon_name: &'static str,
    pub element_type: &'static str,
    pub visible: bool,
    pub locked: bool,
    pub is_selected: bool,
    pub opacity: f32,
    pub z_index: usize,
    pub is_group: bool,
    pub children: Vec<LayerItemInfo>,
    pub is_clip: bool,
}
