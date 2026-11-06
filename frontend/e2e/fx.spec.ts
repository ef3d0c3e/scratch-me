import { expect, test, type Page } from '@playwright/test';
import { GRIDS, giveTicket, newPlayer, scratchAll, visit } from './helpers';

/** Records vibrations and notes played, on window.__fx */
async function listen(page: Page) {
	await page.addInitScript(() => {
		const log = { vibrate: [] as unknown[], notes: 0 };
		(window as unknown as { __fx: typeof log }).__fx = log;
		Object.defineProperty(navigator, 'vibrate', {
			value: (p: unknown) => {
				log.vibrate.push(p);
				return true;
			}
		});
		const start = OscillatorNode.prototype.start;
		OscillatorNode.prototype.start = function (...args) {
			log.notes++;
			return start.apply(this, args);
		};
	});
}

const fxLog = (page: Page) =>
	page.evaluate(() => (window as unknown as { __fx: { vibrate: unknown[]; notes: number } }).__fx);

async function jackpotTicket(page: Page) {
	const name = await newPlayer(page);
	await giveTicket(name, { kind: 'coins', amount: 1000 }, GRIDS.gem, 'gem');
	await visit(page, '/');
}

test('a jackpot plays its fanfare and buzzes', async ({ page }) => {
	await listen(page);
	await jackpotTicket(page);
	await page.getByRole('button', { name: 'Scratch', exact: true }).click();
	await page.getByRole('dialog').getByRole('button', { name: 'Reveal all' }).click();
	await expect(page.getByRole('button', { name: /JACKPOT!/ })).toBeVisible();
	const log = await fxLog(page);
	expect(log.notes).toBeGreaterThan(10);
	expect(log.vibrate).toContainEqual([60, 60, 60, 60, 60, 60, 220]);
});

test('scratching ticks under the finger', async ({ page }) => {
	await listen(page);
	await jackpotTicket(page);
	await page.getByRole('button', { name: 'Scratch', exact: true }).click();
	await scratchAll(page, 0, 0.3);
	const log = await fxLog(page);
	expect(log.vibrate.filter((v) => v === 4).length).toBeGreaterThan(0);
});

test('sound and vibration can be turned off, and stay off', async ({ page }) => {
	await listen(page);
	await jackpotTicket(page);
	await page.getByRole('button', { name: /Sound/ }).click();
	await page.getByRole('button', { name: /Vibration/ }).click();
	await visit(page, '/');
	await expect(page.getByRole('button', { name: /Sound/ })).toHaveAttribute(
		'aria-pressed',
		'false'
	);
	await expect(page.getByRole('button', { name: /Vibration/ })).toHaveAttribute(
		'aria-pressed',
		'false'
	);
	await page.getByRole('button', { name: 'Scratch', exact: true }).click();
	await scratchAll(page, 0, 0.3);
	await page.getByRole('dialog').getByRole('button', { name: 'Reveal all' }).click();
	await expect(page.getByRole('button', { name: /JACKPOT!/ })).toBeVisible();
	const log = await fxLog(page);
	expect(log.notes).toBe(0);
	expect(log.vibrate).toEqual([]);
});
