import { describe, expect, it } from 'vitest';
import { TICK_EVERY, melody, scratchVoice, tickDue, vibration } from './sound.ts';

describe('scratchVoice', () => {
	it('is silent when still, louder and brighter when fast', () => {
		expect(scratchVoice(0).gain).toBe(0);
		expect(scratchVoice(1).gain).toBeGreaterThan(scratchVoice(0.2).gain);
		expect(scratchVoice(1).freq).toBeGreaterThan(scratchVoice(0.2).freq);
	});
	it('caps how loud it gets', () => {
		expect(scratchVoice(100).gain).toBeLessThanOrEqual(0.25);
		expect(scratchVoice(100)).toEqual(scratchVoice(1.5));
	});
});

describe('melody', () => {
	it('grows with the prize', () => {
		const end = (t: Parameters<typeof melody>[0]) =>
			Math.max(...melody(t).map((x) => x.at + x.len));
		expect(melody('small').length).toBeLessThan(melody('big').length);
		expect(melody('big').length).toBeLessThan(melody('jackpot').length);
		expect(end('jackpot')).toBeGreaterThan(end('big'));
	});
	it('keeps every note audible but gentle', () => {
		for (const t of ['none', 'small', 'big', 'jackpot'] as const)
			for (const x of melody(t)) {
				expect(x.vol).toBeGreaterThan(0);
				expect(x.vol).toBeLessThanOrEqual(0.2);
				expect(x.at).toBeGreaterThanOrEqual(0);
			}
	});
});

describe('vibration', () => {
	it('stays still on a loss and buzzes longer for bigger prizes', () => {
		const total = (t: Parameters<typeof vibration>[0]) => vibration(t).reduce((a, b) => a + b, 0);
		expect(vibration('none')).toEqual([]);
		expect(total('small')).toBeLessThan(total('big'));
		expect(total('big')).toBeLessThan(total('jackpot'));
	});
});

describe('tickDue', () => {
	it('ticks while moving, not faster than the limit', () => {
		expect(tickDue(1000, 0, 1)).toBe(true);
		expect(tickDue(1000, 1000 - TICK_EVERY + 1, 1)).toBe(false);
		expect(tickDue(1000, 0, 0)).toBe(false);
	});
});
