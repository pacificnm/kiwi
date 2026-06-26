pub use kiwi_core::selection::SelectionPane;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct TextPosition {
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct TextSelection {
    pub pane: Option<SelectionPane>,
    pub anchor: TextPosition,
    pub cursor: TextPosition,
    pub dragging: bool,
}

impl TextSelection {
    pub fn clear(&mut self) {
        *self = Self::default();
    }

    pub fn begin(&mut self, pane: SelectionPane, pos: TextPosition) {
        self.pane = Some(pane);
        self.anchor = pos;
        self.cursor = pos;
        self.dragging = true;
    }

    pub fn extend(&mut self, pos: TextPosition) {
        if self.pane.is_some() {
            self.cursor = pos;
        }
    }

    pub fn end_drag(&mut self) {
        self.dragging = false;
    }

    pub fn has_highlight(&self) -> bool {
        if self.pane.is_none() {
            return false;
        }
        self.anchor != self.cursor
    }

    pub fn normalized(&self) -> (TextPosition, TextPosition) {
        let (a, b) = (self.anchor, self.cursor);
        if a.line < b.line || (a.line == b.line && a.col <= b.col) {
            (a, b)
        } else {
            (b, a)
        }
    }

    pub fn applies_to(&self, pane: SelectionPane) -> bool {
        self.pane == Some(pane) && self.has_highlight()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalized_orders_positions() {
        let selection = TextSelection {
            pane: Some(SelectionPane::Preview),
            anchor: TextPosition { line: 2, col: 5 },
            cursor: TextPosition { line: 1, col: 1 },
            dragging: false,
        };
        let (start, end) = selection.normalized();
        assert_eq!(start.line, 1);
        assert_eq!(end.line, 2);
    }
}
