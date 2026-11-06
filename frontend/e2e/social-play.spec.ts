import { expect, test, type Browser, type Page } from '@playwright/test';
import { newPlayer, setCoins, sql, visit } from './helpers';

/** A second player, in a browser of their own, on the lobby with the live socket up */
async function other(browser: Browser) {
	const ctx = await browser.newContext({ baseURL: 'http://localhost:5174' });
	const page = await ctx.newPage();
	const name = await newPlayer(page);
	await onLobby(page);
	return { page, name, close: () => ctx.close() };
}

async function onLobby(page: Page) {
	await visit(page, '/chat');
	await expect(page.getByText('Live', { exact: true })).toBeVisible();
	await page.getByRole('link', { name: /Lobby/ }).click();
	await page.waitForURL('/');
}

test('a gift lands in the other player’s stack, from its sender', async ({ page, browser }) => {
	const name = await newPlayer(page);
	const bob = await other(browser);
	await visit(page, `/players/${bob.name}`);
	await page.getByRole('button', { name: /Send a ticket/ }).click();
	await page.getByLabel('Ticket to send').selectOption('lucky_three');
	await page.getByRole('button', { name: 'Send · 35' }).click();
	await expect(page.getByRole('status')).toContainText(`Lucky Three sent to ${bob.name}`);
	await expect(page.getByTestId('coins')).toHaveText('465');

	// Bob hears of it at once, and it waits in his stack
	const toast = bob.page.locator('.news .toast');
	await expect(toast).toHaveAttribute('data-kind', 'gift');
	await expect(toast).toContainText(`A gift from ${name}`);
	await expect(bob.page.locator('.stub').last()).toContainText(`From ${name}`);
	await bob.close();
});

test('a Pot Ticket enters the shared pot and has an instant prize', async ({ page }) => {
	await newPlayer(page);
	const pot = page.getByRole('region', { name: 'Today’s pot' });
	await expect(pot).toContainText('Buy a Pot Ticket to enter');
	const before = Number((await page.getByTestId('pot-amount').innerText()).replace(/\D/g, ''));
	await pot.getByRole('button', { name: 'Enter · 50' }).click();
	await expect(pot).toContainText('You have 1 entry');
	await expect
		.poll(async () => Number((await page.getByTestId('pot-amount').innerText()).replace(/\D/g, '')))
		.toBeGreaterThanOrEqual(before + 20);
	await expect(page.locator('.stub').last()).toContainText('Pot Ticket');

	// Its face: the instant prize under the foil
	await page.getByRole('button', { name: 'Scratch', exact: true }).click();
	const sheet = page.getByRole('dialog');
	await expect(sheet).toContainText('You are in today’s pot');
	await sheet.getByRole('button', { name: 'Reveal all' }).click();
	await expect(sheet.getByTestId('instant')).toBeVisible();
});

test('a duel: challenged, accepted, played out on both sides', async ({ page, browser }) => {
	const name = await newPlayer(page);
	const bob = await other(browser);
	await visit(page, `/players/${bob.name}`);
	await page.getByRole('button', { name: /Challenge to a duel/ }).click();
	await expect(page.getByRole('status')).toContainText(`Challenge sent to ${bob.name}`);
	await expect(page.getByTestId('coins')).toHaveText('400');

	// Bob is told, and answers from the lobby
	await expect(bob.page.locator('.news .toast')).toContainText(`${name} challenges you`);
	const duels = bob.page.getByRole('region', { name: /Duels/ });
	await expect(duels).toContainText(`${name} challenges you · 100 coins`);
	await duels.getByRole('button', { name: 'Accept' }).click();

	// The duel ticket opens at once: both rows of dice, and who won
	const sheet = bob.page.getByRole('dialog');
	await expect(sheet).toContainText('Three dice each');
	await sheet.getByRole('button', { name: 'Reveal all' }).click();
	await expect(sheet.getByText(/You win the duel!|wins the duel/)).toBeVisible();
	const {
		rows: [row]
	} = await sql(
		`SELECT u.username AS winner FROM duels d JOIN users u ON u.id = d.winner_id
		 WHERE d.opponent_id = (SELECT id FROM users WHERE username = $1)`,
		[bob.name]
	);
	await expect(sheet).toContainText(
		row.winner === bob.name ? 'You win the duel!' : `${name} wins the duel`
	);

	// The challenger sees it played, and has a ticket of their own
	await visit(page, '/');
	await expect(page.getByRole('region', { name: /Duels/ })).toContainText(
		`${row.winner === name ? 'You' : bob.name} won`
	);
	await expect(page.locator('.stub').last()).toContainText('Duel');
	await bob.close();
});

test('a challenge can be turned down, and its stake comes back', async ({ page, browser }) => {
	await newPlayer(page);
	const bob = await other(browser);
	await setCoins(bob.name, 500);
	await visit(page, `/players/${bob.name}`);
	await page.getByRole('button', { name: /Challenge to a duel/ }).click();
	await expect(page.getByTestId('coins')).toHaveText('400');
	const duels = bob.page.getByRole('region', { name: /Duels/ });
	await duels.getByRole('button', { name: 'Decline' }).click();
	await expect(duels).toContainText('Turned down');
	// The challenger hears it live, and is paid back
	await visit(page, '/');
	await expect(page.getByTestId('coins')).toHaveText('500');
	await expect(page.getByRole('region', { name: /Duels/ })).toContainText('Turned down');
	await bob.close();
});
