import { expect, test } from '@playwright/test';

import { browserElements } from '../../e2e/helpers/element-helpers';
import { bootAuthenticatedPage, dismissWalkthroughIfPresent } from '../helpers/core-rpc';

test('approval history shows judged decisions and revokes durable tool trust', async ({ page }) => {
  const auditRows = [
    {
      request_id: 'decision-jev',
      tool_name: 'gmail_send',
      action_summary: 'Send the approved message',
      args_redacted: {},
      session_id: 'session-e2e',
      created_at: '2026-05-29T10:00:00Z',
      decided_at: '2026-05-29T10:00:05Z',
      expires_at: '2026-05-29T10:10:00Z',
      decision: 'approve_once',
      decided_by: 'auto:jev',
      judge: {
        outcome: 'qualified',
        intent_millis: 950,
        reversible_millis: 920,
        exfiltrate_millis: 15,
        cached: false,
      },
    },
  ];
  let grants = [{ tool: 'gmail_send', request_id: 'grant-e2e', granted_at_ms: 1780048800000 }];
  let revokedId: string | undefined;

  await page.route('**/rpc', async route => {
    const request = route.request().postDataJSON();
    if (!request) {
      await route.continue();
      return;
    }

    let result: unknown;
    switch (request.method) {
      case 'openhuman.approval_list_recent_decisions':
        result = { result: auditRows, logs: [] };
        break;
      case 'openhuman.approval_list_tool_trust':
        result = { result: grants, logs: [] };
        break;
      case 'openhuman.approval_revoke_tool_trust':
        revokedId = request.params?.request_id;
        grants = grants.filter(grant => grant.request_id !== revokedId);
        result = { result: { revoked: true }, logs: [] };
        break;
      default:
        await route.continue();
        return;
    }

    await route.fulfill({ json: { jsonrpc: '2.0', id: request.id, result } });
  });

  await bootAuthenticatedPage(page, 'pw-security-approval', '/settings/approval-history');
  await dismissWalkthroughIfPresent(page);

  const elements = browserElements(page);
  await expect(elements.testId('approval-history-auto-badge')).toHaveText('Auto-approved by Jev');
  await expect(elements.testId('approval-history-judge-scores')).toContainText('Intent 95%');
  await expect(elements.testId('approval-history-judge-scores')).toContainText('Exfiltration 1.5%');
  await expect(elements.testId('approval-history-expiry')).toContainText('Grant expires');
  await expect(elements.testId('approval-tool-trust-row')).toContainText('gmail_send');

  await elements.testId('approval-tool-trust-revoke-grant-e2e').click();

  await expect.poll(() => revokedId).toBe('grant-e2e');
  await expect(elements.testId('approval-tool-trust-empty')).toBeVisible();
});
