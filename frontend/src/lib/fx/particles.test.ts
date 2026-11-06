import { describe, expect, it } from 'vitest';
import { burst, rain, step } from './particles.ts';

/** Same numbers every run */
function seeded(seed = 1) {
	return () => {
		seed = (seed * 16807) % 2147483647;
		return (seed - 1) / 2147483646;
	};
}

describe('burst', () => {
	it('shoots confetti upward from the point', () => {
		const ps = burst(100, 200, 50, seeded());
		expect(ps).toHaveLength(50);
		expect(ps.every((p) => p.x === 100 && p.y === 200)).toBe(true);
		expect(ps.every((p) => p.vy < 0)).toBe(true);
	});
});

describe('rain', () => {
	it('starts above the screen, across its width, in waves', () => {
		const ps = rain(400, 'coin', 40, 2, seeded());
		expect(ps.every((p) => p.y < 0 && p.x >= 0 && p.x <= 400)).toBe(true);
		expect(Math.max(...ps.map((p) => p.delay))).toBeGreaterThan(450);
	});
});

describe('step', () => {
	it('falls under gravity and goes off the bottom', () => {
		const [p] = rain(400, 'coin', 1, 1, seeded());
		p.delay = 0;
		const y0 = p.y;
		expect(step(p, 16, 800)).toBe(true);
		expect(p.y).toBeGreaterThan(y0);
		let alive = true;
		for (let i = 0; i < 1000 && alive; i++) alive = step(p, 16, 800);
		expect(alive).toBe(false);
	});
	it('waits out its delay without moving', () => {
		const [p] = burst(10, 10, 1, seeded());
		p.delay = 100;
		step(p, 50, 800);
		expect([p.x, p.y]).toEqual([10, 10]);
	});
	it('confetti fades out even when it stays on screen', () => {
		const [p] = burst(10, 10, 1, seeded());
		p.delay = 0;
		let alive = true;
		let t = 0;
		for (; t < 10_000 && alive; t += 16) alive = step(p, 16, 1e9);
		expect(alive).toBe(false);
		expect(t).toBeLessThan(3000);
	});
});
