import { expect, test, type Page } from '@playwright/test';
import { giveRaw, newPlayer, visit } from './helpers';

async function open(page: Page, series: string, layout: object, prize: object) {
	const name = await newPlayer(page);
	await giveRaw(name, series, layout, prize);
	await visit(page, '/');
	await page.getByRole('button', { name: 'Scratch', exact: true }).click();
	const sheet = page.getByRole('dialog');
	await sheet.getByRole('button', { name: 'Reveal all' }).click();
	return sheet;
}
const c = (rank: number) => ({ rank, suit: 'spades' });

test('Blackjack judges each hand against the dealer', async ({ page }) => {
	const sheet = await open(
		page,
		'blackjack',
		{
			dealer: [c(10), c(8)],
			hands: [
				{ cards: [c(14), c(13)], coins: 20, result: 'blackjack' },
				{ cards: [c(10), c(8)], coins: 50, result: 'push' },
				{ cards: [c(10), c(5), c(9)], coins: 100, result: 'lose' }
			]
		},
		{ kind: 'coins', amount: 40 }
	);
	await expect(sheet.locator('[data-result="blackjack"]')).toHaveCount(1);
	await expect(sheet.locator('[data-result="push"]')).toHaveCount(1);
	await expect(sheet.locator('[data-tier]')).toHaveText('+40 coins!');
});

test('Hot Spot counts the matches', async ({ page }) => {
	const mine = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
	const drawn = [1, 2, 3, 4, 50, 51, 52, 53, 54, 55, 56, 57, 58, 59, 60, 61, 62, 63, 64, 65];
	const sheet = await open(
		page,
		'hot_spot',
		{ mine, drawn, matches: 4 },
		{ kind: 'coins', amount: 80 }
	);
	await expect(sheet.locator('[data-hit]')).toHaveCount(4);
	await expect(sheet.getByTestId('matches')).toHaveText('4 matches');
	await expect(sheet.locator('[data-tier]')).toHaveText('+80 coins!');
});

test('Battleship shows hits, misses and the ship sunk', async ({ page }) => {
	const sheet = await open(
		page,
		'battleship',
		{
			ships: [
				{ cells: [0, 1, 2], coins: 400, sunk: true },
				{ cells: [10, 15], coins: 100, sunk: false },
				{ cells: [23, 24], coins: 100, sunk: false }
			],
			shots: [0, 1, 2, 10, 5, 6, 7, 8, 9, 12],
			hits: 4
		},
		{ kind: 'coins', amount: 412 }
	);
	await expect(sheet.locator('[data-state="hit"]')).toHaveCount(4);
	await expect(sheet.locator('[data-state="miss"]')).toHaveCount(6);
	await expect(sheet.locator('[data-sunk]')).toHaveCount(1);
	await expect(sheet.locator('[data-tier]')).toHaveText('+412 coins!');
});

test('Board Walk walks the token, then pays the squares', async ({ page }) => {
	const sheet = await open(
		page,
		'board_walk',
		{
			rolls: [6, 3, 6],
			path: [6, 9, 15],
			board: [0, 5, 0, 10, 0, 0, 20, 5, 0, 40, 0, 0, 10, 5, 0, 120]
		},
		{ kind: 'coins', amount: 180 }
	);
	await expect(sheet.locator('[data-tier]')).toHaveCount(0);
	await expect(sheet.locator('[data-tier]')).toHaveText('+180 coins!', { timeout: 10_000 });
	await expect(sheet.locator('[data-landed]')).toHaveCount(3);
});

test('Fishing fills the aquarium and marks new species', async ({ page }) => {
	const casts = [
		{ fish: 'tuna', coins: 20 },
		{ fish: null, coins: 0 },
		{ fish: 'octopus', coins: 100 },
		{ fish: 'sardine', coins: 5 }
	];
	const sheet = await open(page, 'fishing', { casts }, { kind: 'coins', amount: 125 });
	await expect(sheet.locator('[data-fish="boot"]')).toContainText('Old boot');
	await expect(sheet.getByText('NEW', { exact: true })).toHaveCount(3);
	await expect(sheet.getByTestId('aquarium')).toHaveText(/Aquarium: 3 of 12 species/);
	await expect(sheet.locator('[data-tier]')).toHaveText('+125 coins!');
});

test('Word Bank lights the complete words', async ({ page }) => {
	const sheet = await open(
		page,
		'word_bank',
		{
			words: ['TAXI', 'ZOO', 'PIANO', 'JUDO', 'GOLF'],
			letters: ['T', 'A', 'X', 'I', 'Z', 'O', 'P', 'N', 'B', 'C', 'E', 'H'],
			complete: [0, 1, 2]
		},
		{ kind: 'coins', amount: 400 }
	);
	await expect(sheet.locator('[data-done]')).toHaveCount(3);
	await expect(sheet.locator('[data-tier]')).toHaveText('+400 coins!');
});

test('Word Search highlights the hidden words', async ({ page }) => {
	const grid = Array.from({ length: 64 }, () => 'Q');
	'TAXI'.split('').forEach((ch, i) => (grid[i] = ch));
	const sheet = await open(
		page,
		'word_search',
		{
			grid,
			targets: [
				{ word: 'TAXI', coins: 50, at: [0, 1, 2, 3] },
				{ word: 'ZOO', coins: 300, at: null },
				{ word: 'SKI', coins: 20, at: null },
				{ word: 'GOLF', coins: 100, at: null }
			]
		},
		{ kind: 'coins', amount: 50 }
	);
	await expect(sheet.locator('[data-found]')).toHaveCount(1);
	await expect(sheet.locator('.grid .lit')).toHaveCount(4);
	await expect(sheet.locator('[data-tier]')).toHaveText('+50 coins!');
});
