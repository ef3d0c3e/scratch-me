import { expect, test, type Page } from '@playwright/test';
import { newPlayer } from './helpers';

const OUTCOME = /coins!|Another ticket!|Pocket refilled!|Not this time/;

/** Rubs a finger (real touch events) over the top half of the first square */
async function rubHalfOfFirstSquare(page: Page) {
	const box = (await page.locator('canvas.scratch').boundingBox())!;
	const cdp = await page.context().newCDPSession(page);
	const touch = (type: string, x = 0, y = 0) =>
		cdp.send('Input.dispatchTouchEvent', {
			type,
			touchPoints: type === 'touchEnd' ? [] : [{ x, y, radiusX: 10, radiusY: 10, force: 1 }]
		});
	const square = box.width / 3;
	for (let y = box.y + 8; y < box.y + square / 2; y += 12) {
		await touch('touchStart', box.x + 8, y);
		for (let x = box.x + 8; x < box.x + square - 8; x += 6) {
			await touch('touchMove', x, y);
			await page.waitForTimeout(8);
		}
		await touch('touchEnd');
	}
}

async function openTicket(page: Page) {
	await newPlayer(page);
	await page.getByRole('button', { name: 'Take' }).click();
	await page.getByRole('button', { name: 'Scratch', exact: true }).click();
	await expect(page.getByRole('dialog').getByRole('button', { name: 'Reveal all' })).toBeEnabled();
}

test.describe('scratching by finger', () => {
	test.skip(({ isMobile }) => !isMobile, 'touch events on the phone project');

	test('half of one square leaves the ticket covered', async ({ page }) => {
		await openTicket(page);
		await rubHalfOfFirstSquare(page);
		await page.waitForTimeout(500);
		await expect(page.getByRole('dialog').getByText(OUTCOME)).toBeHidden();
	});

	test('a browser that blanks canvas readback changes nothing', async ({ page }) => {
		// Fingerprinting protection (Brave, Firefox, Safari) can hand back
		// transparent pixels: the foil must not take that for scratched away
		await page.addInitScript(() => {
			CanvasRenderingContext2D.prototype.getImageData = function (_x, _y, w, h) {
				return new ImageData(w, h);
			};
		});
		await openTicket(page);
		await rubHalfOfFirstSquare(page);
		await page.waitForTimeout(500);
		await expect(page.getByRole('dialog').getByText(OUTCOME)).toBeHidden();
	});
});
