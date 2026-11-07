import { expect, test } from '@playwright/test';
import { GRIDS, giveTicket, newPlayer, setCoins, setLevel, visit } from './helpers';

test('charms wait for a level', async ({ page }) => {
	await newPlayer(page);
	await visit(page, '/shop');
	const bargain = page.locator('li[data-charm="discount"]');
	await expect(bargain.locator('[data-tier="bronze"]')).toContainText('Level 2');
	await expect(bargain.locator('[data-tier="gold"]')).toContainText('Level 6');
	await expect(page.getByRole('button', { name: 'Buy Bronze Bargain charm' })).toHaveCount(0);
});

test('a bargain charm takes its share off tickets until its uses run out', async ({ page }) => {
	const name = await newPlayer(page);
	await setLevel(name, 6);
	await setCoins(name, 2000);
	await visit(page, '/shop');
	await page.getByRole('button', { name: 'Buy Bronze Bargain charm' }).click();
	await expect(page.getByText('Bronze Bargain charm is yours')).toBeVisible();
	await expect(page.getByTestId('coins')).toHaveText('1,960');
	const card = page.locator('li[data-charm="discount"]');
	await expect(card).toContainText('5 uses left');
	// One of a kind at a time
	await expect(page.getByRole('button', { name: 'Buy Gold Bargain charm' })).toBeDisabled();

	// 15 % off Lucky Three (35): 30. The Cash Wheel (400) is beyond bronze's reach.
	await expect(page.getByRole('button', { name: 'Buy Lucky Three' })).toHaveText(/35\s*🪙 30/);
	await expect(page.getByRole('button', { name: 'Buy Cash Wheel' })).toHaveText('🪙 400');
	await page.getByRole('button', { name: 'Buy Lucky Three' }).click();
	await expect(page.getByTestId('coins')).toHaveText('1,930');
	await expect(card).toContainText('4 uses left');

	// The lobby shows what is worn
	await page.getByRole('link', { name: /Lobby/ }).click();
	const worn = page.getByRole('region', { name: 'Charms on' });
	await expect(worn).toContainText('Bronze Bargain charm');
	await expect(worn).toContainText('4 uses left');
});

test('a Midas charm adds to the coins a ticket pays', async ({ page }) => {
	const name = await newPlayer(page);
	await setLevel(name, 6);
	await setCoins(name, 2000);
	await visit(page, '/shop');
	await page.getByRole('button', { name: 'Buy Gold Midas charm' }).click();
	await expect(page.getByTestId('coins')).toHaveText('400');
	await giveTicket(name, { kind: 'coins', amount: 100 }, GRIDS.coin, 'coin');
	await visit(page, '/');
	await page.getByRole('button', { name: 'Scratch', exact: true }).click();
	const sheet = page.getByRole('dialog', { name: 'Lucky Three' });
	await sheet.getByRole('button', { name: 'Reveal all' }).click();
	await expect(sheet.getByText('💰 +30 coins from your charm')).toBeVisible();
	await sheet.getByRole('button', { name: 'Back to the lobby' }).click();
	await expect(page.getByTestId('coins')).toHaveText('530');
	await expect(page.getByRole('region', { name: 'Charms on' })).toContainText('11 uses left');
});
