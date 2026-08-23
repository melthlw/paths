use crate::core::document::BooleanOperation;
use crate::core::{PointerButton, PointerEvent};
use crate::plugins::traits::{FeaturePlugin, PluginContext};

pub struct BooleanFeature {
    op: BooleanOperation,
}

impl BooleanFeature {
    pub fn new(op: BooleanOperation, _id: &'static str, _name: &'static str) -> Self {
        Self { op }
    }
}

impl FeaturePlugin for BooleanFeature {
    fn on_activate(&mut self, ctx: &mut PluginContext) {
        if ctx.document.selected_ids.len() >= 2 {
            ctx.document.apply_boolean_operation(self.op);
            ctx.request_redraw();
        }
    }

    fn on_pointer_down(&mut self, ctx: &mut PluginContext, event: &PointerEvent) {
        if event.button != Some(PointerButton::Primary) {
            return;
        }

        // Check if clicking on an element
        if let Some(id) = ctx.document.hit_test(event.world_pos) {
            if !ctx.document.selected_ids.contains(&id) {
                ctx.document.selected_ids.insert(id);
            }
            if ctx.document.selected_ids.len() >= 2 {
                ctx.document.apply_boolean_operation(self.op);
            }
            ctx.request_redraw();
        }
    }

    fn on_pointer_move(&mut self, ctx: &mut PluginContext, _event: &PointerEvent) {
        ctx.set_cursor("crosshair");
    }

    fn on_pointer_up(&mut self, _ctx: &mut PluginContext, _event: &PointerEvent) {}

    fn on_cancel(&mut self, _ctx: &mut PluginContext) {}
}
