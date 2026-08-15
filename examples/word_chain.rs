//! Word chains: find every sequence of `target_len` distinct dictionary words where each word
//! starts with the last letter of the previous one (e.g. `cat -> tag -> gnu`).
//!
//! Unlike the other bundled examples, this one is driven by [`parallel_solutions`] (the `parallel`
//! feature's experimental entry point) rather than [`Solutions`] alone: it runs the same search
//! both ways and prints the timings side by side, since a word chain's near-independent root
//! branches (one per starting word) are exactly the shape `parallel_solutions` is meant for. Run
//! with `cargo run --example word_chain --release --features parallel`.
use std::time::Instant;

use generic_backtracking::{parallel_solutions, Problem, Solutions};
use rayon::iter::ParallelIterator;

#[derive(Clone)]
struct WordChain {
    dictionary: Vec<String>,
    target_len: usize,
}

impl Problem for WordChain {
    type Possibility = String;
    type Solution = Vec<String>;

    fn extend_possibilities(&self, possibilities: &mut Vec<String>, history: &[String]) {
        if history.len() == self.target_len {
            return;
        }
        let last_char = history.last().and_then(|w| w.chars().last());
        possibilities.extend(
            self.dictionary
                .iter()
                .filter(|w| !history.contains(w))
                // Deliberately not `Option::is_none_or`: that would raise this package's minimum
                // supported Rust version to 1.82 for no gain.
                .filter(|w| match last_char {
                    Some(c) => w.starts_with(c),
                    None => true,
                })
                .cloned(),
        );
    }

    fn what_if(&mut self, _decision: String) {}
    fn undo(&mut self, _last: &String, _history: &[String]) {}

    fn is_solution(&self, history: &[String]) -> Option<Vec<String>> {
        (history.len() == self.target_len).then(|| history.to_vec())
    }
}

// Synthetic dictionary: every 3-letter word over a 6-letter alphabet (6^3 = 216 words).
// Self-contained (no external word-list file), but gives each starting word a large enough
// subtree to make parallelizing across roots worthwhile, while still finishing in reasonable time.
fn generate_dictionary() -> Vec<String> {
    let alphabet = ['a', 'b', 'c', 'd', 'e', 'f'];
    let mut words = Vec::new();
    for a in alphabet {
        for b in alphabet {
            for c in alphabet {
                words.push(format!("{a}{b}{c}"));
            }
        }
    }
    words
}

