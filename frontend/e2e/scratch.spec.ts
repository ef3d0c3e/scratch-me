import { expect, test } from '@playwright/test';
import { newPlayer, scratchAll } from './helpers';

const OUTCOME = /coins!|Another ticket!|Pocket refilled!|Not this time|A bonus!/;

test('a new player takes the pocket and scratches a ticket', async ({ page }) => {
	await newPlayer(page);
	await expect(page.getByTestId('coins')).toHaveText('500');
	await expect(page.getByText('No tickets right now')).toBeVisible();
	await expect(page.getByText('3 tickets ready')).toBeVisible();

	await page.getByRole('button', { name: 'Take', exact: true }).click();
	await expect(page.getByText('3 tickets to scratch')).toBeVisible();
	await expect(page.getByText(/Next in 19:/)).toBeVisible();
	await expect(page.getByText('Up to 35 coins each, more as you level')).toBeVisible();

	await page.getByRole('button', { name: 'Scratch', exact: true }).click();
	// Level 1's pocket: the classic
	const sheet = page.getByRole('dialog', { name: 'Lucky Three' });
	await expect(sheet).toBeVisible();
	await expect(sheet.getByRole('button', { name: 'Reveal all' })).toBeEnabled();
	await scratchAll(page);

	// The foil falls off by itself once most of it is gone
	const outcome = sheet.getByText(OUTCOME);
	await expect(outcome).toBeVisible();
	// One ticket fewer, unless it won more (a ticket prize, a bonus box): the server knows
	const left: number = (await (await page.request.get('/api/me')).json()).tickets.length;
	expect([2, 3, 4]).toContain(left);
	await expect(sheet.getByRole('button', { name: `Next ticket (${left} left)` })).toBeVisible();
	await sheet.getByRole('button', { name: 'Back to the lobby' }).click();
	await expect(sheet).toBeHidden();
	await expect(page.getByText(`${left} tickets to scratch`)).toBeVisible();
});

test('the foil stays until every square is scratched', async ({ page }) => {
	await newPlayer(page);
	await page.getByRole('button', { name: 'Take', exact: true }).click();
	await page.getByRole('button', { name: 'Scratch', exact: true }).click();
	const sheet = page.getByRole('dialog');
	await expect(sheet.getByRole('button', { name: 'Reveal all' })).toBeEnabled();
	// The top two rows: two thirds of the foil, the bottom row still covered
	await scratchAll(page, 0, 0.62);
	await page.waitForTimeout(400);
	await expect(sheet.getByText(OUTCOME)).toBeHidden();
	await expect(sheet.getByRole('button', { name: 'Reveal all' })).toBeVisible();
	// The bottom row too: now it all falls off
	await scratchAll(page, 0.62, 1);
	await expect(sheet.getByText(OUTCOME)).toBeVisible();
});

test('reveal all shows the prize without scratching', async ({ page }) => {
	await newPlayer(page);
	await page.getByRole('button', { name: 'Take', exact: true }).click();
	await page.getByRole('button', { name: 'Scratch', exact: true }).click();
	const sheet = page.getByRole('dialog');
	await sheet.getByRole('button', { name: 'Reveal all' }).click();
	await expect(sheet.getByText(OUTCOME)).toBeVisible();
	// Nine symbols in the grid
	await expect(sheet.locator('.grid svg[role="img"]')).toHaveCount(9);
});

test('the daily envelope opens once', async ({ page }) => {
	await newPlayer(page);
	await expect(page.getByText('Day 1 is waiting')).toBeVisible();
	await page.getByRole('button', { name: 'Open' }).click();
	await expect(page.getByTestId('coins')).toHaveText('600');
	await expect(page.getByText(/Next one in \d+:\d\d/)).toBeVisible();
	await expect(page.getByText('1 day in a row')).toBeVisible();
	await expect(page.getByRole('button', { name: 'Open' })).toBeDisabled();
});
