use crate::plugins::traits::{ToolbarItemDescriptor, UiPlugin};

pub struct ToolUiItem {
    descriptor: ToolbarItemDescriptor,
}

impl ToolUiItem {
    pub fn new(
        id: &'static str,
        _name: &'static str,
        icon_name: &'static str,
        icon_resource: Option<&'static str>,
        tooltip: &'static str,
        order: i32,
    ) -> Self {
        Self {
            descriptor: ToolbarItemDescriptor {
                tool_id: id,
                icon_name,
                icon_resource,
                tooltip,
                group_id: None,
                order,
            },
        }
    }

    pub fn with_group(mut self, group_id: &'static str) -> Self {
        self.descriptor.group_id = Some(group_id);
        self
    }
}

impl UiPlugin for ToolUiItem {
    fn toolbar_item(&self) -> Option<ToolbarItemDescriptor> {
        Some(self.descriptor.clone())
    }
}
