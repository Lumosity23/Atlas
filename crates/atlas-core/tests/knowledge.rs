use atlas_core::{Atlas, EdgeKind, Error, Source};
use tempfile::TempDir;

fn vault() -> (TempDir, Atlas) {
    let dir = TempDir::new().unwrap();
    let atlas = Atlas::open(dir.path()).unwrap();
    (dir, atlas)
}

#[test]
fn plan_orders_prerequisites_first_and_skips_known() {
    let (_dir, mut a) = vault();
    let algebra = a.add_concept("Algèbre", "", vec![]).unwrap();
    let derivee = a.add_concept("Dérivée", "", vec![]).unwrap();
    let limite = a.add_concept("Limite", "", vec![]).unwrap();
    let taylor = a.add_concept("Taylor", "", vec![]).unwrap();

    a.link(&derivee, &limite, EdgeKind::Prerequisite).unwrap();
    a.link(&taylor, &derivee, EdgeKind::Prerequisite).unwrap();
    a.link(&limite, &algebra, EdgeKind::Prerequisite).unwrap();

    // Nothing known yet: full chain, deepest foundation first.
    assert_eq!(a.plan(&taylor).unwrap(), vec![algebra.clone(), limite.clone(), derivee.clone(), taylor.clone()]);

    // Algebra demonstrated: it drops out of the plan.
    a.add_evidence(vec![algebra.clone()], Source::Manual, "résolu sans aide", true).unwrap();
    assert_eq!(a.plan(&taylor).unwrap(), vec![limite, derivee, taylor]);
}

#[test]
fn only_verified_evidence_counts_as_known() {
    let (_dir, mut a) = vault();
    let c = a.add_concept("Bayes", "", vec![]).unwrap();

    a.add_evidence(vec![c.clone()], Source::Note { page: "p12".into() }, "lu le chapitre", false).unwrap();
    assert!(!a.known().contains(&c), "reading a note must not count as knowing it");

    a.add_evidence(
        vec![c.clone()],
        Source::Conversation { session: "s1".into() },
        "explique avec mes mots",
        true,
    )
    .unwrap();
    assert!(a.known().contains(&c));
}

#[test]
fn prerequisite_cycles_are_refused() {
    let (_dir, mut a) = vault();
    let x = a.add_concept("X", "", vec![]).unwrap();
    let y = a.add_concept("Y", "", vec![]).unwrap();
    let z = a.add_concept("Z", "", vec![]).unwrap();

    a.link(&x, &y, EdgeKind::Prerequisite).unwrap();
    a.link(&y, &z, EdgeKind::Prerequisite).unwrap();

    let err = a.link(&z, &x, EdgeKind::Prerequisite).unwrap_err();
    assert!(matches!(err, Error::PrerequisiteCycle { .. }));
    assert!(matches!(a.link(&x, &x, EdgeKind::Related), Err(Error::SelfLink(_))));

    // Non-prerequisite links may loop: they describe association, not order.
    a.link(&z, &x, EdgeKind::Related).unwrap();
}

#[test]
fn state_survives_reopening_the_vault() {
    let dir = TempDir::new().unwrap();
    let (id, ev) = {
        let mut a = Atlas::open(dir.path()).unwrap();
        let id = a.add_concept("Entropie", "désordre", vec!["physique".into()]).unwrap();
        let ev = a.add_evidence(vec![id.clone()], Source::Exercise { reference: "td3".into() }, "ok", true).unwrap();
        (id, ev)
    };

    let a = Atlas::open(dir.path()).unwrap();
    assert_eq!(a.concept(&id).unwrap().title, "Entropie");
    assert!(a.known().contains(&id));
    assert_eq!(a.evidence_for(&id)[0].id, ev);
}

#[test]
fn context_lists_prerequisites_nearest_first() {
    let (_dir, mut a) = vault();
    let base = a.add_concept("Base", "", vec![]).unwrap();
    let mid = a.add_concept("Milieu", "", vec![]).unwrap();
    let top = a.add_concept("Sommet", "", vec![]).unwrap();
    let side = a.add_concept("Voisin", "", vec![]).unwrap();

    a.link(&top, &mid, EdgeKind::Prerequisite).unwrap();
    a.link(&mid, &base, EdgeKind::Prerequisite).unwrap();
    a.link(&top, &side, EdgeKind::Related).unwrap();

    let ctx = a.context(&top).unwrap();
    assert_eq!(ctx.prerequisites, vec![mid, base]);
    assert_eq!(ctx.linked, vec![(side, EdgeKind::Related)]);
    assert!(!ctx.known);
}
