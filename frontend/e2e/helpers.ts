import { expect, type Page } from '@playwright/test';

/** Goes to a page and waits for hydration: clicks on server-rendered markup do nothing before */
export async function visit(page: Page, path: string) {
	await page.goto(path);
	await page.locator('html[data-hydrated]').waitFor();
}

/** Signs a new player up through the form and lands on the lobby */
export async function newPlayer(page: Page) {
	const name = `p${Date.now().toString(36)}${Math.floor(Math.random() * 1e4)}`;
	await visit(page, '/register');
	await page.getByLabel('Username').fill(name);
	await page.getByLabel('Email').fill(`${name}@example.com`);
	await page.getByLabel('Password').fill('123456789');
	await page.getByRole('button', { name: 'Start scratching' }).click();
	await page.waitForURL('/');
	await page.locator('html[data-hydrated]').waitFor();
	return name;
}

/** Drags across the foil row by row, like a finger, from `from` to `to` of its height */
export async function scratchAll(page: Page, from = 0, to = 1) {
	const foil = page.getByRole('dialog').locator('canvas.scratch');
	const box = (await foil.boundingBox())!;
	expect(box.width).toBeGreaterThan(100);
	for (let y = box.y + box.height * from + 10; y < box.y + box.height * to; y += 18) {
		await page.mouse.move(box.x + 5, y);
		await page.mouse.down();
		await page.mouse.move(box.x + box.width - 5, y, { steps: 6 });
		await page.mouse.up();
	}
}

/** Runs SQL on the dev database (see .env), for states the UI cannot reach quickly */
export async function sql(query: string, params: unknown[] = []) {
	const pg = await import('pg');
	const client = new pg.default.Client({
		connectionString:
			process.env.DATABASE_URL ?? 'postgres://scratchme:scratchme@127.0.0.1:5434/scratchme'
	});
	await client.connect();
	try {
		return await client.query(query, params);
	} finally {
		await client.end();
	}
}

type Cell = 'gem' | 'coin' | 'clover' | 'ticket' | 'hourglass' | 'star';

/** A Lucky Three with a known outcome, straight into the player's stack */
export async function giveTicket(username: string, prize: object, cells: Cell[], win: Cell | null) {
	await sql(
		`INSERT INTO tickets (user_id, series, layout, prize, source)
		 SELECT id, 'lucky_three', $2, $3, 'prize' FROM users WHERE username = $1`,
		[username, JSON.stringify({ cells, win }), JSON.stringify(prize)]
	);
}

export const GRIDS = {
	gem: ['gem', 'coin', 'gem', 'star', 'clover', 'coin', 'clover', 'gem', 'star'],
	coin: ['coin', 'star', 'coin', 'gem', 'clover', 'clover', 'star', 'coin', 'gem'],
	lose: ['gem', 'coin', 'clover', 'star', 'gem', 'ticket', 'hourglass', 'coin', 'star']
} satisfies Record<string, Cell[]>;

/** Any ticket with a known layout and prize, straight into the player's stack */
export async function giveRaw(username: string, series: string, layout: object, prize: object) {
	await sql(
		`INSERT INTO tickets (user_id, series, layout, prize, source)
		 SELECT id, $2, $3, $4, 'prize' FROM users WHERE username = $1`,
		[username, series, JSON.stringify(layout), JSON.stringify(prize)]
	);
}

export async function setCoins(username: string, coins: number) {
	await sql('UPDATE users SET coins = $2 WHERE username = $1', [username, coins]);
}
