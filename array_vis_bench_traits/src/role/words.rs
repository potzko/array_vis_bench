//! `with_words`, the one allocation of the fixed size arrays a component keeps,
//! the ones a C version holds on the stack: bit masks, counts, a table of places.
//! A role that has them (`StablePartition`, `Extraction`, `RotationMerge`) names
//! their lengths in `SCRATCH_SECTIONS` and takes the words as `&mut [usize]`, and
//! the driver of a sort allocates them here, once, so the visualiser shows them
//! and every write to them is logged.

use sort_logger::SortLogger;

/// Run `body` on the words of `sections` laid end to end, created through the
/// logger as an array a section in one row, each drawn to its own scale, and
/// freed after: the allocation of a whole sort, made once and lent to every call
/// of the component, which cuts it with `split_sections` by the same lengths.
/// Sections of no length make no events, and `body` gets an empty slice.
///
/// Zero on creation, but a component must not rely on it: the words hold what
/// the calls before left.
#[inline]
pub fn with_words<T, U, R>(logger: &mut U, sections: &[usize], body: impl FnOnce(&mut U, &mut [usize]) -> R) -> R
where
    T: Ord + Copy,
    U: ?Sized + SortLogger<T>,
{
    if sections.iter().all(|&len| len == 0) { return body(logger, &mut []); }
    let mut words = logger.create_aux_arr_split(sections);
    let ret = body(logger, &mut words);
    logger.free_aux_arr_split(&words, sections);
    ret
}

#[cfg(test)]
mod tests {
    use super::*;
    use core::marker::PhantomData;
    use sort_logger::{split_sections, SortLog, VisualizerLogger};

    fn recording() -> VisualizerLogger<usize> { VisualizerLogger { type_ghost: PhantomData, log: Vec::new() } }

    // The sections are one allocation announced as an array each, side by side,
    // and are freed once the body is done with them.
    #[test]
    fn sections_are_created_in_a_row_and_freed_after_the_body() {
        let mut log = recording();
        let mut names = Vec::new();
        with_words::<usize, _, _>(&mut log, &[3, 0, 2], |log, words| {
            assert_eq!(words.len(), 5);
            let [head, empty, tail] = split_sections(words, [3, 0, 2]);
            assert_eq!((head.len(), empty.len(), tail.len()), (3, 0, 2));
            names = vec![head.as_ptr() as usize, tail.as_ptr() as usize];
            log.write_data_u(tail, 1, 9);
        });
        let [head, tail] = [names[0], names[1]];
        assert_eq!(
            log.log,
            [
                SortLog::CreateAuxArr { name: head, length: 3 },
                SortLog::CreateAuxArr { name: tail, length: 2 },
                SortLog::Beside { name: tail, of: head },
                SortLog::WriteDataU { name: tail, ind: 1, data: 9 },
                SortLog::FreeAuxArr { name: head },
                SortLog::FreeAuxArr { name: tail },
            ]
        );
    }

    #[test]
    fn no_words_make_no_events_and_an_empty_slice() {
        for sections in [&[][..], &[0], &[0, 0]] {
            let mut log = recording();
            let got = with_words::<usize, _, _>(&mut log, sections, |_, words| words.len());
            assert_eq!((got, log.log.len()), (0, 0), "{sections:?}");
        }
    }

    #[test]
    fn the_body_result_comes_back() {
        let mut log = recording();
        assert_eq!(with_words::<usize, _, _>(&mut log, &[4], |_, words| words.iter().sum::<usize>() + 7), 7);
    }
}
