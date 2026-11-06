import { expect, test, type Page } from '@playwright/test';
import { newPlayer, scratchAll } from './helpers';

/** The shine layer as an image, to compare before and after */
const sheen = (page: Page) =>
	page.locator('canvas.sheen').evaluate((c: HTMLCanvasElement) => c.toDataURL());

/** Alpha of the shine at a point of the foil, 0–1 from its left/top */
const sheenAlpha = (page: Page, fx: number, fy: number) =>
	page
		.locator('canvas.sheen')
		.evaluate(
			(c: HTMLCanvasElement, [fx, fy]) =>
				c.getContext('2d')!.getImageData(Math.floor(c.width * fx), Math.floor(c.height * fy), 1, 1)
					.data[3],
			[fx, fy]
		);

async function openTicket(page: Page) {
	// No drift: the shine only moves when the player does
	await page.emulateMedia({ reducedMotion: 'reduce' });
	await newPlayer(page);
	await page.getByRole('button', { name: 'Take' }).click();
	await page.getByRole('button', { name: 'Scratch', exact: true }).click();
	await expect(page.getByRole('dialog').getByRole('button', { name: 'Reveal all' })).toBeEnabled();
	await page.waitForTimeout(400);
}

async function tilt(page: Page, beta: number, gamma: number) {
	await page.evaluate(
		([beta, gamma]) =>
			dispatchEvent(new DeviceOrientationEvent('deviceorientation', { beta, gamma, alpha: 0 })),
		[beta, gamma]
	);
}

test('tilting the phone moves the shine', async ({ page }) => {
	await openTicket(page);
	await tilt(page, 40, 0);
	await page.waitForTimeout(600);
	const held = await sheen(page);
	await tilt(page, 40, 20);
	await page.waitForTimeout(600);
	expect(await sheen(page)).not.toBe(held);
});

test('the shine stays still without input when motion is reduced', async ({ page }) => {
	await openTicket(page);
	const a = await sheen(page);
	await page.waitForTimeout(700);
	expect(await sheen(page)).toBe(a);
});

test('the mouse moves the shine on a computer', async ({ page, isMobile }) => {
	test.skip(isMobile, 'mouse hover on desktop');
	await openTicket(page);
	const box = (await page.locator('canvas.scratch').boundingBox())!;
	await page.mouse.move(box.x + 10, box.y + 10);
	await page.waitForTimeout(600);
	const corner = await sheen(page);
	await page.mouse.move(box.x + box.width - 10, box.y + box.height - 10);
	await page.waitForTimeout(600);
	expect(await sheen(page)).not.toBe(corner);
});

test('no shine where the foil is scratched off', async ({ page }) => {
	await openTicket(page);
	// The shine's centre band sits mid-foil with no input
	const before = await sheenAlpha(page, 0.5, 0.5);
	expect(before).toBeGreaterThan(100);
	await scratchAll(page, 0.3, 0.7);
	await page.waitForTimeout(300);
	// Gone, but for anti-aliasing at the edge of a stroke
	expect(await sheenAlpha(page, 0.5, 0.5)).toBeLessThan(before / 8);
});
