//! The bounded parse of workspace source.
//!
//! syn parses with recursion, once per nesting level, and it has no depth
//! limit. A stack overflow aborts the process, so one deeply nested file can
//! end the whole run. Every parse of workspace source goes through
//! [`parse_source`]. `.clippy.toml` forbids a direct call of the syn entry
//! points everywhere else.
//!
//! [`parse_source`] runs the parse on a new thread. The stack of that thread
//! is sized from the source length. Each nesting level uses at least one
//! source byte, and syn uses a bounded quantity of stack for each level. So a
//! stack of [`STACK_PER_SOURCE_BYTE`] bytes for each source byte cannot
//! overflow. This is the same model as `parse_bounded` in fallow.
//!
//! The module uses only `std` and `syn`, so the fuzz target in `fuzz/` can
//! include it.

use std::thread;

/// Stack bytes reserved for each source byte.
///
/// A measurement with syn 3.0.9 found the most expensive shape in an unclosed
/// run of `<` or `&` in a type. It uses about 32 KiB of stack for each source
/// byte in a debug build, and about 2.9 KiB in a release build. The values
/// below add a margin. AddressSanitizer pads every frame, so the fuzz build
/// reserves 8 times the debug value.
const STACK_PER_SOURCE_BYTE: usize = if cfg!(fuzzing) {
    8 * 48 * 1024
} else if cfg!(debug_assertions) {
    48 * 1024
} else {
    4 * 1024
};

/// Stack for the frames around the parse and for the thread start.
const BASE_STACK: usize = 1 << 20;

/// The largest stack that a parse asks for.
///
/// The stack is a reservation: a parse touches only the pages that its depth
/// uses. A larger need is refused, and the source counts as too deep. In a
/// 64-bit release build, only a source over 16 MiB reaches the cap.
const MAX_STACK: usize = if cfg!(target_pointer_width = "64") {
    64 << 30
} else {
    1 << 30
};

/// The stack size is a multiple of this value. Some platforms refuse a stack
/// size that is not a multiple of the page size.
const STACK_ALIGN: usize = 64 * 1024;

/// The result of a bounded parse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseOutcome {
    /// The source parsed.
    Parsed,
    /// The parser stopped at an error, or it panicked.
    Failed {
        /// The number of parser errors. A panic counts as one error.
        error_count: u32,
    },
    /// The source was not parsed. The stack that the parse needs is larger
    /// than [`MAX_STACK`], or the thread with that stack did not start.
    TooDeep,
}

/// Parses `source` as a Rust file without a stack overflow.
///
/// A panic in the parser gives [`ParseOutcome::Failed`]. A source that needs
/// more stack than this function can get gives [`ParseOutcome::TooDeep`].
pub fn parse_source(source: &str) -> ParseOutcome {
    let Some(stack) = stack_need(source.len()) else {
        return ParseOutcome::TooDeep;
    };
    run_with_stack(stack, || parse_unbounded(source))
}

/// The stack that a parse of `source_len` bytes can use, or `None` when that
/// is more than [`MAX_STACK`].
fn stack_need(source_len: usize) -> Option<usize> {
    let need = source_len
        .checked_mul(STACK_PER_SOURCE_BYTE)?
        .checked_add(BASE_STACK)?
        .checked_next_multiple_of(STACK_ALIGN)?;
    (need <= MAX_STACK).then_some(need)
}

/// Runs `parse` on a new thread with `stack` bytes of stack.
///
/// The parsed syntax tree is also dropped on that thread, because the drop
/// recurses as deep as the parse.
fn run_with_stack(stack: usize, parse: impl FnOnce() -> ParseOutcome + Send) -> ParseOutcome {
    thread::scope(|scope| {
        let spawned = thread::Builder::new()
            .name("cargo-fallow-parse".to_owned())
            .stack_size(stack)
            .spawn_scoped(scope, parse);
        match spawned {
            // A joined panic does not reach the scope, so the run continues.
            Ok(handle) => handle
                .join()
                .unwrap_or(ParseOutcome::Failed { error_count: 1 }),
            Err(_) => ParseOutcome::TooDeep,
        }
    })
}

/// Parses `source` on the current thread. Call it only through
/// [`run_with_stack`].
fn parse_unbounded(source: &str) -> ParseOutcome {
    #[expect(
        clippy::disallowed_methods,
        reason = "the one parse site; it runs on the sized stack of `parse_source`"
    )]
    let parsed = syn::parse_file(source);
    match parsed {
        Ok(_) => ParseOutcome::Parsed,
        Err(err) => ParseOutcome::Failed {
            error_count: u32::try_from(err.into_iter().count()).unwrap_or(u32::MAX),
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_and_invalid_source() {
        assert_eq!(parse_source("fn f() {}"), ParseOutcome::Parsed);
        assert_eq!(
            parse_source("fn f( {"),
            ParseOutcome::Failed { error_count: 1 }
        );
    }

    #[test]
    fn deep_nesting_parses_on_the_sized_stack() {
        let source = format!("type T = {}u8{};", "Box<".repeat(2000), ">".repeat(2000));
        assert_eq!(parse_source(&source), ParseOutcome::Parsed);
        let unclosed = format!("type T = {}", "<".repeat(5000));
        assert_eq!(
            parse_source(&unclosed),
            ParseOutcome::Failed { error_count: 1 }
        );
    }

    #[test]
    fn need_grows_with_length_and_is_aligned() {
        assert_eq!(stack_need(0), Some(BASE_STACK));
        assert!(stack_need(1000) > stack_need(100));
        assert_eq!(stack_need(1).map(|need| need % STACK_ALIGN), Some(0));
        assert_eq!(stack_need(usize::MAX), None);
        assert_eq!(stack_need(MAX_STACK / STACK_PER_SOURCE_BYTE + 1), None);
    }

    #[test]
    fn source_over_the_cap_is_too_deep_without_a_parse() {
        let source = " ".repeat(MAX_STACK / STACK_PER_SOURCE_BYTE + 1);
        assert_eq!(parse_source(&source), ParseOutcome::TooDeep);
    }

    #[test]
    fn parser_panic_is_a_failed_parse() {
        let outcome = run_with_stack(BASE_STACK, || panic!("parser panic"));
        assert_eq!(outcome, ParseOutcome::Failed { error_count: 1 });
    }
}
