import { expect, test } from '@playwright/test';
import { newPlayer, visit } from './helpers';

test('a guest plays at once, has no chat, then signs up and keeps it all', async ({ page }) => {
	await visit(page, '/register');
	await page.getByRole('button', { name: 'Play as a guest' }).click();
	await page.waitForURL('/');
	await page.locator('html[data-hydrated]').waitFor();
	const banner = page.getByRole('complementary', { name: 'Playing as a guest' });
	await expect(banner).toBeVisible();
	await expect(page.getByTestId('coins')).toHaveText('500');

	// Plays like anyone: the pocket's tickets
	await page.getByRole('button', { name: 'Take', exact: true }).click();
	await expect(page.getByText('3 tickets to scratch')).toBeVisible();

	// No chat: no tab, the page sends back to the lobby, and no logging out
	const nav = page.getByRole('navigation');
	await expect(nav.getByRole('link', { name: /Chat/ })).toHaveCount(0);
	await expect(page.getByRole('button', { name: 'Log out' })).toHaveCount(0);
	await visit(page, '/chat');
	await expect(page).toHaveURL('/');
	expect((await page.request.get('/api/shoutbox')).status()).toBe(403);

	// Signing up keeps the same account
	await banner.getByRole('link', { name: 'Sign up' }).click();
	await expect(page.getByRole('heading', { name: 'Keep your progress' })).toBeVisible();
	const name = `k${Date.now().toString(36)}${Math.floor(Math.random() * 1e4)}`;
	await page.getByLabel('Username').fill('guest-me');
	await page.getByLabel('Email').fill(`${name}@example.com`);
	await page.getByLabel('Password').fill('123456789');
	await page.getByRole('button', { name: 'Keep my progress' }).click();
	await expect(page.getByRole('alert')).toContainText('are for guests');
	await page.getByLabel('Username').fill(name);
	await page.getByRole('button', { name: 'Keep my progress' }).click();

	await page.waitForURL('/');
	await expect(page.locator('.news .toast')).toContainText(`Welcome, ${name}!`);
	await expect(page.getByText('3 tickets to scratch')).toBeVisible();
	await expect(banner).toHaveCount(0);
	await expect(page.getByRole('button', { name: 'Log out' })).toBeVisible();

	// The chat is open now
	await nav.getByRole('link', { name: /Chat/ }).click();
	await expect(page.getByText('Live', { exact: true })).toBeVisible();
	const hello = `kept by ${name}`;
	await page.getByRole('textbox', { name: 'Say something nice' }).fill(hello);
	await page.getByRole('button', { name: 'Send' }).click();
	await expect(page.getByText(hello)).toBeVisible();
});

test('the login page offers to play as a guest too', async ({ page }) => {
	await visit(page, '/login');
	await page.getByRole('button', { name: 'Play as a guest' }).click();
	await page.waitForURL('/');
	await expect(page.getByRole('complementary', { name: 'Playing as a guest' })).toBeVisible();
});

test('guests and members offer each other no gifts or duels', async ({ page, browser }) => {
	const member = await newPlayer(page);
	const ctx = await browser.newContext({ baseURL: 'http://localhost:5174' });
	const g = await ctx.newPage();
	await visit(g, '/login');
	await g.getByRole('button', { name: 'Play as a guest' }).click();
	await g.waitForURL('/');
	const guest: string = (await (await g.request.get('/api/me')).json()).username;
	await expect(g.getByRole('region', { name: /Duels/ })).toHaveCount(0);

	// The guest looks at a member: nothing to send
	await visit(g, `/players/${member}`);
	await expect(g.getByRole('heading', { name: member })).toBeVisible();
	await expect(g.getByRole('button', { name: /Send a ticket|Challenge|Mute/ })).toHaveCount(0);

	// The member looks at the guest: nothing either
	await visit(page, `/players/${guest}`);
	await expect(page.getByRole('heading', { name: guest })).toBeVisible();
	await expect(page.getByRole('button', { name: /Send a ticket|Challenge|Mute/ })).toHaveCount(0);
	await ctx.close();
});
