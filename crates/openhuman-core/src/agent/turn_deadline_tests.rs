use super::*;
use std::time::Instant;

#[test]
fn parse_backstop_defaults_and_disables() {
    assert_eq!(parse_backstop(None), Some(Duration::from_secs(900)));
    assert_eq!(
        parse_backstop(Some("garbage")),
        Some(Duration::from_secs(900))
    );
    assert_eq!(
        parse_backstop(Some(" 120 ")),
        Some(Duration::from_secs(120))
    );
    assert_eq!(parse_backstop(Some("0")), None);
}

#[tokio::test]
async fn task_local_scope_is_visible_inside_and_absent_outside() {
    assert!(current().is_none());
    let deadline = TurnDeadline::starting_now(Duration::from_secs(60));
    let seen = with_turn_deadline(Some(deadline), async { current() }).await;
    assert_eq!(seen, Some(deadline));
    assert!(current().is_none());
    assert!(with_turn_deadline(None, async { current() })
        .await
        .is_none());
}

/// The grounded close gets what is left of the backstop minus the commit
/// reserve, and nothing once the reserve itself is reached (#7716).
#[test]
fn close_budget_is_backstop_remainder_minus_reserve() {
    let now = Instant::now();
    let backstop = Duration::from_secs(900);
    let at = |elapsed: u64| TurnDeadline::new(now - Duration::from_secs(elapsed), backstop);
    // The wind-down point of a default turn leaves 120s, 110s of it for the close.
    assert_eq!(close_budget(&at(780), now), Duration::from_secs(110));
    assert_eq!(close_budget(&at(889), now), Duration::from_secs(1));
    assert_eq!(close_budget(&at(890), now), Duration::ZERO);
    assert_eq!(close_budget(&at(950), now), Duration::ZERO);
}
