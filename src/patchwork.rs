use std::ops::Range;

#[derive(Debug)]
struct Patch {
    deletion_range: Range<usize>,
    insertion_size: usize,
    ordering: PatchOrdering,
}

#[derive(Debug)]
pub struct Patchwork {
    text: String,
    patches: Vec<Patch>,
}

impl Patchwork {
    pub fn new(text: String) -> Patchwork {
        Patchwork {
            text,
            patches: Vec::new(),
        }
    }
    pub fn patch_insert(&mut self, offset: usize, patch: &str, ordering: PatchOrdering) {
        self.patch_range(offset..offset, patch, ordering)
    }
    // TODO: should we check that deletions are disjoint from other patches?
    pub fn patch_range(&mut self, range: Range<usize>, patch: &str, ordering: PatchOrdering) {
        let (delete, insert) = self
            .patches
            .iter()
            .take_while(|patch| {
                (patch.deletion_range.start, patch.ordering) <= (range.start, ordering)
            })
            .map(|patch| {
                (
                    patch.deletion_range.end - patch.deletion_range.start,
                    patch.insertion_size,
                )
            })
            .fold((0usize, 0usize), |(x1, y1), (x2, y2)| (x1 + x2, y1 + y2));

        let offset = insert - delete;
        self.text
            .replace_range((range.start + offset)..(range.end + offset), patch);

        self.patches.push(Patch {
            deletion_range: range,
            insertion_size: patch.len(),
            ordering,
        });
        self.patches
            .sort_by_key(|patch| (patch.deletion_range.start, patch.ordering));
    }

    pub fn text(&self) -> &str {
        self.text.as_ref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum PatchOrdering {
    BeforeOtherPatches,
    Normal,
    AfterOtherPatches,
}
