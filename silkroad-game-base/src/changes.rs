use std::collections::LinkedList;

pub trait ChangeTracked {
    type ChangeItem: Change;
    /// Returns any changes that have occurred in the time since the last request.
    fn changes(&mut self) -> Vec<Self::ChangeItem>;
}

/// A result for trying to merge two changes together
pub enum MergeResult<T> {
    /// Both changes remain unchanged.
    Unchanged(T, T),
    /// Same as [MergeResult::Unchanged], but also that the left change
    /// would be disrupted by the right change. I.e. if the change would be
    /// reordered, the changes would result in different outcomes.
    Incompatible(T, T),
    /// The changes could be merged together
    Merged(T),
    /// The two changes were opposite each other and thus cancelled each other out
    Cancelled,
}

pub trait Change: Sized {
    /// Try and merge the `other` element with this element.
    /// There are several valid outcomes from this merge. For one, these elements could be the direct
    /// opposite of each other and cancel out ([MergeResult::Cancelled]). In that case, the state before
    /// these two changes would be equal to the state after these two changes.
    /// Another outcome is the two elements being similar and could thus be merged ([MergeResult::Merged]).
    /// For example, if a change is a step from `1` to `2` and the next step is from `2` to `3`, they could
    /// be merged into a single step from `1` to `3`.
    /// In some cases, changes are indifferent to each other ([MergeResult::Unchanged]) where they don't
    /// influence each other. However, they could still be merged with other changes. A change that sets
    /// attribute `a` to 1 and a change that sets attribute `b` to 2 can't be merged together, but also
    /// don't block other changes from being merged into either. There are cases, however, where that would
    /// not be desirable and could cause inconsistencies. In those cases, they would be incompatible with
    /// each other ([MergeResult::Incompatible]).
    fn merge(self, other: Self) -> MergeResult<Self>;
}

pub trait ToOptimizedChange {
    /// Optimize the set of contained changes by merging them
    /// as much as possible.
    fn optimize(self) -> Self;
}

impl<T> ToOptimizedChange for Vec<T>
where
    T: Change,
{
    fn optimize(self) -> Self {
        self.into_iter()
            .fold(Vec::new(), |acc, next| merge_recursive(acc, next))
    }
}

fn merge_recursive<T: Change>(items: Vec<T>, new: T) -> Vec<T> {
    let mut to_do_stack = items;
    let mut unchanged_stack = LinkedList::new();
    unchanged_stack.push_back(new);
    'split: while let Some(item) = to_do_stack.pop() {
        let Some(next) = unchanged_stack.pop_front() else {
            unchanged_stack.push_front(item);
            break 'split;
        };
        match item.merge(next) {
            MergeResult::Unchanged(left, right) => {
                unchanged_stack.push_back(left);
                unchanged_stack.push_front(right);
            },
            MergeResult::Incompatible(left, right) => {
                unchanged_stack.push_back(left);
                unchanged_stack.push_front(right);
                break 'split;
            },
            MergeResult::Merged(res) => {
                unchanged_stack.push_front(res);
            },
            MergeResult::Cancelled => {
                continue;
            },
        }
    }

    to_do_stack.extend(unchanged_stack.into_iter().rev());
    to_do_stack
}

#[cfg(test)]
mod test {
    use super::*;

    #[derive(Eq, PartialEq, Debug)]
    struct TestChange(u8);

    impl Change for TestChange {
        fn merge(self, other: Self) -> MergeResult<Self> {
            if self.0 == other.0 {
                MergeResult::Cancelled
            } else if self.0 + other.0 >= 10 {
                MergeResult::Merged(TestChange((self.0 + other.0) % 10))
            } else if self.0 == 5 {
                MergeResult::Incompatible(self, other)
            } else {
                MergeResult::Unchanged(self, other)
            }
        }
    }

    #[test]
    fn test_keeps_changes() {
        let changes = vec![TestChange(1), TestChange(2), TestChange(3)];
        let optimized = changes.optimize();
        assert_eq!(3, optimized.len());
        assert_eq!(vec![TestChange(1), TestChange(2), TestChange(3)], optimized);
    }

    #[test]
    fn test_can_cancel() {
        let changes = vec![TestChange(1), TestChange(1)];
        let optimized = changes.optimize();
        assert_eq!(0, optimized.len());

        let changes = vec![TestChange(1), TestChange(1), TestChange(2)];
        let optimized = changes.optimize();
        assert_eq!(1, optimized.len());
        assert_eq!(vec![TestChange(2)], optimized);

        let changes = vec![TestChange(1), TestChange(2), TestChange(1)];
        let optimized = changes.optimize();
        assert_eq!(1, optimized.len());
        assert_eq!(vec![TestChange(2)], optimized);

        let changes = vec![
            TestChange(2),
            TestChange(1),
            TestChange(2),
            TestChange(1),
            TestChange(1),
            TestChange(1),
            TestChange(2),
        ];
        let optimized = changes.optimize();
        assert_eq!(1, optimized.len());
        assert_eq!(vec![TestChange(2)], optimized);
    }

    #[test]
    fn test_incompatible() {
        let changes = vec![TestChange(1), TestChange(1), TestChange(5)];
        let optimized = changes.optimize();
        assert_eq!(1, optimized.len());
        assert_eq!(vec![TestChange(5)], optimized);

        let changes = vec![TestChange(5), TestChange(1), TestChange(1)];
        let optimized = changes.optimize();
        assert_eq!(1, optimized.len());
        assert_eq!(vec![TestChange(5)], optimized);

        let changes = vec![TestChange(1), TestChange(5), TestChange(1)];
        let optimized = changes.optimize();
        assert_eq!(3, optimized.len());
        assert_eq!(vec![TestChange(1), TestChange(5), TestChange(1)], optimized);
    }

    #[test]
    fn test_merge() {
        let changes = vec![TestChange(6), TestChange(8)];
        let optimized = changes.optimize();
        assert_eq!(1, optimized.len());
        assert_eq!(vec![TestChange(4)], optimized);

        let changes = vec![TestChange(7), TestChange(6), TestChange(8)];
        let optimized = changes.optimize();
        assert_eq!(1, optimized.len());
        assert_eq!(vec![TestChange(1)], optimized);
    }
}
