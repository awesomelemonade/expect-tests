use std::ops::Range;

#[derive(Debug)]
pub struct Patchwork {
    text: String,
    indels: Vec<(Range<usize>, usize)>,
}

impl Patchwork {
    pub fn new(text: String) -> Patchwork {
        Patchwork {
            text,
            indels: Vec::new(),
        }
    }
    pub fn patch_insert(&mut self, offset: usize, patch: &str) {
        self.patch_range(offset..offset, patch)
    }
    pub fn patch_range(&mut self, range: Range<usize>, patch: &str) {
        let (delete, insert) = self
            .indels
            .iter()
            .take_while(|(delete, _)| delete.start < range.start)
            .map(|(delete, insert)| (delete.end - delete.start, insert))
            .fold((0usize, 0usize), |(x1, y1), (x2, y2)| (x1 + x2, y1 + y2));

        let offset = insert - delete;
        self.text
            .replace_range((range.start + offset)..(range.end + offset), patch);

        self.indels.push((range, patch.len()));
        self.indels.sort_by_key(|(delete, _insert)| delete.start);
    }

    pub fn text(&self) -> &str {
        self.text.as_ref()
    }
}
