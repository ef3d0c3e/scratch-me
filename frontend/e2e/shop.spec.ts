import { expect, test } from '@playwright/test';
import { newPlayer, setCoins, setLevel, visit } from './helpers';

test('the shop opens on bundles; each tier has its tab, opening with a level', async ({ page }) => {
	await newPlayer(page);
	await visit(page, '/shop');
	await expect(page.getByRole('tab', { name: 'Bundles' })).toHaveAttribute('aria-selected', 'true');
	await expect(page.locator('li[data-bundle]').first()).toHaveAttribute('data-bundle', 'starter');
	await page.getByRole('tab', { name: 'Tier 3' }).click();
	await expect(page).toHaveURL(/tab=tier3/);
	await expect(page.getByText('Opens at level 6')).toBeVisible();
	await expect(page.locator('li[data-series="cash_wheel"]')).toContainText('Level 6');
	await expect(page.getByRole('button', { name: 'Buy Cash Wheel' })).toHaveCount(0);
	await page.getByRole('tab', { name: 'Tier 1' }).click();
	await expect(page.getByRole('button', { name: 'Buy Lucky Three' })).toBeEnabled();
	await expect(page.locator('li[data-series="pot_ticket"]')).toHaveCount(0);
});

test('a bundle puts its sealed tickets in the stack', async ({ page }) => {
	await newPlayer(page);
	await visit(page, '/shop');
	const starter = page.locator('li[data-bundle="starter"]');
	await expect(starter).toContainText('Second chance on a loss');
	await starter.getByRole('button', { name: 'Buy Starter Pack' }).click();
	await expect(page.getByText('Starter Pack is in your stack')).toBeVisible();
	await expect(page.getByTestId('coins')).toHaveText('300');
	await visit(page, '/');
	await expect(page.getByText('4 tickets to scratch')).toBeVisible();
	// The pile shows the top three; all four carry the seal
	await expect(page.locator('.stub[data-seal="sunrise"]')).toHaveCount(3);
	const me = await (await page.request.get('/api/me')).json();
	expect(me.tickets.map((t: { seal: { key: string } }) => t.seal.key)).toEqual(
		Array(4).fill('sunrise')
	);
	await page.getByRole('button', { name: 'Scratch', exact: true }).click();
	const sheet = page.getByRole('dialog');
	await expect(sheet.locator('[data-seal="sunrise"]').first()).toBeVisible();
	await expect(sheet.getByText('Bundle', { exact: true })).toBeVisible();
});

test('buying a ticket puts it in the stack, ready to scratch', async ({ page }) => {
	const name = await newPlayer(page);
	await setLevel(name, 6);
	await visit(page, '/');
	await page.getByRole('link', { name: /Shop/ }).click();
	await page.getByRole('tab', { name: 'Tier 3' }).click();
	await page.getByRole('button', { name: 'Buy Cash Wheel' }).click();
	await expect(page.getByTestId('coins')).toHaveText('100');
	await expect(page.getByText('Cash Wheel is in your stack')).toBeVisible();
	// 100 coins left: Mystery Boxes (260) is out of reach
	await expect(page.getByRole('button', { name: 'Buy Mystery Boxes' })).toBeDisabled();
	await page.getByRole('button', { name: 'Scratch it now' }).click();
	await expect(page.getByRole('dialog', { name: 'Cash Wheel' })).toBeVisible();
});

test('a theme bought and worn stays on after a reload', async ({ page }) => {
	const name = await newPlayer(page);
	await setCoins(name, 2000);
	await visit(page, '/shop?tab=style');
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
	await visit(page, '/shop?tab=style');
	await page.getByRole('button', { name: 'Buy Key' }).click();
	await page.getByRole('button', { name: 'Use Key' }).click();
	await page.getByRole('link', { name: /Lobby/ }).click();
	await page.getByRole('button', { name: 'Take', exact: true }).click();
	await page.getByRole('button', { name: 'Scratch', exact: true }).click();
	const box = (await page.locator('.play canvas.scratch').boundingBox())!;
	await page.mouse.move(box.x + 40, box.y + 40);
	await page.mouse.down();
	await page.mouse.move(box.x + 120, box.y + 60, { steps: 4 });
	await expect(page.locator('.pen')).toBeVisible();
	await page.mouse.up();
	await expect(page.locator('.pen')).toHaveCount(0);
});

test('a bought ticket, once scratched, is no longer offered to scratch', async ({ page }) => {
	await newPlayer(page);
	await visit(page, '/shop?tab=tier1');
	await page.getByRole('button', { name: 'Buy Lucky Three' }).click();
	await page.getByRole('button', { name: 'Scratch it now' }).click();
	const sheet = page.getByRole('dialog');
	await sheet.getByRole('button', { name: 'Reveal all' }).click();
	await sheet
		.getByRole('button', { name: /Back to the lobby|Next ticket/ })
		.first()
		.waitFor();
	await sheet.getByRole('button', { name: 'Back to the lobby' }).click();
	await expect(page.getByRole('button', { name: 'Scratch it now' })).toHaveCount(0);
});

test('a ticket scratched in another tab is dropped, not an error', async ({ page, context }) => {
	await newPlayer(page);
	await page.getByRole('button', { name: 'Take', exact: true }).click();
	await expect(page.getByText('3 tickets to scratch')).toBeVisible();
	// Another tab scratches the first ticket behind this one's back
	const me = await (await page.request.get('/api/me')).json();
	const other = await context.newPage();
	await other.request.post(`/api/tickets/${me.tickets[0].id}/scratch`, { data: {} });
	await other.close();
	// What is left: 2, or more when that ticket won tickets (a ticket prize, a bonus box)
	const left: number = (await (await page.request.get('/api/me')).json()).tickets.length;
	expect(left).toBeGreaterThanOrEqual(2);

	await page.getByRole('button', { name: 'Scratch', exact: true }).click();
	const sheet = page.getByRole('dialog');
	await expect(sheet.getByRole('alert')).toContainText('already scratched');
	await sheet.getByRole('button', { name: 'Back to the lobby' }).click();
	await expect(page.getByText(`${left} tickets to scratch`)).toBeVisible();
});

test('buying does not move the list: taps in a row hit the same button', async ({ page }) => {
	await newPlayer(page);
	await visit(page, '/shop?tab=tier1');
	const buy = page.getByRole('button', { name: 'Buy Lucky Three' });
	const before = (await buy.boundingBox())!;
	await buy.click();
	await expect(page.getByText('Lucky Three is in your stack')).toBeVisible();
	expect((await buy.boundingBox())!.y).toBe(before.y);
	// Three quick taps where the button was: three tickets
	for (let i = 0; i < 3; i++) {
		await page.mouse.click(before.x + before.width / 2, before.y + before.height / 2);
		await expect(page.getByTestId('coins')).toHaveText(String(500 - 35 * (i + 2)));
	}
	await visit(page, '/');
	await expect(page.getByText('4 tickets to scratch')).toBeVisible();
});