fn main() {
    let dictionary = generate_dictionary();
    println!("dictionary size: {}", dictionary.len());

    let target_len = 4;

    let problem = WordChain {
        dictionary: dictionary.clone(),
        target_len,
    };
    let start = Instant::now();
    let sequential_count = Solutions::new(problem).count();
    let sequential_elapsed = start.elapsed();
    println!("sequential: {sequential_count} solutions in {sequential_elapsed:?}");

    let problem = WordChain {
        dictionary,
        target_len,
    };
    let start = Instant::now();
    let parallel_count = parallel_solutions(problem).count();
    let parallel_elapsed = start.elapsed();
    println!("parallel:   {parallel_count} solutions in {parallel_elapsed:?}");

    assert_eq!(sequential_count, parallel_count);
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dictionary() -> Vec<String> {
        ["cat", "tag", "gnu", "used", "dog"]
            .into_iter()
            .map(String::from)
            .collect()
    }

    #[test]
    fn generate_dictionary_has_every_distinct_3_letter_word_over_the_alphabet() {
        let words = generate_dictionary();

        // 6-letter alphabet, 3-letter words: 6^3 combinations, all distinct.
        assert_eq!(216, words.len());
        let mut distinct = words.clone();
        distinct.sort();
        distinct.dedup();
        assert_eq!(216, distinct.len());
        assert!(words.iter().all(|w| w.len() == 3));
    }

    #[test]
    fn extend_possibilities_filters_by_last_letter() {
        let problem = WordChain {
            dictionary: dictionary(),
            target_len: 3,
        };
        let history = vec!["cat".to_string()];

        let mut possibilities = Vec::new();
        problem.extend_possibilities(&mut possibilities, &history);

        // "tag" is the only word starting with "cat"'s last letter.
        assert_eq!(vec!["tag".to_string()], possibilities);
    }

    /// Words may not repeat within a chain. `gnu -> used -> dog` leaves "gnu" as the only word
    /// starting with "dog"'s last letter, and it is already spent, so the chain is a dead end.
    /// Needs a history that actually revisits, which is why the shorter cases above cannot cover
    /// this: `dictionary`'s successor graph is `cat -> tag -> gnu` plus the 3-cycle
    /// `gnu -> used -> dog -> gnu`, so no chain shorter than four words can repeat one.
    #[test]
    fn extend_possibilities_excludes_words_already_in_the_chain() {
        let problem = WordChain {
            dictionary: dictionary(),
            target_len: 4,
        };
        let history = ["gnu", "used", "dog"].map(String::from).to_vec();

        let mut possibilities = Vec::new();
        problem.extend_possibilities(&mut possibilities, &history);

        assert!(
            possibilities.is_empty(),
            "expected a dead end, got {possibilities:?}"
        );
    }

    #[test]
    fn extend_possibilities_empty_once_target_len_reached() {
        let problem = WordChain {
            dictionary: dictionary(),
            target_len: 1,
        };
        let history = vec!["cat".to_string()];

        let mut possibilities = Vec::new();
        problem.extend_possibilities(&mut possibilities, &history);

        assert!(possibilities.is_empty());
    }

    #[test]
    fn is_solution_once_history_reaches_target_len() {
        let problem = WordChain {
            dictionary: dictionary(),
            target_len: 2,
        };

        assert_eq!(None, problem.is_solution(&["cat".to_string()]));
        assert_eq!(
            Some(vec!["cat".to_string(), "tag".to_string()]),
            problem.is_solution(&["cat".to_string(), "tag".to_string()])
        );
    }

    /// The whole point of this example is that `parallel_solutions` agrees with `Solutions` for a
    /// heap-backed (`String`, non-`Copy`) `Possibility`. `main` only cross checks the two counts;
    /// this pins the stronger property that they yield the same solutions.
    #[test]
    fn parallel_search_yields_the_same_chains_as_sequential_search() {
        let problem = WordChain {
            dictionary: dictionary(),
            target_len: 3,
        };

        let mut sequential: Vec<_> = Solutions::new(problem.clone()).collect();
        let mut parallel: Vec<_> = parallel_solutions(problem).collect();

        sequential.sort();
        parallel.sort();
        assert_eq!(sequential, parallel);
        assert!(!parallel.is_empty());
    }

    #[test]
    fn finds_every_chain_of_the_target_length() {
        let problem = WordChain {
            dictionary: dictionary(),
            target_len: 3,
        };

        let mut solutions: Vec<_> = Solutions::new(problem).collect();
        solutions.sort();

        let mut expected = vec![
            vec!["cat", "tag", "gnu"],
            vec!["tag", "gnu", "used"],
            vec!["gnu", "used", "dog"],
            vec!["used", "dog", "gnu"],
            vec!["dog", "gnu", "used"],
        ];
        expected.sort();
        assert_eq!(expected, solutions);
    }

    /// The length-3 case above cannot see the distinctness rule at all (see
    /// `extend_possibilities_excludes_words_already_in_the_chain`); at length 4 it starts pruning,
    /// cutting what would otherwise be five chains down to two. Without it the three chains that
    /// walk the `gnu -> used -> dog` cycle back onto their own first word would also be reported.
    #[test]
    fn chains_may_not_reuse_a_word() {
        let problem = WordChain {
            dictionary: dictionary(),
            target_len: 4,
        };

        let mut solutions: Vec<_> = Solutions::new(problem).collect();
        solutions.sort();

        let mut expected = vec![
            vec!["cat", "tag", "gnu", "used"],
            vec!["tag", "gnu", "used", "dog"],
        ];
        expected.sort();
        assert_eq!(expected, solutions);
    }
}
