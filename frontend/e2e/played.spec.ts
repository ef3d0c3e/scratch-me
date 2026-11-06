import { expect, test, type Page } from '@playwright/test';
import { GRIDS, giveRaw, giveTicket, newPlayer, sql, visit } from './helpers';

async function open(page: Page, series: string, layout: object) {
	const name = await newPlayer(page);
	await giveRaw(name, series, layout, { kind: 'nothing' });
	await visit(page, '/');
	await page.getByRole('button', { name: 'Scratch', exact: true }).click();
	const sheet = page.getByRole('dialog');
	await expect(sheet.getByRole('button', { name: 'Reveal all' })).toHaveCount(0);
	return sheet;
}

/** Scratches a small foil clean */
async function scratch(page: Page, selector: string) {
	const box = (await page.locator(`${selector} canvas.scratch`).boundingBox())!;
	for (let y = box.y + 3; y < box.y + box.height; y += 7) {
		await page.mouse.move(box.x + 2, y);
		await page.mouse.down();
		await page.mouse.move(box.x + box.width - 2, y, { steps: 3 });
		await page.mouse.up();
	}
}

test('Push Your Luck: flip, grow the pot, cash out', async ({ page }) => {
	const panels: object[] = Array.from({ length: 12 }, () => ({ kind: 'coins', coins: 20 }));
	panels[0] = panels[1] = panels[2] = { kind: 'bomb' };
	const sheet = await open(page, 'push_luck', { panels });
	await expect(sheet.getByRole('button', { name: 'Cash out 0' })).toBeDisabled();
	await scratch(page, '[data-panel="5"]');
	await expect(sheet.getByTestId('pot')).toContainText('Pot: 20');
	await scratch(page, '[data-panel="6"]');
	await expect(sheet.getByTestId('pot')).toContainText('Pot: 40');
	await sheet.getByRole('button', { name: 'Cash out 40' }).click();
	await expect(sheet.locator('[data-tier]')).toHaveText('+40 coins!');
	// The bombs show once it is over
	await expect(sheet.locator('[data-panel="0"]')).toHaveAttribute('data-is', 'bomb');
});

test('Safe Cracker: hints, then the code', async ({ page }) => {
	const sheet = await open(page, 'safe_cracker', { code: [3, 7, 0] });
	await sheet.getByRole('button', { name: 'Try this code (5 tries left)' }).click();
	await expect(sheet.locator('[data-try="0"]')).toContainText('0▲0▲0✓');
	for (let i = 0; i < 3; i++) await sheet.getByRole('button', { name: 'Dial 1 up' }).click();
	for (let i = 0; i < 3; i++) await sheet.getByRole('button', { name: 'Dial 2 down' }).click();
	await expect(sheet.getByTestId('dial-1')).toHaveText('7');
	await sheet.getByRole('button', { name: /Try this code/ }).click();
	await expect(sheet.getByText('Open!')).toBeVisible();
	await expect(sheet.locator('[data-tier]')).toHaveText('+1,000 coins!');
});

test('Mystery Boxes: the host opens an empty box, switching finds the prize', async ({ page }) => {
	const sheet = await open(page, 'mystery_boxes', { prize_box: 2, coins: 600, host: 0 });
	await expect(sheet.getByTestId('boxes-step')).toHaveText('Pick a box');
	await sheet.getByRole('button', { name: 'Box 1' }).click();
	await expect(sheet.locator('[data-box="1"]')).toContainText('Empty');
	await sheet.getByRole('button', { name: 'Switch' }).click();
	await expect(sheet.getByTestId('boxes-step')).toHaveText('You found the prize!');
	await expect(sheet.locator('[data-tier]')).toHaveText('+600 coins!');
});

test('Defuse the Bomb: the clock runs, a dud, then the right wire', async ({ page }) => {
	const sheet = await open(page, 'defuse_bomb', {
		wires: ['dud', 'boom', 'dud', 'defuse', 'dud', 'dud'],
		coins: 400
	});
	const clock = sheet.getByTestId('clock');
	await expect(clock).toHaveText(/^1\d\.\d$/);
	const first = Number(await clock.textContent());
	await page.waitForTimeout(1200);
	expect(Number(await clock.textContent())).toBeLessThan(first);
	await sheet.getByRole('button', { name: 'Cut wire 1' }).click();
	await expect(sheet.locator('[data-wire="0"]')).toHaveAttribute('data-is', 'dud');
	await sheet.getByRole('button', { name: 'Cut wire 4' }).click();
	await expect(sheet.locator('[data-tier]')).toHaveText('+400 coins!');
	await expect(sheet.locator('[data-wire="1"]')).toHaveAttribute('data-is', 'boom');
});

test('Rocket: launch, climb, cash out what the server measured', async ({ page }) => {
	const sheet = await open(page, 'rocket', { crash: 40.0 });
	await sheet.getByRole('button', { name: 'Launch' }).click();
	await page.waitForTimeout(2000);
	await expect(sheet.getByTestId('multiplier')).not.toHaveText('×1.00');
	await sheet.getByRole('button', { name: 'Cash out' }).click();
	await expect(sheet.getByText(/Cashed out at ×1\.\d\d/)).toBeVisible();
	await expect(sheet.locator('[data-tier]')).toHaveText(/^\+1\d\d coins!$/);
});

test('Rocket: a crash comes to the screen by itself', async ({ page }) => {
	const sheet = await open(page, 'rocket', { crash: 1.3 });
	await sheet.getByRole('button', { name: 'Launch' }).click();
	await expect(sheet.getByText('Crashed at ×1.30')).toBeVisible({ timeout: 6000 });
	await expect(sheet.locator('[data-tier="none"]')).toBeVisible();
});

test('a bonus box doubles the coins', async ({ page }) => {
	const name = await newPlayer(page);
	await giveTicket(name, { kind: 'coins', amount: 400 }, GRIDS.coin, 'coin');
	// The layout carries a ×2 bonus box; the prize already counts it
	await sql(
		`UPDATE tickets SET layout = layout || '{"bonus": {"kind": "times", "times": 2}}'
		 WHERE user_id = (SELECT id FROM users WHERE username = $1)`,
		[name]
	);
	await visit(page, '/');
	await page.getByRole('button', { name: 'Scratch', exact: true }).click();
	const sheet = page.getByRole('dialog');
	// There, under its own foil, until the ticket is uncovered
	const box = sheet.locator('[data-bonus="times"]');
	await expect(box).not.toHaveClass(/open/);
	await sheet.getByRole('button', { name: 'Reveal all' }).click();
	await expect(box).toHaveClass(/open/);
	await expect(box).toContainText('×2');
	await expect(sheet.locator('[data-tier]')).toHaveText('+400 coins!');
});
