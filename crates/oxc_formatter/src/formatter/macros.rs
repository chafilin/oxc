/// Constructs the parameters for other formatting macros.
///
/// This macro functions by taking a list of objects implementing [crate::Format]. It canonicalize the
/// arguments into a single type.
///
/// This macro produces a value of type [crate::Arguments]. This value can be passed to
/// the macros within [crate]. All other formatting macros ([`format!`](crate::format!),
/// [`write!`](crate::write!)) are proxied through this one. This macro avoids heap allocations.
///
/// You can use the [`Arguments`] value that `format_args!` returns in  `Format` contexts
/// as seen below.
///
/// ```rust
/// use biome_formatter::{SimpleFormatContext, format, format_args};
/// use biome_formatter::prelude::*;
///
/// # fn main() -> FormatResult<()> {
/// let formatted = format!(SimpleFormatContext::default(), [
///     format_args!(text("Hello World"))
/// ])?;
///
/// assert_eq!("Hello World", formatted.print()?.as_code());
/// # Ok(())
/// # }
/// ```
///
/// [`Format`]: crate::Format
/// [`Arguments`]: crate::Arguments
#[macro_export]
macro_rules! format_args {
    ($($value:expr),+ $(,)?) => {
        $crate::formatter::Arguments::new(&[
            $(
                $crate::formatter::Argument::new(&$value)
            ),+
        ])
    }
}

/// Writes formatted data into a buffer.
///
/// This macro accepts a 'buffer' and a list of format arguments. Each argument will be formatted
/// and the result will be passed to the buffer. The writer may be any value with a `write_fmt` method;
/// generally this comes from an implementation of the [crate::Buffer] trait.
///
/// # Examples
///
/// ```rust
/// use biome_formatter::prelude::*;
/// use biome_formatter::{Buffer, FormatState, SimpleFormatContext, VecBuffer, write};
///
/// # fn main() -> FormatResult<()> {
/// let mut state = FormatState::new(SimpleFormatContext::default());
/// let mut buffer = VecBuffer::new(&mut state);
/// write!(&mut buffer, [text("Hello"), space()])?;
/// write!(&mut buffer, [text("World")])?;
///
/// assert_eq!(
///     buffer.into_vec(),
///     vec![
///         FormatElement::StaticText { text: "Hello" },
///         FormatElement::Space,
///         FormatElement::StaticText { text: "World" },
///     ]
///  );
/// #  Ok(())
/// # }
/// ```
#[macro_export]
macro_rules! write {
    ($dst:expr, [$($arg:expr),+ $(,)?]) => {{
        let result = $dst.write_fmt($crate::format_args!($($arg),+));
        result
    }};
    ($dst:expr, $arg:expr) => {{
        let result = $dst.write_fmt($crate::format_args!($arg));
        result
    }}
}

/// Writes formatted data into the given buffer and prints all written elements for a quick and dirty debugging.
///
/// An example:
///
/// ```rust
/// use biome_formatter::prelude::*;
/// use biome_formatter::{FormatState, VecBuffer};
///
/// # fn main() -> FormatResult<()> {
/// let mut state = FormatState::new(SimpleFormatContext::default());
/// let mut buffer = VecBuffer::new(&mut state);
///
/// dbg_write!(buffer, [text("Hello")])?;
/// // ^-- prints: [src/main.rs:7][0] = StaticToken("Hello")
///
/// assert_eq!(buffer.into_vec(), vec![FormatElement::StaticText { text: "Hello" }]);
/// # Ok(())
/// # }
/// ```
///
/// Note that the macro is intended as debugging tool and therefore you should avoid having
/// uses of it in version control for long periods (other than in tests and similar). Format output
/// from production code is better done with `[write!]`
#[macro_export]
macro_rules! dbg_write {
    ($dst:expr, [$($arg:expr),+ $(,)?]) => {{
        use $crate::BufferExtensions;
        let mut count = 0;
        let mut inspect = $dst.inspect(|element: &FormatElement| {
            std::eprintln!(
                "[{}:{}][{}] = {element:#?}",
                std::file!(), std::line!(), count
            );
            count += 1;
        });
        let result = inspect.write_fmt($crate::format_args!($($arg),+));
        result
    }}
}

/// Creates the Format IR for a value.
///
/// The first argument `format!` receives is the [FormatContext] that specify how elements must be formatted.
/// Additional parameters passed get formatted by using their [crate::Format] implementation.
///
///
/// ## Examples
///
/// ```
/// use biome_formatter::prelude::*;
/// use biome_formatter::format;
///
/// let formatted = format!(SimpleFormatContext::default(), [text("("), text("a"), text(")")]).unwrap();
///
/// assert_eq!(
///     formatted.into_document(),
///     Document::from(vec![
///         FormatElement::StaticText { text: "(" },
///         FormatElement::StaticText { text: "a" },
///         FormatElement::StaticText { text: ")" },
///     ])
/// );
/// ```
#[macro_export]
macro_rules! format {
    ($context:expr, [$($arg:expr),+ $(,)?]) => {{
        ($crate::formatter::format($context, $crate::format_args!($($arg),+)))
    }}
}

