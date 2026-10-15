use super::*;
use crate::core::runtime::DomainSet;

#[test]
fn unbound_context_has_no_core() {
    let ctx = DispatchCtx::unbound();
    assert!(ctx.core().is_none());
    assert!(ctx.into_core().is_none());
}

#[test]
fn new_context_carries_exactly_the_given_core() {
    let core = CoreContext::for_test(DomainSet::full(), None);
    let ctx = DispatchCtx::new(Arc::clone(&core));
    assert!(Arc::ptr_eq(ctx.core().expect("bound"), &core));
    let cloned = ctx.clone();
    assert!(Arc::ptr_eq(cloned.core().expect("bound"), &core));
}

#[test]
fn from_arc_binds_the_context() {
    let core = CoreContext::for_test(DomainSet::full(), None);
    let ctx: DispatchCtx = Arc::clone(&core).into();
    assert!(Arc::ptr_eq(ctx.core().expect("bound"), &core));
}

#[tokio::test]
async fn ambient_reads_the_scoped_context_and_explicit_ignores_it() {
    let scoped = CoreContext::for_test(DomainSet::full(), None);
    let other = CoreContext::for_test(DomainSet::full(), None);
    CoreContext::scope(Arc::clone(&scoped), async {
        // An explicit context is the one handed in, whatever the task scope says.
        let explicit = DispatchCtx::new(Arc::clone(&other));
        assert!(Arc::ptr_eq(explicit.core().unwrap(), &other));
        let ambient = DispatchCtx::ambient();
        assert!(Arc::ptr_eq(ambient.core().unwrap(), &scoped));
    })
    .await;
}
