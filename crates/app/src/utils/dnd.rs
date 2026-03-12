#[cfg(target_arch = "wasm32")]
use wasm_bindgen::prelude::*;

#[derive(Debug, Clone)]
pub struct DndReorder {
    pub from_index: usize,
    pub to_index: usize,
}

pub fn reorder<T: Clone>(items: &[T], from_index: usize, to_index: usize) -> Vec<T> {
    if from_index >= items.len() || to_index >= items.len() || from_index == to_index {
        return items.to_vec();
    }

    let mut next = items.to_vec();
    let item = next.remove(from_index);
    next.insert(to_index, item);
    next
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub fn enable_drag_and_drop() {
    // Hook for browser-side DnD wiring in a future pass.
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reorders_items() {
        let values = vec![1, 2, 3, 4];
        let reordered = reorder(&values, 1, 3);
        assert_eq!(reordered, vec![1, 3, 4, 2]);
    }
}
