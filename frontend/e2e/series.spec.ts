import { expect, test, type Page } from '@playwright/test';
import { giveRaw, newPlayer, visit } from './helpers';

/** A new player with one known ticket, opened and revealed */
async function reveal(page: Page, series: string, layout: object, prize: object) {
	const name = await newPlayer(page);
	await giveRaw(name, series, layout, prize);
	await visit(page, '/');
	await page.getByRole('button', { name: 'Scratch', exact: true }).click();
	const sheet = page.getByRole('dialog');
	await sheet.getByRole('button', { name: 'Reveal all' }).click();
	return sheet;
}

const spots = (hits: number[]) =>
	[3, 9, 14, 21, 7, 28, 30, 2].map((n, i) => ({
		n: hits.includes(i) ? 12 : n,
		coins: [10, 50, 20, 100, 10, 250, 20, 50][i],
		hit: hits.includes(i)
	}));

test('Number Match pays every matching number', async ({ page }) => {
	const sheet = await reveal(
		page,
		'number_match',
		{ winning: [12, 17, 25], spots: spots([1, 3]) },
		{ kind: 'coins', amount: 150 }
	);
	await expect(sheet.getByRole('heading', { name: 'Number Match' })).toBeVisible();
	await expect(sheet.locator('[data-hit]')).toHaveCount(2);
	await expect(sheet.locator('[data-tier="small"]')).toHaveText('+150 coins!');
});

test('Ticket Chain gives every ticket it uncovers', async ({ page }) => {
	const sheet = await reveal(
		page,
		'ticket_chain',
		{ panels: [null, 'cash_wheel', null, null, 'lucky_three', null] },
		{ kind: 'tickets', series: ['cash_wheel', 'lucky_three'] }
	);
	await expect(sheet.locator('[data-tier]')).toHaveText('+2 tickets!');
	await expect(sheet.getByText('Cash Wheel · Lucky Three')).toBeVisible();
	await sheet.getByRole('button', { name: /Next ticket \(2 left\)/ }).click();
	await expect(page.getByRole('dialog', { name: 'Cash Wheel' })).toBeVisible();
});

const WHEEL = [1, 2, 1, 3, 1, 5, 2, 10, 2, 4, 3, 20];

test('Cash Wheel spins, then pays base times the stop', async ({ page }) => {
	const sheet = await reveal(
		page,
		'cash_wheel',
		{ base: 100, wheel: WHEEL, stop: 5 },
		{ kind: 'coins', amount: 500 }
	);
	// While it turns: no prize yet, no buttons
	await expect(sheet.locator('[data-tier]')).toHaveCount(0);
	await expect(sheet.getByTestId('wheel-result')).toHaveText('100 × 5', { timeout: 8000 });
	await expect(sheet.locator('[data-tier="big"]')).toContainText('+500 coins!');
	await sheet.getByRole('button', { name: 'Back to the lobby' }).click();
	await expect(page.getByTestId('coins')).toHaveText('1,000');
});

test('an empty Cash Wheel spot does not spin', async ({ page }) => {
	const sheet = await reveal(
		page,
		'cash_wheel',
		{ base: 0, wheel: WHEEL, stop: 3 },
		{ kind: 'nothing' }
	);
	await expect(sheet.locator('[data-tier="none"]')).toHaveText('Not this time');
	await expect(sheet.getByTestId('wheel-result')).toHaveCount(0);
});

test('a Golden Ticket tool lands in the shop, and a second one pays coins', async ({ page }) => {
	const prize = { kind: 'cosmetic', item: { kind: 'tool', id: 'laser' } };
	const name = await newPlayer(page);
	await giveRaw(name, 'golden', { prize }, prize);
	await giveRaw(name, 'golden', { prize }, prize);
	await visit(page, '/');
	await page.getByRole('button', { name: 'Scratch', exact: true }).click();
	const sheet = page.getByRole('dialog', { name: 'Golden Ticket' });
	await sheet.getByRole('button', { name: 'Reveal all' }).click();
	await expect(sheet.locator('[data-tier="big"]')).toHaveText('New scratch tool!');
	await expect(sheet.getByText('Laser')).toHaveCount(2);
	await sheet.getByRole('button', { name: /Next ticket/ }).click();
	await page.getByRole('dialog').getByRole('button', { name: 'Reveal all' }).click();
	await expect(
		page.getByRole('dialog').getByText('Already yours: +800 coins instead')
	).toBeVisible();
	await page.getByRole('dialog').getByRole('button', { name: 'Back to the lobby' }).click();
	await expect(page.getByTestId('coins')).toHaveText('1,300');

	await page.getByRole('link', { name: /Shop/ }).click();
	await page.getByRole('tab', { name: 'Charms & Style' }).click();
	await expect(page.getByRole('button', { name: 'Use Laser' })).toBeVisible();
});
