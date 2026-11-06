import { expect, test } from '@playwright/test';
import { GRIDS, giveTicket, newPlayer, setQuests, setXp, sql, visit } from './helpers';

test('a first win earns achievements and a level, announced', async ({ page }) => {
	const name = await newPlayer(page, { fresh: true });
	await setQuests(name, []);
	await setXp(name, 95);
	await giveTicket(name, { kind: 'coins', amount: 200 }, GRIDS.coin, 'coin');
	await visit(page, '/');
	await expect(page.getByTestId('level')).toHaveAttribute('data-level', '1');
	await page.getByRole('button', { name: 'Scratch', exact: true }).click();
	await page.getByRole('dialog').getByRole('button', { name: 'Reveal all' }).click();

	// One at a time: the level first, then the achievements
	const toast = page.locator('.news .toast');
	await expect(toast).toHaveAttribute('data-kind', 'level');
	await expect(toast).toContainText('Level 2!');
	await expect(toast).toContainText('Number Match');
	await toast.click();
	await expect(toast).toContainText('First scratch');
	await toast.click();
	await expect(toast).toContainText('Beginner’s luck');
	await toast.click();
	await expect(toast).toContainText('Big win');
	await expect(page.getByTestId('level')).toHaveAttribute('data-level', '2');

	// The level's ticket waits in the stack (opening it would scratch it)
	await page.getByRole('dialog').getByRole('button', { name: 'Back to the lobby' }).click();
	await expect(page.getByText('1 ticket to scratch')).toBeVisible();
	await expect(page.locator('.stub')).toContainText('Number Match');
	await expect(page.locator('.stub')).toContainText('Reward');

	await visit(page, '/trophies');
	await expect(page.getByText(/^3 of \d+ earned$/)).toBeVisible();
	await expect(page.locator('li[data-achievement="big_win"]')).toContainText('🏆');
	await expect(page.locator('li[data-achievement="scratch_100"]')).toContainText('1 / 100');
});

test('quests move as you play, pay once, and one can be swapped', async ({ page }) => {
	const name = await newPlayer(page);
	await setQuests(name, [
		['pocket', 0],
		['scratch_5', 0],
		['win_3', 0]
	]);
	await visit(page, '/');
	const quests = page.getByRole('region', { name: 'Daily quests' });
	await expect(quests.locator('li')).toHaveCount(3);
	await expect(quests).toContainText('0/3');

	await page.getByRole('button', { name: 'Take', exact: true }).click();
	await expect(page.locator('.news .toast')).toContainText('Take your pocket');
	await expect(page.getByTestId('coins')).toHaveText('550');
	await expect(quests.locator('li[data-quest="pocket"]')).toContainText('✅');
	await expect(quests).toContainText('1/3');

	await quests.getByRole('button', { name: 'Swap: Win 3 times' }).click();
	await expect(quests.locator('li[data-quest="win_3"]')).toHaveCount(0);
	await expect(quests.locator('li')).toHaveCount(3);
	// One swap a day
	await expect(quests.getByRole('button', { name: /Swap/ })).toHaveCount(0);
});

test('finishing the three opens three harder ones, round by round', async ({ page }) => {
	const name = await newPlayer(page);
	await setQuests(name, [
		['pocket', 0],
		['scratch_5', 5],
		['win_3', 3]
	]);
	await sql(
		`UPDATE quests SET done_at = now()
		 WHERE key <> 'pocket' AND user_id = (SELECT id FROM users WHERE username = $1)`,
		[name]
	);
	await visit(page, '/');
	const quests = page.getByRole('region', { name: 'Daily quests' });
	await expect(quests).toContainText('Round 1 of 4');
	await expect(quests).toContainText('Then three harder quests');

	await page.getByRole('button', { name: 'Take', exact: true }).click();
	const toast = page.locator('.news .toast');
	await expect(toast).toContainText('Take your pocket');
	await toast.click();
	await expect(toast).toContainText('Three harder quests are up');
	await expect(quests).toContainText('Round 2 of 4');
	await expect(quests).toContainText('0/3');
	// Half as much again: the envelope does not come back
	await expect(quests.locator('li[data-quest="envelope"]')).toHaveCount(0);
	const goals: number[] = (await (await page.request.get('/api/quests')).json()).items.map(
		(q: { goal: number }) => q.goal
	);
	expect(goals.every((g) => g >= 2)).toBe(true);
});
