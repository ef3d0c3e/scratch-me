import { expect, type Page } from '@playwright/test';

/** Goes to a page and waits for hydration: clicks on server-rendered markup do nothing before */
export async function visit(page: Page, path: string) {
	await page.goto(path);
	await page.locator('html[data-hydrated]').waitFor();
}

/**
 * Signs a new player up through the form and lands on the lobby. Unless
 * `fresh`, their quests and achievements stay out of the way (no quest
 * today, every achievement earned), so tests can count coins.
 */
export async function newPlayer(page: Page, { fresh = false } = {}) {
	const name = `p${Date.now().toString(36)}${Math.floor(Math.random() * 1e4)}`;
	await visit(page, '/register');
	await page.getByLabel('Username').fill(name);
	await page.getByLabel('Email').fill(`${name}@example.com`);
	await page.getByLabel('Password').fill('123456789');
	await page.getByRole('button', { name: 'Start scratching' }).click();
	await page.waitForURL('/');
	await page.locator('html[data-hydrated]').waitFor();
	if (!fresh) {
		await quiet(page, name);
		await visit(page, '/');
	}
	return name;
}

async function quiet(page: Page, name: string) {
	const ids: { id: string }[] = await (await page.request.get('/api/achievements')).json();
	// The lobby has drawn today's quests already: a placeholder replaces them
	await sql('DELETE FROM quests WHERE user_id = (SELECT id FROM users WHERE username = $1)', [
		name
	]);
	await sql(
		`INSERT INTO quests (user_id, day, key)
		 SELECT id, (now() AT TIME ZONE tz)::date, 'none' FROM users WHERE username = $1`,
		[name]
	);
	await sql(
		`INSERT INTO achievements (user_id, id)
		 SELECT u.id, a FROM users u, unnest($2::text[]) a WHERE u.username = $1 ON CONFLICT DO NOTHING`,
		[name, ids.map((a) => a.id)]
	);
}

export async function setXp(username: string, xp: number) {
	await sql('UPDATE users SET xp = $2 WHERE username = $1', [username, xp]);
}

/** Experience at which a level starts: the server's curve (progress/levels.rs) */
export const xpFor = (level: number) => 50 * level * (level - 1);

/** Puts a player at the start of a level */
export const setLevel = (username: string, level: number) => setXp(username, xpFor(level));

/** Sets today's quests to these keys, with this much progress */
export async function setQuests(username: string, quests: [string, number][]) {
	await sql('DELETE FROM quests WHERE user_id = (SELECT id FROM users WHERE username = $1)', [
		username
	]);
	for (const [key, progress] of quests)
		await sql(
			`INSERT INTO quests (user_id, day, key, progress)
			 SELECT id, (now() AT TIME ZONE tz)::date, $2, $3 FROM users WHERE username = $1`,
			[username, key, progress]
		);
}

/** Drags across the foil row by row, like a finger, from `from` to `to` of its height */
export async function scratchAll(page: Page, from = 0, to = 1) {
	const foil = page.getByRole('dialog').locator('.play canvas.scratch');
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
