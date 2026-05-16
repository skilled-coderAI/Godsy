use std::collections::HashSet;

use crate::error::{CoreError, Result};
use crate::plan::Plan;

#[derive(Debug)]
pub struct VerificationReport {
    pub citation_ids_used: HashSet<String>,
    pub task_ids: HashSet<String>,
    pub component_ids: HashSet<String>,
    pub issues: Vec<String>,
}

impl VerificationReport {
    pub fn ok(&self) -> bool {
        self.issues.is_empty()
    }
}

/// Mechanical structural verification of a plan — Layer 3 of `prd.md` §10.
/// Confirms:
///   - every citation referenced by id actually exists
///   - every task's component_refs resolve to a real component
///   - every task's depends_on resolves to an earlier task in order
///   - no duplicate ids in tasks, components, citations, entities
pub fn verify_structure(plan: &Plan) -> Result<VerificationReport> {
    let mut report = VerificationReport {
        citation_ids_used: HashSet::new(),
        task_ids: HashSet::new(),
        component_ids: HashSet::new(),
        issues: Vec::new(),
    };

    let citation_ids: HashSet<&str> = plan.citations.iter().map(|c| c.id.as_str()).collect();
    if citation_ids.len() != plan.citations.len() {
        report.issues.push("duplicate citation ids".to_string());
    }

    let component_ids: HashSet<&str> =
        plan.architecture.components.iter().map(|c| c.id.as_str()).collect();
    if component_ids.len() != plan.architecture.components.len() {
        report.issues.push("duplicate component ids".to_string());
    }
    report.component_ids = component_ids.iter().map(ToString::to_string).collect();

    let entity_ids: HashSet<&str> =
        plan.data_model.entities.iter().map(|e| e.id.as_str()).collect();
    if entity_ids.len() != plan.data_model.entities.len() {
        report.issues.push("duplicate entity ids".to_string());
    }

    let task_ids: HashSet<&str> = plan.tasks.iter().map(|t| t.id.as_str()).collect();
    if task_ids.len() != plan.tasks.len() {
        report.issues.push("duplicate task ids".to_string());
    }
    report.task_ids = task_ids.iter().map(ToString::to_string).collect();

    for cid in &plan.stack.citation_ids {
        if !citation_ids.contains(cid.as_str()) {
            report.issues.push(format!("stack references missing citation: {cid}"));
        } else {
            report.citation_ids_used.insert(cid.clone());
        }
    }

    let mut tasks_sorted = plan.tasks.clone();
    tasks_sorted.sort_by_key(|t| t.order);
    let mut seen_so_far: HashSet<String> = HashSet::new();
    for t in &tasks_sorted {
        for cref in &t.component_refs {
            if !component_ids.contains(cref.as_str()) {
                report.issues.push(format!("task {} references missing component: {cref}", t.id));
            }
        }
        for dep in &t.depends_on {
            if !task_ids.contains(dep.as_str()) {
                report.issues.push(format!("task {} depends on missing task: {dep}", t.id));
            } else if !seen_so_far.contains(dep) {
                report.issues.push(format!(
                    "task {} depends on later task: {dep} (forward dependency)",
                    t.id
                ));
            }
        }
        seen_so_far.insert(t.id.clone());
    }

    for sec in &plan.confidence.sections {
        for cid in &sec.citation_ids {
            if !citation_ids.contains(cid.as_str()) {
                report.issues.push(format!(
                    "confidence section '{}' references missing citation: {cid}",
                    sec.section
                ));
            } else {
                report.citation_ids_used.insert(cid.clone());
            }
        }
    }

    if report.issues.is_empty() {
        Ok(report)
    } else {
        Err(CoreError::Validation(report.issues.join("; ")))
    }
}

