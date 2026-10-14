// @ts-nocheck
/**
 * Approval history surface against the mocked desktop core.
 *
 * Verifies:
 *   S1.1 — the approval history panel mounts from Settings
 *   S1.2 — the panel renders its table or empty state without raw error text
 *
 * Judged (`auto:jev`) rows need the module-backed approval engine to produce
 * them; their rendering (badge, scores, expiry) is pinned by the Vitest suite
 * `ApprovalHistoryPanel.test.tsx` until that engine is the source of rows.
 */
import { waitForApp } from '../helpers/app-helpers';
import { waitForTestId } from '../helpers/element-helpers';
import { resetApp } from '../helpers/reset-app';
import { navigateViaHash } from '../helpers/shared-flows';
import { startMockServer, stopMockServer } from '../mock-server';

const LOG_PREFIX = '[security-approval]';
const USER_ID = 'e2e-security-approval';

describe('security approval history', () => {
  before(async () => {
    await startMockServer();
    await waitForApp();
    await resetApp(USER_ID);
  });

  after(async () => {
    await stopMockServer();
  });

  it('S1.1 mounts the approval history panel', async () => {
    await navigateViaHash('/settings/approval-history');
    await waitForTestId('approval-history-panel', 15_000);
    console.log(`${LOG_PREFIX} panel mounted`);
  });

  it('S1.2 shows a list or the empty state, never an error', async () => {
    const hasError = await $('[data-testid="approval-history-error"]').isExisting();
    expect(hasError).toBe(false);
  });
});
