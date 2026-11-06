import { expect, test } from '@playwright/test';
import { newPlayer, setCoins, setXp, visit } from './helpers';

test('the better tickets wait for a level', async ({ page }) => {
	await newPlayer(page);
	await visit(page, '/shop');
	await expect(page.locator('li[data-series="cash_wheel"]')).toContainText('Level 5');
	await expect(page.getByRole('button', { name: 'Buy Cash Wheel' })).toHaveCount(0);
	await expect(page.getByRole('button', { name: 'Buy Lucky Three' })).toBeEnabled();
});

test('buying a ticket puts it in the stack, ready to scratch', async ({ page }) => {
	const name = await newPlayer(page);
	await setXp(name, 1000);
	await visit(page, '/');
	await page.getByRole('link', { name: /Shop/ }).click();
	await page.getByRole('button', { name: 'Buy Cash Wheel' }).click();
	await expect(page.getByTestId('coins')).toHaveText('100');
	await expect(page.getByText('Cash Wheel is in your stack')).toBeVisible();
	// 100 coins left: Number Match (150) is out of reach
	await expect(page.getByRole('button', { name: 'Buy Number Match' })).toBeDisabled();
	await page.getByRole('button', { name: 'Scratch it now' }).click();
	await expect(page.getByRole('dialog', { name: 'Cash Wheel' })).toBeVisible();
});

test('a theme bought and worn stays on after a reload', async ({ page }) => {
	const name = await newPlayer(page);
	await setCoins(name, 2000);
	await visit(page, '/shop');
	await expect(page.locator('li[data-item="theme:neon"]')).toContainText('Golden Ticket only');
	await page.getByRole('button', { name: 'Buy Sunset' }).click();
	await expect(page.getByText('Sunset is yours')).toBeVisible();
	await expect(page.getByTestId('coins')).toHaveText('500');
	await page.getByRole('button', { name: 'Use Sunset' }).click();
	await expect(page.locator('html')).toHaveAttribute('data-skin', 'sunset');
	await expect(page.locator('li[data-item="theme:sunset"]')).toContainText('In use');
	// Rendered by the server with the theme: no flash of the default
	const html = await (await page.request.get('/shop')).text();
	expect(html).toContain('data-skin="sunset"');
});

test('a tool in use shows under the finger while scratching', async ({ page }) => {
	const name = await newPlayer(page);
	await setCoins(name, 2000);
	await visit(page, '/shop');
	await page.getByRole('button', { name: 'Buy Key' }).click();
	await page.getByRole('button', { name: 'Use Key' }).click();
	await page.getByRole('link', { name: /Lobby/ }).click();
	await page.getByRole('button', { name: 'Take' }).click();
	await page.getByRole('button', { name: 'Scratch', exact: true }).click();
	const box = (await page.locator('canvas.scratch').boundingBox())!;
	await page.mouse.move(box.x + 40, box.y + 40);
	await page.mouse.down();
	await page.mouse.move(box.x + 120, box.y + 60, { steps: 4 });
	await expect(page.locator('.pen')).toBeVisible();
	await page.mouse.up();
	await expect(page.locator('.pen')).toHaveCount(0);
});
