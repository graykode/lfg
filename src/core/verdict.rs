#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Pass,
    Ask,
    Block,
}

impl Verdict {
    pub const fn exit_code(self) -> i32 {
        match self {
            Self::Pass => 0,
            Self::Ask => 20,
            Self::Block => 30,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn verdicts_map_to_policy_exit_codes() {
        assert_eq!(Verdict::Pass.exit_code(), 0);
        assert_eq!(Verdict::Ask.exit_code(), 20);
        assert_eq!(Verdict::Block.exit_code(), 30);
    }
}