/// Layer 1 of `prd.md` §10 — mechanical Layer-1 citation resolution.
///
/// Enforces:
///   - every `Citation` has a non-empty id
///   - `Web` citations carry a non-empty `url`
///   - `KnowledgeBase` citations carry a non-empty `chunk_id`
///   - `ProjectFile` citations carry a non-empty `path`
///   - no two citations share an id
///   - the `stack.citation_ids` and every `confidence.sections[*].citation_ids`
///     resolve to a real `Citation` (also enforced by `verify_structure`,
///     surfaced here for callers that only want the Layer-1 view).
pub fn verify_citations(plan: &Plan) -> Result<()> {
    use crate::citation::CitationKind;

    let mut issues: Vec<String> = Vec::new();
    let mut seen_ids: HashSet<&str> = HashSet::new();

    for c in &plan.citations {
        if c.id.is_empty() {
            issues.push("citation has empty id".to_string());
            continue;
        }
        if !seen_ids.insert(c.id.as_str()) {
            issues.push(format!("duplicate citation id: {}", c.id));
        }
        match &c.source {
            CitationKind::Web { url } if url.is_empty() => {
                issues.push(format!("citation {} is Web but has empty url", c.id));
            }
            CitationKind::KnowledgeBase { chunk_id } if chunk_id.is_empty() => {
                issues.push(format!("citation {} is KnowledgeBase but has empty chunk_id", c.id));
            }
            CitationKind::ProjectFile { path, .. } if path.is_empty() => {
                issues.push(format!("citation {} is ProjectFile but has empty path", c.id));
            }
            _ => {}
        }
    }

    let known: HashSet<&str> = plan.citations.iter().map(|c| c.id.as_str()).collect();
    for cid in &plan.stack.citation_ids {
        if !known.contains(cid.as_str()) {
            issues.push(format!("stack.citation_ids: missing citation {cid}"));
        }
    }
    for sec in &plan.confidence.sections {
        for cid in &sec.citation_ids {
            if !known.contains(cid.as_str()) {
                issues.push(format!(
                    "confidence.sections[{}].citation_ids: missing citation {cid}",
                    sec.section
                ));
            }
        }
    }

    if issues.is_empty() {
        Ok(())
    } else {
        Err(CoreError::MissingCitation(issues.join("; ")))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::plan::{Component, Plan};
    use crate::task::{Complexity, Task};

    fn base_plan() -> Plan {
        let mut p = Plan::new("test");
        p.architecture.components.push(Component {
            id: "c1".into(),
            name: "Web UI".into(),
            responsibility: "render".into(),
            depends_on: vec![],
        });
        p.tasks.push(Task {
            id: "t1".into(),
            order: 1,
            goal: "scaffold".into(),
            component_refs: vec!["c1".into()],
            inputs: vec![],
            outputs: vec![],
            files_touched: vec![],
            acceptance_criteria: vec!["compiles".into()],
            complexity: Complexity::S,
            depends_on: vec![],
        });
        p
    }

    #[test]
    fn accepts_well_formed_plan() {
        let p = base_plan();
        assert!(verify_structure(&p).is_ok());
    }

    #[test]
    fn rejects_dangling_component_ref() {
        let mut p = base_plan();
        p.tasks[0].component_refs = vec!["does-not-exist".into()];
        assert!(verify_structure(&p).is_err());
    }

    #[test]
    fn citations_pass_when_all_resolve() {
        use crate::citation::{Citation, CitationKind};
        let mut p = base_plan();
        p.citations.push(Citation::new(
            "cit1",
            CitationKind::Web { url: "https://x".into() },
            "snippet",
        ));
        p.stack.citation_ids = vec!["cit1".into()];
        assert!(verify_citations(&p).is_ok());
    }

    #[test]
    fn citations_reject_dangling_stack_ref() {
        let mut p = base_plan();
        p.stack.citation_ids = vec!["nope".into()];
        assert!(verify_citations(&p).is_err());
    }

    #[test]
    fn citations_reject_empty_web_url() {
        use crate::citation::{Citation, CitationKind};
        let mut p = base_plan();
        p.citations.push(Citation::new("c1", CitationKind::Web { url: String::new() }, "x"));
        assert!(verify_citations(&p).is_err());
    }

    #[test]
    fn rejects_forward_dependency() {
        let mut p = base_plan();
        p.tasks.push(Task {
            id: "t2".into(),
            order: 2,
            goal: "next".into(),
            component_refs: vec!["c1".into()],
            inputs: vec![],
            outputs: vec![],
            files_touched: vec![],
            acceptance_criteria: vec!["x".into()],
            complexity: Complexity::S,
            depends_on: vec!["t3".into()],
        });
        assert!(verify_structure(&p).is_err());
    }
}
