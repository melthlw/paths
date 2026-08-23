use super::Document;

impl Document {
    pub fn snapshot(&mut self) {
        self.undo_stack.push((
            self.elements.clone(),
            self.selected_ids.clone(),
            self.pages.clone(),
            self.active_page_id,
            self.guides.clone(),
        ));
        self.redo_stack.clear();
        if self.undo_stack.len() > 64 {
            self.undo_stack.remove(0);
        }
    }

    pub fn undo(&mut self) -> bool {
        if let Some((prev_elements, prev_selection, prev_pages, prev_active_page, prev_guides)) =
            self.undo_stack.pop()
        {
            self.redo_stack.push((
                self.elements.clone(),
                self.selected_ids.clone(),
                self.pages.clone(),
                self.active_page_id,
                self.guides.clone(),
            ));
            self.elements = prev_elements;
            self.selected_ids = prev_selection;
            self.pages = prev_pages;
            self.active_page_id = prev_active_page;
            self.guides = prev_guides;
            true
        } else {
            false
        }
    }

    pub fn redo(&mut self) -> bool {
        if let Some((next_elements, next_selection, next_pages, next_active_page, next_guides)) =
            self.redo_stack.pop()
        {
            self.undo_stack.push((
                self.elements.clone(),
                self.selected_ids.clone(),
                self.pages.clone(),
                self.active_page_id,
                self.guides.clone(),
            ));
            self.elements = next_elements;
            self.selected_ids = next_selection;
            self.pages = next_pages;
            self.active_page_id = next_active_page;
            self.guides = next_guides;
            true
        } else {
            false
        }
    }

    pub fn can_undo(&self) -> bool {
        !self.undo_stack.is_empty()
    }

    pub fn can_redo(&self) -> bool {
        !self.redo_stack.is_empty()
    }
}
