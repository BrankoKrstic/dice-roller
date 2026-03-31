pub struct ReferenceSection {
    pub label: &'static str,
    pub title: &'static str,
    pub summary: &'static str,
    pub entries: &'static [ReferenceEntry],
}

pub struct ReferenceEntry {
    pub syntax: &'static str,
    pub meaning: &'static str,
    pub notes: &'static [&'static str],
}

const BASICS: &[ReferenceEntry] = &[
    ReferenceEntry {
        syntax: "2d6 + 3",
        meaning: "Roll dice, then add or subtract flat modifiers with normal arithmetic operators.",
        notes: &["`+`, `-`, `*`, and `/` all work in the expression editor."],
    },
    ReferenceEntry {
        syntax: "(1d8 + 4) * 2",
        meaning: "Use parentheses to control order of operations before multiplying or dividing.",
        notes: &["Parentheses are the clearest way to group damage formulas."],
    },
    ReferenceEntry {
        syntax: "-2 + d20",
        meaning: "Unary minus is supported, so penalties can be written inline instead of rephrased.",
        notes: &["The parser binds unary minus tighter than multiplication."],
    },
];

const DICE_TYPES: &[ReferenceEntry] = &[
    ReferenceEntry {
        syntax: "d4",
        meaning: "Single four-sided die.",
        notes: &[],
    },
    ReferenceEntry {
        syntax: "d6",
        meaning: "Single six-sided die.",
        notes: &[],
    },
    ReferenceEntry {
        syntax: "d8",
        meaning: "Single eight-sided die.",
        notes: &[],
    },
    ReferenceEntry {
        syntax: "d10",
        meaning: "Single ten-sided die.",
        notes: &[],
    },
    ReferenceEntry {
        syntax: "d12",
        meaning: "Single twelve-sided die.",
        notes: &[],
    },
    ReferenceEntry {
        syntax: "d20",
        meaning: "Single twenty-sided die.",
        notes: &[],
    },
    ReferenceEntry {
        syntax: "d%",
        meaning: "Percentile die.",
        notes: &["`d%` is the percentile form supported by the parser."],
    },
    ReferenceEntry {
        syntax: "4dF",
        meaning: "Fudge/Fate dice.",
        notes: &["Fudge dice roll from `-1` to `1` instead of `1..N`."],
    },
];

const SHORTCUTS: &[ReferenceEntry] = &[
    ReferenceEntry {
        syntax: "d20adv",
        meaning: "Roll advantage on a single d20.",
        notes: &["Advantage only works on a single `d20` term."],
    },
    ReferenceEntry {
        syntax: "d20dis",
        meaning: "Roll disadvantage on a single d20.",
        notes: &["Disadvantage only works on a single `d20` term."],
    },
];

const MODIFIERS: &[ReferenceEntry] = &[
    ReferenceEntry {
        syntax: "4d6kh3",
        meaning: "Keep the highest three dice.",
        notes: &["Use `kh`, `kl`, or threshold forms like `k>=12`."],
    },
    ReferenceEntry {
        syntax: "4d6dl2",
        meaning: "Drop the lowest two dice.",
        notes: &["Use `dh`, `dl`, or threshold forms like `d>=5`."],
    },
    ReferenceEntry {
        syntax: "2d6r<=2",
        meaning: "Reroll matching dice until they miss the condition.",
        notes: &["Add `times2` to cap rerolls: `2d6r<=2times2`."],
    },
    ReferenceEntry {
        syntax: "1d6ex=6times2",
        meaning: "Explode matching dice and cap the extra rolls.",
        notes: &["Without `timesN`, explosions continue until the condition stops matching."],
    },
    ReferenceEntry {
        syntax: "4d6c>=5",
        meaning: "Count how many kept dice meet a target.",
        notes: &["Bare `c` counts how many kept dice remain."],
    },
    ReferenceEntry {
        syntax: "3d6sa",
        meaning: "Sort rolls ascending for cleaner readouts.",
        notes: &["Use `s` for descending sort, `sa` for ascending."],
    },
    ReferenceEntry {
        syntax: "3d6min2max5",
        meaning: "Clamp each die between a minimum and maximum result.",
        notes: &["`minN` and `maxN` apply after rolling."],
    },
    ReferenceEntry {
        syntax: "6d6u",
        meaning: "Force all kept dice to be unique by rerolling duplicates.",
        notes: &["`u` is exclusive and cannot be combined with other modifiers."],
    },
];

const RECIPES: &[ReferenceEntry] = &[
    ReferenceEntry {
        syntax: "d20adv + 5",
        meaning: "Advantage attack roll with a flat bonus.",
        notes: &["Useful when the table already knows the modifier."],
    },
    ReferenceEntry {
        syntax: "2d6 + 1d8 + 4",
        meaning: "Combine multiple dice groups in one expression.",
        notes: &["Each term is rolled independently and then added together."],
    },
    ReferenceEntry {
        syntax: "4d6r<=3times2kh2d>=5c>=6smin2max5",
        meaning: "Stack rerolls, keep/drop logic, counting, sorting, and clamps in one expression.",
        notes: &["This matches the parser’s modifier-composition test coverage."],
    },
];

const GUARDRAILS: &[ReferenceEntry] = &[
    ReferenceEntry {
        syntax: "d20adv",
        meaning: "Advantage/disadvantage are special d20 shortcuts, not generic modifiers.",
        notes: &["`2d20adv` and `d6adv` are rejected by the parser."],
    },
    ReferenceEntry {
        syntax: "6d6u",
        meaning: "Unique rolls are valid, but `u` cannot be combined with keep/drop/reroll/explode modifiers.",
        notes: &[
            "If you need both uniqueness and other logic, split the roll into separate expressions.",
        ],
    },
    ReferenceEntry {
        syntax: "1d6ex=6times2",
        meaning: "Use `timesN` when a reroll or explosion condition could loop too long.",
        notes: &["The parser rejects impossible infinite conditions unless you cap them."],
    },
];

pub const REFERENCE_SECTIONS: &[ReferenceSection] = &[
    ReferenceSection {
        label: "Read It",
        title: "How to read an expression",
        summary: "The expression editor reads left to right like arithmetic. Write dice groups, combine them with math operators, and add parentheses when you want to force evaluation order.",
        entries: BASICS,
    },
    ReferenceSection {
        label: "Dice",
        title: "Supported dice types",
        summary: "The parser currently supports the core polyhedral dice, percentile dice, and fudge dice.",
        entries: DICE_TYPES,
    },
    ReferenceSection {
        label: "Shortcuts",
        title: "Single-d20 shortcuts",
        summary: "Advantage and disadvantage are special shortcuts, not general-purpose suffixes for every die.",
        entries: SHORTCUTS,
    },
    ReferenceSection {
        label: "Modifiers",
        title: "Modifier catalog",
        summary: "Most advanced syntax lives on a dice term. Modifiers can be stacked in one expression when the parser allows that combination.",
        entries: MODIFIERS,
    },
    ReferenceSection {
        label: "Recipes",
        title: "Combination recipes",
        summary: "These examples show how to combine arithmetic, multiple dice groups, and stacked modifiers into one command.",
        entries: RECIPES,
    },
    ReferenceSection {
        label: "Guardrails",
        title: "What to watch for",
        summary: "The editor is flexible, but a few modifiers have important limits and exclusivity rules.",
        entries: GUARDRAILS,
    },
];
