import { describe, expect, it } from 'vitest';
import { TILT_RANGE, approach, drift, pointerOffset, tiltOffset } from './tilt.ts';

describe('tiltOffset', () => {
	const base = { beta: 40, gamma: 0 };
	it('is centred on the starting hold', () => {
		expect(tiltOffset(base, base)).toEqual({ x: 0, y: 0 });
	});
	it('moves with the tilt and stops at the ends', () => {
		expect(tiltOffset({ beta: 40, gamma: TILT_RANGE / 2 }, base)).toEqual({ x: 0.5, y: 0 });
		expect(tiltOffset({ beta: 40 - TILT_RANGE, gamma: 0 }, base)).toEqual({ x: 0, y: -1 });
		expect(tiltOffset({ beta: 170, gamma: -90 }, base)).toEqual({ x: -1, y: 1 });
	});
});

describe('pointerOffset', () => {
	const box = { left: 100, top: 100, width: 200, height: 100 };
	it('maps the box to -1…1', () => {
		expect(pointerOffset({ x: 200, y: 150 }, box)).toEqual({ x: 0, y: 0 });
		expect(pointerOffset({ x: 100, y: 100 }, box)).toEqual({ x: -1, y: -1 });
		expect(pointerOffset({ x: 900, y: 150 }, box)).toEqual({ x: 1, y: 0 });
	});
});

describe('drift', () => {
	it('stays inside the range and moves', () => {
		for (let t = 0; t < 20_000; t += 333) {
			const d = drift(t);
			expect(Math.abs(d.x)).toBeLessThanOrEqual(1);
			expect(Math.abs(d.y)).toBeLessThanOrEqual(1);
		}
		expect(drift(0)).not.toEqual(drift(1000));
	});
});

describe('approach', () => {
	it('closes in without overshooting', () => {
		let cur = { x: 0, y: 0 };
		const target = { x: 1, y: -1 };
		for (let i = 0; i < 10; i++) {
			const next = approach(cur, target, 16);
			expect(next.x).toBeGreaterThan(cur.x);
			expect(next.x).toBeLessThan(1);
			cur = next;
		}
		cur = approach(cur, target, 5000);
		expect(cur.x).toBeCloseTo(1);
		expect(cur.y).toBeCloseTo(-1);
	});
});