/// Provides multiple different alternatives and the printer picks the first one that fits.
///
/// Use this as last resort because it requires that the printer must try all variants in the worst case.
/// The passed variants must be in the following order:
/// * First: The variant that takes up most space horizontally
/// * Last: The variant that takes up the least space horizontally by splitting the content over multiple lines.
///
/// ## Examples
///
/// ```
/// use biome_formatter::{Formatted, LineWidth, format, format_args, SimpleFormatOptions};
/// use biome_formatter::prelude::*;
///
/// # fn main() -> FormatResult<()> {
/// let formatted = format!(
///     SimpleFormatContext::default(),
///     [
///         text("aVeryLongIdentifier"),
///         best_fitting!(
///             // Everything fits on a single line
///             format_args!(
///                 text("("),
///                 group(&format_args![
///                     text("["),
///                         soft_block_indent(&format_args![
///                         text("1,"),
///                         soft_line_break_or_space(),
///                         text("2,"),
///                         soft_line_break_or_space(),
///                         text("3"),
///                     ]),
///                     text("]")
///                 ]),
///                 text(")")
///             ),
///
///             // Breaks after `[`, but prints all elements on a single line
///             format_args!(
///                 text("("),
///                 text("["),
///                 block_indent(&text("1, 2, 3")),
///                 text("]"),
///                 text(")"),
///             ),
///
///             // Breaks after `[` and prints each element on a single line
///             format_args!(
///                 text("("),
///                 block_indent(&format_args![
///                     text("["),
///                     block_indent(&format_args![
///                         text("1,"),
///                         hard_line_break(),
///                         text("2,"),
///                         hard_line_break(),
///                         text("3"),
///                     ]),
///                     text("]"),
///                 ]),
///                 text(")")
///             )
///         )
///     ]
/// )?;
///
/// let document = formatted.into_document();
///
/// // Takes the first variant if everything fits on a single line
/// assert_eq!(
///     "aVeryLongIdentifier([1, 2, 3])",
///     Formatted::new(document.clone(), SimpleFormatContext::default())
///         .print()?
///         .as_code()
/// );
///
/// // It takes the second if the first variant doesn't fit on a single line. The second variant
/// // has some additional line breaks to make sure inner groups don't break
/// assert_eq!(
///     "aVeryLongIdentifier([\n\t1, 2, 3\n])",
///     Formatted::new(document.clone(), SimpleFormatContext::new(SimpleFormatOptions { line_width: 21.try_into().unwrap(), ..SimpleFormatOptions::default() }))
///         .print()?
///         .as_code()
/// );
///
/// // Prints the last option as last resort
/// assert_eq!(
///     "aVeryLongIdentifier(\n\t[\n\t\t1,\n\t\t2,\n\t\t3\n\t]\n)",
///     Formatted::new(document.clone(), SimpleFormatContext::new(SimpleFormatOptions { line_width: 20.try_into().unwrap(), ..SimpleFormatOptions::default() }))
///         .print()?
///         .as_code()
/// );
/// # Ok(())
/// # }
/// ```
///
/// ### Enclosing group with `should_expand: true`
///
/// ```
/// use biome_formatter::{Formatted, LineWidth, format, format_args, SimpleFormatOptions};
/// use biome_formatter::prelude::*;
///
/// # fn main() -> FormatResult<()> {
/// let formatted = format!(
///     SimpleFormatContext::default(),
///     [
///         best_fitting!(
///             // Prints the method call on the line but breaks the array.
///             format_args!(
///                 text("expect(a).toMatch("),
///                 group(&format_args![
///                     text("["),
///                     soft_block_indent(&format_args![
///                         text("1,"),
///                         soft_line_break_or_space(),
///                         text("2,"),
///                         soft_line_break_or_space(),
///                         text("3"),
///                     ]),
///                     text("]")
///                 ]).should_expand(true),
///                 text(")")
///             ),
///
///             // Breaks after `(`
///            format_args!(
///                 text("expect(a).toMatch("),
///                 group(&soft_block_indent(
///                     &group(&format_args![
///                         text("["),
///                         soft_block_indent(&format_args![
///                             text("1,"),
///                             soft_line_break_or_space(),
///                             text("2,"),
///                             soft_line_break_or_space(),
///                             text("3"),
///                         ]),
///                         text("]")
///                     ]).should_expand(true),
///                 )).should_expand(true),
///                 text(")")
///             ),
///         )
///     ]
/// )?;
///
/// let document = formatted.into_document();
///
/// assert_eq!(
///     "expect(a).toMatch([\n\t1,\n\t2,\n\t3\n])",
///     Formatted::new(document.clone(), SimpleFormatContext::default())
///         .print()?
///         .as_code()
/// );
///
/// # Ok(())
/// # }
/// ```
///
/// The first variant fits because all its content up to the first line break fit on the line without exceeding
/// the configured print width.
///
/// ## Complexity
/// Be mindful of using this IR element as it has a considerable performance penalty:
/// * There are multiple representation for the same content. This results in increased memory usage
///   and traversal time in the printer.
/// * The worst case complexity is that the printer tires each variant. This can result in quadratic
///   complexity if used in nested structures.
///
/// ## Behavior
/// This IR is similar to Prettier's `conditionalGroup`. The printer measures each variant, except the [`MostExpanded`], in [`Flat`] mode
/// to find the first variant that fits and prints this variant in [`Flat`] mode. If no variant fits, then
/// the printer falls back to printing the [`MostExpanded`] variant in `[`Expanded`] mode.
///
/// The definition of *fits* differs to groups in that the printer only tests if it is possible to print
/// the content up to the first non-soft line break without exceeding the configured print width.
/// This definition differs from groups as that non-soft line breaks make group expand.
///
/// [crate::BestFitting] acts as a "break" boundary, meaning that it is considered to fit
///
///
/// [`Flat`]: crate::format_element::PrintMode::Flat
/// [`Expanded`]: crate::format_element::PrintMode::Expanded
/// [`MostExpanded`]: crate::format_element::BestFittingElement::most_expanded
#[macro_export]
macro_rules! best_fitting {
    ($least_expanded:expr, $($tail:expr),+ $(,)?) => {
        BestFitting::from_arguments_unchecked($crate::format_args!($least_expanded, $($tail),+))
    };
}
