use crate::core::Verdict;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProviderReview {
    pub verdict: Verdict,
    pub reason: Option<String>,
    pub evidence: Vec<EvidenceItem>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EvidenceItem {
    pub path: String,
    pub signal: String,
}

pub fn parse_provider_output(input: &str) -> ProviderReview {
    let mut verdict = None;
    let mut ambiguous_verdict = false;
    let mut reason = None;
    let mut evidence = Vec::new();
    let mut in_evidence = false;

    for raw_line in input.lines() {
        let line = raw_line.trim();

        if let Some(value) = line.strip_prefix("verdict:") {
            let parsed = parse_verdict(value.trim());
            match (verdict, parsed) {
                (None, Some(next)) => verdict = Some(next),
                (Some(existing), Some(next)) if existing == next => {}
                _ => ambiguous_verdict = true,
            }
            continue;
        }

        if let Some(value) = line.strip_prefix("reason:") {
            reason = Some(value.trim().to_owned());
            continue;
        }

        if line == "evidence:" {
            in_evidence = true;
            continue;
        }

        if in_evidence {
            if let Some(value) = line.strip_prefix("- ") {
                if let Some((path, signal)) = value.split_once(':') {
                    evidence.push(EvidenceItem {
                        path: path.trim().to_owned(),
                        signal: signal.trim().to_owned(),
                    });
                }
            }
        }
    }

    ProviderReview {
        verdict: if ambiguous_verdict {
            Verdict::Ask
        } else {
            verdict.unwrap_or(Verdict::Ask)
        },
        reason,
        evidence,
    }
}

fn parse_verdict(value: &str) -> Option<Verdict> {
    match value {
        "pass" => Some(Verdict::Pass),
        "ask" => Some(Verdict::Ask),
        "block" => Some(Verdict::Block),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_provider_output() {
        let parsed = parse_provider_output(
            "verdict: block\nreason: added install script\n\nevidence:\n- package.json: added preinstall hook\n",
        );

        assert_eq!(parsed.verdict, Verdict::Block);
        assert_eq!(parsed.reason.as_deref(), Some("added install script"));
        assert_eq!(parsed.evidence.len(), 1);
        assert_eq!(parsed.evidence[0].path, "package.json");
        assert_eq!(parsed.evidence[0].signal, "added preinstall hook");
    }

    #[test]
    fn missing_verdict_returns_ask() {
        let parsed = parse_provider_output("reason: no verdict\n");

        assert_eq!(parsed.verdict, Verdict::Ask);
    }

    #[test]
    fn malformed_verdict_returns_ask() {
        let parsed = parse_provider_output("verdict: maybe\nreason: no clear decision\n");

        assert_eq!(parsed.verdict, Verdict::Ask);
    }

    #[test]
    fn conflicting_verdicts_return_ask() {
        let parsed = parse_provider_output("verdict: pass\nverdict: block\nreason: conflict\n");

        assert_eq!(parsed.verdict, Verdict::Ask);
    }
}
