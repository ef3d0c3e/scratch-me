import { expect, test, type Page } from '@playwright/test';
import { giveRaw, newPlayer, visit } from './helpers';

async function open(page: Page, series: string, layout: object, prize: object) {
	const name = await newPlayer(page);
	await giveRaw(name, series, layout, prize);
	await visit(page, '/');
	await page.getByRole('button', { name: 'Scratch', exact: true }).click();
	return page.getByRole('dialog');
}

test('High Card pays every card above the dealer', async ({ page }) => {
	const c = (rank: number, suit = 'spades') => ({ rank, suit });
	const sheet = await open(
		page,
		'high_card',
		{
			dealer: c(9, 'hearts'),
			hands: [
				{ card: c(12), coins: 20 },
				{ card: c(9), coins: 500 },
				{ card: c(3), coins: 50 },
				{ card: c(14, 'diamonds'), coins: 10 }
			]
		},
		{ kind: 'coins', amount: 30 }
	);
	await expect(sheet.getByRole('heading', { name: 'High Card' })).toBeVisible();
	await sheet.getByRole('button', { name: 'Reveal all' }).click();
	// A tie (9 against 9) loses
	await expect(sheet.locator('[data-win]')).toHaveCount(2);
	await expect(sheet.locator('[data-tier]')).toHaveText('+30 coins!');
});

test('Lucky Reels light the lines that pay', async ({ page }) => {
	const sheet = await open(
		page,
		'lucky_reels',
		{
			reels: [
				['cherry', 'bell', 'lemon'],
				['cherry', 'bell', 'bar'],
				['cherry', 'lemon', 'bell']
			],
			lines: [0]
		},
		{ kind: 'coins', amount: 60 }
	);
	await sheet.getByRole('button', { name: 'Reveal all' }).click();
	await expect(sheet.locator('[data-lit]')).toHaveCount(3);
	await expect(sheet.locator('[data-tier]')).toHaveText('+60 coins!');
});

test('Bingo daubs the calls and pays the lines', async ({ page }) => {
	const card = [
		[1, 16, 31, 46, 61],
		[2, 17, 32, 47, 62],
		[3, 18, 0, 48, 63],
		[4, 19, 34, 49, 64],
		[5, 20, 35, 50, 65]
	];
	// The first column, plus a few off the card
	const calls = [1, 2, 3, 4, 5, 17, 48, 70, 71, 72, 73, 74, 75, 6, 7, 8, 9, 10, 11, 12, 13, 14];
	const sheet = await open(
		page,
		'bingo',
		{ card, calls, lines: [5] },
		{ kind: 'coins', amount: 120 }
	);
	await sheet.getByRole('button', { name: 'Reveal all' }).click();
	// 5 in the line, 17, 48, and the free middle
	await expect(sheet.locator('[data-daub]')).toHaveCount(8);
	await expect(sheet.locator('[data-tier]')).toHaveText('+120 coins!');
});

/** Scratches one spot of the beach clean */
async function digAt(page: Page, cell: number) {
	const foil = page.locator(`[data-cell="${cell}"] canvas.scratch`);
	const box = (await foil.boundingBox())!;
	for (let y = box.y + 4; y < box.y + box.height; y += 8) {
		await page.mouse.move(box.x + 2, y);
		await page.mouse.down();
		await page.mouse.move(box.x + box.width - 2, y, { steps: 3 });
		await page.mouse.up();
	}
}

test('Treasure Dig: the player chooses, the map adds a dig, the rest shows at the end', async ({
	page
}) => {
	const cells: object[] = Array.from({ length: 16 }, () => ({ kind: 'sand' }));
	cells[0] = { kind: 'chest', coins: 400 };
	cells[5] = { kind: 'map' };
	cells[15] = { kind: 'chest', coins: 3000 };
	const sheet = await open(page, 'treasure_dig', { cells }, { kind: 'nothing' });
	await expect(sheet.getByRole('button', { name: 'Reveal all' })).toHaveCount(0);
	await expect(sheet.getByTestId('digs-left')).toHaveText(/4 digs left/);

	await digAt(page, 5);
	await expect(sheet.locator('[data-cell="5"]')).toHaveAttribute('data-spot', 'map');
	await expect(sheet.getByTestId('digs-left')).toHaveText(/4 digs left/);
	await digAt(page, 0);
	await expect(sheet.locator('[data-cell="0"]')).toHaveAttribute('data-spot', 'chest');
	await expect(sheet.getByTestId('digs-left')).toHaveText(/3 digs left/);

	// Close half way: the dig waits in the stack, its spots kept
	await sheet.getByRole('button', { name: 'Close' }).click();
	await expect(page.getByText('1 ticket to scratch')).toBeVisible();
	await page.getByRole('button', { name: 'Scratch', exact: true }).click();
	await expect(sheet.locator('[data-cell="0"]')).toHaveAttribute('data-spot', 'chest');

	// One at a time: each dig answered (its spot shown) before the next, as a player would
	for (const cell of [1, 2, 3]) {
		await digAt(page, cell);
		await expect(sheet.locator(`[data-cell="${cell}"]`)).toHaveAttribute('data-spot', /.+/);
	}
	await expect(sheet.locator('[data-tier]')).toHaveText('+400 coins!');
	await expect(sheet.getByTestId('digs-left')).toHaveText('All dug! Here is what was hidden');
	// The 3,000 chest missed, now in view
	await expect(sheet.locator('[data-cell="15"]')).toHaveAttribute('data-spot', 'chest');
	await sheet.getByRole('button', { name: 'Back to the lobby' }).click();
	await expect(page.getByTestId('coins')).toHaveText('900');
});
