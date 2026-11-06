import { expect, test } from '@playwright/test';
import { newPlayer, setCoins, sql, visit } from './helpers';

test('the leaderboard ranks by coins and by level, and shows your place', async ({ page }) => {
	const name = await newPlayer(page);
	// Top coin holder for a moment: above every other player on this database
	const {
		rows: [{ most }]
	} = await sql('SELECT COALESCE(MAX(coins), 0)::bigint AS most FROM users WHERE NOT is_guest');
	await setCoins(name, Number(most) + 1);

	await page
		.getByRole('navigation', { name: 'Sections' })
		.getByRole('link', { name: /Top/ })
		.click();
	await expect(page.getByRole('heading', { name: 'Leaderboard' })).toBeVisible();
	const coins = page.getByRole('list', { name: 'Coins' });
	// On the podium: first, or second when the other project's run went one higher
	const mine = coins.locator('li.mine');
	await expect(mine).toContainText(name);
	await expect(mine).toHaveAttribute('data-rank', /^[12]$/);
	await expect(mine).toContainText(/🥇|🥈/);
	await expect(coins.locator('li').first()).toHaveAttribute('data-rank', '1');

	// By level: a new player is far down, their place pinned under the top
	await page.getByRole('link', { name: '⭐ Level' }).click();
	await expect(page).toHaveURL(/by=level/);
	const levels = page.getByRole('list', { name: 'Level' });
	await expect(levels.locator('li').first()).toContainText('XP');
	const pinned = page.getByTestId('my-rank');
	if (await levels.getByText(name).count()) {
		await expect(levels.locator('li.mine')).toContainText(name);
	} else {
		await expect(pinned).toContainText(name);
		await expect(pinned).toContainText('Level 1');
	}

	// Back to coins, so the next tests are not outranked forever
	await setCoins(name, 500);
});

test('guests look at the board without a place on it', async ({ page }) => {
	await visit(page, '/login');
	await page.getByRole('button', { name: 'Play as a guest' }).click();
	await page.waitForURL('/');
	await visit(page, '/top');
	await expect(page.getByText('Guests are not ranked')).toBeVisible();
	await expect(page.getByTestId('my-rank')).toHaveCount(0);
	const me: string = (await (await page.request.get('/api/me')).json()).username;
	await expect(page.getByRole('list', { name: 'Coins' }).getByText(me)).toHaveCount(0);
});
