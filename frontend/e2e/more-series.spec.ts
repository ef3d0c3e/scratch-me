import { expect, test, type Page } from '@playwright/test';
import { giveRaw, newPlayer, setCoins, visit } from './helpers';

async function open(page: Page, series: string, layout: object, prize: object) {
	const name = await newPlayer(page);
	await giveRaw(name, series, layout, prize);
	await visit(page, '/');
	await page.getByRole('button', { name: 'Scratch', exact: true }).click();
	const sheet = page.getByRole('dialog');
	await sheet.getByRole('button', { name: 'Reveal all' }).click();
	return sheet;
}

test('Tic-Tac-Toe pays the games X won', async ({ page }) => {
	const x = 'x';
	const o = 'o';
	const sheet = await open(
		page,
		'tic_tac_toe',
		{
			games: [
				{ board: [x, x, x, o, o, x, o, x, o], coins: 50, won: true },
				{ board: [x, o, x, x, o, o, o, x, x], coins: 500, won: false },
				{ board: [o, x, o, x, x, o, x, o, x], coins: 10, won: false }
			]
		},
		{ kind: 'coins', amount: 50 }
	);
	await expect(sheet.getByRole('heading', { name: 'Tic-Tac-Toe' })).toBeVisible();
	await expect(sheet.locator('[data-won]')).toHaveCount(1);
	await expect(sheet.locator('[data-tier]')).toHaveText('+50 coins!');
});

test('Lucky Dice names the roll', async ({ page }) => {
	const sheet = await open(
		page,
		'lucky_dice',
		{ dice: [4, 2, 3], roll: 'straight' },
		{ kind: 'coins', amount: 100 }
	);
	await expect(sheet.getByText('A straight!')).toBeVisible();
	await expect(sheet.locator('[data-die]')).toHaveCount(3);
	await expect(sheet.locator('[data-tier]')).toHaveText('+100 coins!');
});

test('Derby runs the race before the prize', async ({ page }) => {
	const sheet = await open(
		page,
		'derby',
		{
			bets: [
				{ horse: 3, coins: 250 },
				{ horse: 5, coins: 100 }
			],
			order: [3, 1, 6, 2, 5, 4]
		},
		{ kind: 'coins', amount: 250 }
	);
	// The race is on: no prize yet
	await expect(sheet.locator('[data-tier]')).toHaveCount(0);
	await expect(sheet.getByTestId('derby-winner')).toHaveText('Horse 3 wins!', { timeout: 8000 });
	await expect(sheet.locator('[data-tier]')).toHaveText('+250 coins!');
});

test('Plinko drops every ball before paying their bins', async ({ page }) => {
	const sheet = await open(
		page,
		'plinko',
		{
			balls: [
				{ path: [true, false, true, false, true, false, true, false], bin: 4 },
				{ path: [true, true, true, true, true, true, false, false], bin: 6 }
			],
			bins: [3000, 500, 150, 60, 25, 60, 150, 500, 3000]
		},
		{ kind: 'coins', amount: 175 }
	);
	await expect(sheet.getByTestId('plinko-balls')).toHaveText('2 balls');
	await expect(sheet.locator('[data-tier]')).toHaveCount(0);
	await expect(sheet.locator('[data-tier]')).toHaveText('+175 coins!', { timeout: 10_000 });
	await expect(sheet.locator('.bin.hit')).toHaveCount(2);
});

test('a foil bought and worn covers the tickets', async ({ page }) => {
	const name = await newPlayer(page);
	await setCoins(name, 3000);
	await visit(page, '/shop');
	const gold = page.locator('li[data-item="foil:gold"]');
	await gold.getByRole('button', { name: 'Buy Gold' }).click();
	await gold.getByRole('button', { name: 'Use Gold' }).click();
	await expect(gold).toContainText('In use');
	await expect(page.locator('li[data-item="tool:shovel"]')).toContainText('Trophy: Deep digger');
	await expect(page.locator('li[data-item="theme:champion"]')).toContainText('Trophy: Level 10');
	await expect(page.locator('li[data-item="foil:galaxy"]')).toContainText('Golden Ticket only');

	await page.getByRole('link', { name: /Lobby/ }).click();
	await page.getByRole('button', { name: 'Take', exact: true }).click();
	await page.getByRole('button', { name: 'Scratch', exact: true }).click();
	await page.waitForTimeout(400);
	// The foil's middle is gold, not silver
	const [r, g, b] = await page
		.locator('.play canvas.scratch')
		.evaluate((c: HTMLCanvasElement) => [
			...c.getContext('2d')!.getImageData(5, c.height / 2, 1, 1).data
		]);
	expect(r).toBeGreaterThan(b + 60);
	expect(g).toBeGreaterThan(b + 20);
});
