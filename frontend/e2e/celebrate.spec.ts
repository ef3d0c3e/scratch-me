import { expect, test, type Page } from '@playwright/test';
import { GRIDS, giveTicket, newPlayer, visit } from './helpers';

/** A new player with one known ticket in their stack, opened */
async function openKnown(
	page: Page,
	prize: object,
	cells: (typeof GRIDS)[keyof typeof GRIDS],
	win: string | null
) {
	const name = await newPlayer(page);
	await giveTicket(name, prize, cells, win as never);
	await visit(page, '/');
	await page.getByRole('button', { name: 'Scratch', exact: true }).click();
	const sheet = page.getByRole('dialog');
	await sheet.getByRole('button', { name: 'Reveal all' }).click();
	return sheet;
}

test('the jackpot takes over the screen, then lands in the wallet', async ({ page }) => {
	const sheet = await openKnown(page, { kind: 'coins', amount: 1000 }, GRIDS.gem, 'gem');
	const takeover = page.getByRole('button', { name: /JACKPOT!/ });
	await expect(takeover).toBeVisible();
	await expect(takeover).toContainText('+1,000');
	await expect(page.locator('canvas.celebration')).toBeAttached();
	await takeover.click();
	await expect(takeover).toBeHidden();
	await expect(sheet.locator('[data-tier="jackpot"]')).toContainText('+1,000 coins!');
	await sheet.getByRole('button', { name: 'Back to the lobby' }).click();
	await expect(page.getByTestId('coins')).toHaveText('1,500');
});

test('a big win rains coins but keeps the ticket in view', async ({ page }) => {
	const sheet = await openKnown(page, { kind: 'coins', amount: 200 }, GRIDS.coin, 'coin');
	await expect(sheet.locator('[data-tier="big"]')).toContainText('+200 coins!');
	await expect(page.locator('canvas.celebration')).toBeAttached();
	await expect(page.getByRole('button', { name: /JACKPOT!/ })).toHaveCount(0);
	await expect(sheet).toHaveClass(/shake/);
});

test('a loss gets no confetti', async ({ page }) => {
	const sheet = await openKnown(page, { kind: 'nothing' }, GRIDS.lose, null);
	await expect(sheet.locator('[data-tier="none"]')).toHaveText('Not this time');
	await expect(page.locator('canvas.celebration')).toHaveCount(0);
	await expect(sheet).not.toHaveClass(/shake/);
});
