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

test('Pumpkin Patch counts the candy', async ({ page }) => {
	const candy = (n: number) => ({ kind: 'candy', candy: n });
	const pumpkins = [
		candy(3),
		candy(2),
		candy(2),
		candy(2),
		{ kind: 'ghost' },
		candy(2),
		candy(1),
		candy(1),
		{ kind: 'golden' }
	];
	const sheet = await open(
		page,
		'pumpkin_patch',
		{ pumpkins, candy: 13 },
		{ kind: 'coins', amount: 540 }
	);
	await expect(sheet.getByTestId('candy')).toHaveText('13 candies');
	await expect(sheet.locator('[data-pumpkin="golden"]')).toHaveCount(1);
	await expect(sheet.locator('[data-tier]')).toHaveText('+540 coins!');
});

test('Snowball rolls and grows before it pays', async ({ page }) => {
	const sheet = await open(
		page,
		'snowball',
		{ bumps: [3, 2, -1, 3, 1], sizes: [4, 6, 5, 8, 9] },
		{ kind: 'coins', amount: 150 }
	);
	await expect(sheet.locator('[data-tier]')).toHaveCount(0);
	await expect(sheet.locator('[data-tier]')).toHaveText('+150 coins!', { timeout: 8000 });
	await expect(sheet.getByTestId('snowball')).toHaveAttribute('data-size', '9');
});

test('the shop sells what is in season, with its last day', async ({ page }) => {
	const name = await newPlayer(page);
	await setCoins(name, 1000);
	await visit(page, '/shop');
	const shop = await (await page.request.get('/api/shop')).json();
	const pumpkin = shop.tickets.find((t: { series: string }) => t.series === 'pumpkin_patch');
	const snow = shop.tickets.find((t: { series: string }) => t.series === 'snowball');
	await expect(page.locator('li[data-series="pumpkin_patch"]')).toHaveCount(pumpkin ? 1 : 0);
	await expect(page.locator('li[data-series="snowball"]')).toHaveCount(snow ? 1 : 0);
	const seasonal = pumpkin ?? snow;
	if (seasonal)
		await expect(page.locator(`li[data-series="${seasonal.series}"]`)).toContainText('Until');
	await expect(page.locator('li[data-item="theme:spooky"]')).toContainText('Trophy: Pumpkin king');
});
