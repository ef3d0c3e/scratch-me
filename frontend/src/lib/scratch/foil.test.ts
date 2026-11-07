import { describe, expect, it } from 'vitest';
import { CELL_REVEAL_AT, Coverage, PEEK_AT, PEEK_INSET, brush, flakes, ready } from './foil.ts';

/** Horizontal strokes `gap` px apart over the rows from y0 to y1, x0 to x1 */
function rub(c: Coverage, x0: number, x1: number, y0: number, y1: number, size = 20, gap = 16) {
	for (let y = y0; y <= y1; y += gap) c.stroke(x0, y, x1, y, size);
}

describe('Coverage', () => {
	it('starts covered', () => {
		expect(new Coverage(300, 300).shares(3, 3)).toEqual(new Array(9).fill(0));
	});

	it('marks a stroke and only near it', () => {
		const c = new Coverage(300, 300);
		// Ends 10 px short of the next square: the round end of the brush stops at its edge
		c.stroke(0, 50, 90, 50, 20);
		const [first, second] = c.shares(3, 3);
		// A 20 px band across a 100 × 100 square
		expect(first).toBeGreaterThan(0.15);
		expect(first).toBeLessThan(0.25);
		expect(second).toBe(0);
	});

	it('a tap marks a dot', () => {
		const c = new Coverage(100, 100);
		c.stroke(50, 50, 50, 50, 20);
		const [s] = c.shares();
		// π × 10² out of 100²
		expect(s).toBeGreaterThan(0.02);
		expect(s).toBeLessThan(0.05);
	});

	it('ignores strokes off the foil', () => {
		const c = new Coverage(100, 100);
		c.stroke(-500, -500, -400, -400, 30);
		expect(c.shares()).toEqual([0]);
	});

	it('rubbing everything clears it all', () => {
		const c = new Coverage(300, 300);
		rub(c, 0, 300, 0, 300);
		expect(c.shares(3, 3).every((s) => s === 1)).toBe(true);
	});
});

describe('ready', () => {
	it('waits after half of the first square', () => {
		const c = new Coverage(300, 300);
		rub(c, 0, 100, 0, 50);
		expect(ready(c.seen(3, 3))).toBe(false);
	});

	it('waits while a third of the foil is scratched', () => {
		const c = new Coverage(300, 300);
		rub(c, 0, 100, 0, 300);
		expect(ready(c.seen(3, 3))).toBe(false);
	});

	it('waits while one square is still covered, however much the rest is', () => {
		const c = new Coverage(300, 300);
		rub(c, 0, 300, 0, 195);
		rub(c, 0, 195, 200, 300);
		expect(ready(c.seen(3, 3))).toBe(false);
	});

	it('goes once every square is mostly scratched', () => {
		const c = new Coverage(300, 300);
		// Strokes 28 px apart with a 20 px brush: about 70 % of everything
		rub(c, 0, 300, 10, 300, 20, 28);
		expect(Math.min(...c.shares(3, 3))).toBeLessThan(1);
		expect(ready(c.seen(3, 3))).toBe(true);
	});
});

describe('peeking', () => {
	it('a scratch across the middle of every square is enough to see them', () => {
		const c = new Coverage(300, 300);
		// One stroke through the middle of each row of squares, 40 px wide: a
		// band across each symbol, far from mostly scratching the squares
		for (const y of [40, 50, 60, 140, 150, 160, 240, 250, 260]) c.stroke(0, y, 300, y, 20);
		expect(Math.max(...c.shares(3, 3))).toBeLessThan(CELL_REVEAL_AT);
		expect(c.seen(3, 3).every(Boolean)).toBe(true);
		expect(ready(c.seen(3, 3))).toBe(true);
	});

	it('the edges of a square are not a peek at its symbol', () => {
		const c = new Coverage(300, 300);
		// Around the border of the first square only
		rub(c, 0, 100, 0, 15, 20, 15);
		rub(c, 0, 100, 85, 100, 20, 15);
		expect(c.shares(3, 3, PEEK_INSET)[0]).toBeLessThan(PEEK_AT);
		expect(c.seen(3, 3)[0]).toBe(false);
	});

	it('the middle counts on its own', () => {
		const c = new Coverage(100, 100);
		rub(c, 25, 75, 25, 75, 20, 10);
		expect(c.shares(1, 1, PEEK_INSET)[0]).toBeGreaterThan(0.8);
		expect(c.shares()[0]).toBeLessThan(CELL_REVEAL_AT);
		expect(c.seen()).toEqual([true]);
	});
});

describe('brush', () => {
	it('widens with speed, within bounds', () => {
		expect(brush(0)).toBe(20);
		expect(brush(1)).toBeGreaterThan(brush(0.2));
		expect(brush(50)).toBe(30);
		expect(brush(-1)).toBe(20);
	});
	it("takes the tool's range", () => {
		expect(brush(0, [12, 18])).toBe(12);
		expect(brush(9, [12, 18])).toBe(18);
	});
});

describe('flakes', () => {
	it('sheds more for longer strokes, capped', () => {
		expect(flakes(0)).toBe(0);
		expect(flakes(30)).toBe(5);
		expect(flakes(500)).toBe(8);
	});
});
