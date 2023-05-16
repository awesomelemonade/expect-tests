#[derive(Debug, Clone, Copy)]
pub enum StrLitKind {
    Normal,     // use ""
    Raw(usize), // use r#""# with variable number of #'s
}

impl StrLitKind {
    pub fn write_start(&self, w: &mut impl std::fmt::Write) -> std::fmt::Result {
        match self {
            Self::Normal => write!(w, "\""),
            Self::Raw(n) => {
                write!(w, "r")?;
                for _ in 0..*n {
                    write!(w, "#")?;
                }
                write!(w, "\"")
            }
        }
    }

    pub fn write_end(&self, w: &mut impl std::fmt::Write) -> std::fmt::Result {
        match self {
            Self::Normal => write!(w, "\""),
            Self::Raw(n) => {
                write!(w, "\"")?;
                for _ in 0..*n {
                    write!(w, "#")?;
                }
                Ok(())
            }
        }
    }
}

impl From<&str> for StrLitKind {
    fn from(s: &str) -> Self {
        let has_double_quote = s.chars().any(|c| c == '"');
        if has_double_quote {
            // Find the maximum number of hashes that follow a double quote in the string.
            // We need to use one more than that to delimit the string.
            let max_hashes = s
                .split('"')
                .map(|s: &str| s.chars().take_while(|&c| c == '#').count())
                .max()
                .unwrap();
            StrLitKind::Raw(max_hashes + 1)
        } else {
            let has_backslash_or_newline = s.chars().any(|c| matches!(c, '\\' | '\n'));
            if has_backslash_or_newline {
                StrLitKind::Raw(1)
            } else {
                StrLitKind::Normal
            }
        }
    }
}
