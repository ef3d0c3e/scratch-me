import { expect, test, type Locator, type Page } from '@playwright/test';
import { giveRaw, newPlayer, visit } from './helpers';

/** Opens a ticket with a known layout, played by choice: no reveal-all */
async function open(page: Page, series: string, layout: object) {
	const name = await newPlayer(page);
	await giveRaw(name, series, layout, { kind: 'nothing' });
	await visit(page, '/');
	await page.getByRole('button', { name: 'Scratch', exact: true }).click();
	const sheet = page.getByRole('dialog');
	await expect(sheet.getByRole('button', { name: 'Reveal all' })).toHaveCount(0);
	return sheet;
}

/** Scratches one little foil all over, like a finger */
async function scratch(page: Page, where: Locator) {
	const box = (await where.locator('canvas.scratch').boundingBox())!;
	for (let y = box.y + 4; y < box.y + box.height; y += 8) {
		await page.mouse.move(box.x + 2, y);
		await page.mouse.down();
		await page.mouse.move(box.x + box.width - 2, y, { steps: 3 });
		await page.mouse.up();
	}
}

test('Coin Flip: the scratched side is the call, then the coin flips', async ({ page }) => {
	const sheet = await open(page, 'coin_flip', { bets: [50, 250], flip: 'tails' });
	await expect(sheet.getByTestId('coin-step')).toHaveText('Scratch the side you bet on');
	await scratch(page, sheet.locator('[data-side="tails"]'));
	await expect(sheet.locator('[data-side="tails"]')).toContainText('250');
	await expect(sheet.getByTestId('coin-step')).toHaveText('Tails! You win your bet', {
		timeout: 10_000
	});
	await expect(sheet.locator('[data-tier]')).toHaveText('+250 coins!');
	// The other side's bet shows too, to see what it would have been
	await expect(sheet.locator('[data-side="heads"]')).toContainText('50');
});

test('Roulette: the scratched colour is the bet, then the wheel spins', async ({ page }) => {
	const sheet = await open(page, 'roulette', { bets: [140, 170, 2500], pocket: 0 });
	await scratch(page, sheet.locator('[data-colour="red"]'));
	await expect(sheet.getByTestId('roulette-step')).toHaveText('0. Not this time', {
		timeout: 10_000
	});
	await expect(sheet.locator('[data-pocket="0"]')).toHaveCount(1);
	await expect(sheet.locator('[data-colour="green"]')).toContainText('2,500');
	await expect(sheet.getByText('Not this time', { exact: true })).toBeVisible();
});

test('Battleship: each cell is a shot, sunk ships show whole', async ({ page }) => {
	const sheet = await open(page, 'battleship', {
		ships: [
			{ cells: [0, 1, 2], coins: 15 },
			{ cells: [10, 15], coins: 5 },
			{ cells: [23, 24], coins: 5 }
		]
	});
	await expect(sheet.getByTestId('shots-left')).toHaveText(/10 shots left/);
	for (const cell of [0, 1, 2]) {
		await scratch(page, sheet.locator(`[data-cell="${cell}"]`));
		await expect(sheet.locator(`[data-cell="${cell}"]`)).toHaveAttribute('data-state', 'hit');
	}
	await expect(sheet.getByTestId('shots-left')).toHaveText(/7 shots left · 🪙 60/);
	await scratch(page, sheet.locator('[data-cell="12"]'));
	await expect(sheet.locator('[data-cell="12"]')).toHaveAttribute('data-state', 'miss');
	// The sunk ship shows; the others stay hidden
	await expect(sheet.locator('.cell.ship')).toHaveCount(3);
});
