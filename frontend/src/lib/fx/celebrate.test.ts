import { describe, expect, it } from 'vitest';
import { countUp, show, tier } from './celebrate.ts';

describe('tier', () => {
	it('grows with the prize', () => {
		expect(tier({ kind: 'nothing' })).toBe('none');
		expect(tier({ kind: 'coins', amount: 50 })).toBe('small');
		expect(tier({ kind: 'coins', amount: 200 })).toBe('big');
		expect(tier({ kind: 'coins', amount: 1000 })).toBe('jackpot');
		expect(tier({ kind: 'ticket', series: 'lucky_three' })).toBe('small');
		expect(tier({ kind: 'pocket_refill' })).toBe('small');
		expect(tier({ kind: 'tickets', series: ['lucky_three', 'number_match'] })).toBe('small');
		expect(tier({ kind: 'tickets', series: ['lucky_three', 'golden'] })).toBe('big');
		expect(tier({ kind: 'cosmetic', item: { kind: 'tool', id: 'laser' } })).toBe('big');
	});
});

describe('show', () => {
	it('gets bigger tier by tier', () => {
		const [none, small, big, jackpot] = (['none', 'small', 'big', 'jackpot'] as const).map(show);
		expect(none.confetti + none.coins + none.gems).toBe(0);
		expect(small.confetti).toBeGreaterThan(0);
		expect(big.coins).toBeGreaterThan(small.coins);
		expect(jackpot.coins).toBeGreaterThan(big.coins);
		expect(jackpot.takeover && !big.takeover).toBe(true);
		expect(big.shake && !small.shake).toBe(true);
	});
});

describe('countUp', () => {
	it('starts at from and lands exactly on to', () => {
		expect(countUp(100, 300, 0, 1000)).toBe(100);
		expect(countUp(100, 300, 1000, 1000)).toBe(300);
		expect(countUp(100, 300, 5000, 1000)).toBe(300);
	});
	it('runs fast first', () => {
		expect(countUp(0, 1000, 500, 1000)).toBeGreaterThan(800);
	});
	it('only goes one way', () => {
		let prev = -1;
		for (let t = 0; t <= 1000; t += 50) {
			const v = countUp(0, 1000, t, 1000);
			expect(v).toBeGreaterThanOrEqual(prev);
			prev = v;
		}
	});
	it('jumps straight there without a duration', () => {
		expect(countUp(0, 50, 0, 0)).toBe(50);
	});
});
