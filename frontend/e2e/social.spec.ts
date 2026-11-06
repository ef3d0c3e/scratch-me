import { expect, test, type Browser, type Page } from '@playwright/test';
import { GRIDS, giveTicket, newPlayer, sql, visit } from './helpers';

/** A second player, in a browser of their own */
async function other(browser: Browser) {
	const ctx = await browser.newContext({ baseURL: 'http://localhost:5174' });
	const page = await ctx.newPage();
	const name = await newPlayer(page);
	return { page, name, close: () => ctx.close() };
}

const live = (page: Page) => expect(page.getByText('Live', { exact: true })).toBeVisible();

test('a shout reaches the other player live, until they mute its author', async ({
	page,
	browser
}) => {
	const name = await newPlayer(page);
	const bob = await other(browser);
	await visit(bob.page, '/');
	await visit(page, '/chat');
	await live(page);
	// Other tests shout at the same time: a text of our own
	const hello = `hello from ${name}`;

	await page.getByRole('textbox', { name: 'Say something nice' }).fill(hello);
	await page.getByRole('button', { name: 'Send' }).click();
	await expect(page.getByText(hello)).toBeVisible();
	await expect(page.getByRole('textbox', { name: 'Say something nice' })).toHaveValue('');

	// Bob is on the lobby: an unread mark on the Chat tab, then the shout itself
	await expect(bob.page.getByTestId('unread')).toBeVisible();
	await bob.page.getByRole('link', { name: /Chat/ }).click();
	await expect(bob.page.getByText(hello)).toBeVisible();
	await expect(bob.page.getByTestId('unread')).toHaveCount(0);

	// From the name to the profile, and mute
	await bob.page.getByRole('link', { name, exact: true }).click();
	await expect(bob.page.getByRole('heading', { name })).toBeVisible();
	await bob.page.getByRole('button', { name: 'Mute in the shoutbox' }).click();
	await expect(bob.page.getByRole('button', { name: 'Unmute' })).toBeVisible();
	await visit(bob.page, '/chat');
	await live(bob.page);
	await expect(bob.page.getByText(hello)).toHaveCount(0);
	await bob.close();
});

test('shouting too fast is refused kindly', async ({ page }) => {
	await newPlayer(page);
	await visit(page, '/chat');
	const box = page.getByRole('textbox', { name: 'Say something nice' });
	await box.fill('one');
	await page.getByRole('button', { name: 'Send' }).click();
	await expect(box).toHaveValue('');
	await box.fill('two');
	await page.getByRole('button', { name: 'Send' }).click();
	await expect(page.getByRole('alert')).toHaveText('Easy! Wait a moment before the next one.');
	await expect(box).toHaveValue('two');
});

test('an admin announcement shows on everyone’s lobby, and can be closed', async ({
	page,
	browser
}) => {
	const name = await newPlayer(page);
	await sql('UPDATE users SET is_admin = true WHERE username = $1', [name]);
	const bob = await other(browser);
	await visit(bob.page, '/');
	await visit(page, '/chat');
	await live(page);
	const text = `Happy hour ${Date.now()}`;
	await page.getByRole('textbox', { name: /Announcement/ }).fill(text);
	await page.getByRole('button', { name: 'Announce' }).click();

	const notice = bob.page.getByRole('complementary', { name: 'Announcement' });
	await expect(notice).toContainText(text);
	await notice.getByRole('button', { name: 'Close' }).click();
	await expect(bob.page.getByText(text)).toHaveCount(0);
	// Closed stays closed in this browser
	await visit(bob.page, '/');
	await live(bob.page).catch(() => {});
	await bob.page.waitForTimeout(500);
	await expect(bob.page.getByText(text)).toHaveCount(0);
	await bob.close();
});

test('a jackpot reaches the feed, but not before its own player sees it', async ({
	page,
	browser
}) => {
	const name = await newPlayer(page);
	await giveTicket(name, { kind: 'coins', amount: 1000 }, GRIDS.gem, 'gem');
	const bob = await other(browser);
	await visit(bob.page, '/chat');
	await live(bob.page);
	await visit(page, '/');
	await page.getByRole('button', { name: 'Scratch', exact: true }).click();
	// Opening the sheet scratches it server side: Bob hears of it right away
	await expect(bob.page.getByText(`${name} won 1,000 coins on Lucky Three`)).toBeVisible();
	// The player's own ticker waits until the ticket is put away
	await expect(page.getByText(`${name} won`)).toHaveCount(0);
	await page.getByRole('dialog').getByRole('button', { name: 'Reveal all' }).click();
	await page.getByRole('button', { name: /JACKPOT!/ }).click();
	await page.getByRole('dialog').getByRole('button', { name: 'Back to the lobby' }).click();
	await expect(page.getByTestId('ticker')).toContainText(`${name} won 1,000 coins`);
	await bob.close();
});

test('a title earned shows on the profile and in the shoutbox', async ({ page }) => {
	const name = await newPlayer(page);
	await visit(page, `/players/${name}`);
	await page.getByLabel('Title shown next to your name').selectOption({ label: 'Jackpot!' });
	await expect(page.locator('.who .title')).toHaveText('Jackpot!');
	await visit(page, '/chat');
	await page.getByRole('textbox', { name: 'Say something nice' }).fill(`titled ${name}`);
	await page.getByRole('button', { name: 'Send' }).click();
	await expect(page.locator('li', { hasText: `titled ${name}` }).locator('.title')).toHaveText(
		'Jackpot!'
	);
});
