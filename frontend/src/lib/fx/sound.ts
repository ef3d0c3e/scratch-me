/** What the scratching and the wins sound and feel like: numbers only, no Web Audio */

import type { Tier } from './celebrate.ts';

/** The scratch noise for a finger moving at `speed` px/ms: louder and brighter when fast */
export function scratchVoice(speed: number): { gain: number; freq: number } {
	const s = Math.min(1, Math.max(0, speed) / 1.5);
	return { gain: s === 0 ? 0 : 0.05 + 0.2 * s, freq: 1400 + 2600 * s };
}

export interface Note {
	/** Hz */
	f: number;
	/** Start, seconds from now */
	at: number;
	/** Length, seconds */
	len: number;
	vol: number;
	wave: OscillatorType;
}

const n = (
	f: number,
	at: number,
	len: number,
	vol = 0.18,
	wave: OscillatorType = 'triangle'
): Note => ({
	f,
	at,
	len,
	vol,
	wave
});

// C major, two octaves up from middle C
const C5 = 523.25;
const E5 = 659.25;
const G5 = 783.99;
const C6 = 1046.5;
const E6 = 1318.51;
const G6 = 1567.98;

/** The jingle for a prize: a shrug for nothing, a fanfare for the jackpot */
export function melody(tier: Tier): Note[] {
	switch (tier) {
		case 'none':
			return [n(392, 0, 0.18, 0.08, 'sine'), n(330, 0.16, 0.3, 0.08, 'sine')];
		case 'small':
			return [n(E5, 0, 0.12), n(C6, 0.1, 0.35)];
		case 'big':
			return [
				n(C5, 0, 0.1),
				n(E5, 0.09, 0.1),
				n(G5, 0.18, 0.1),
				n(C6, 0.27, 0.5),
				// Coins landing
				n(E6, 0.4, 0.08, 0.07, 'square'),
				n(G6, 0.5, 0.08, 0.07, 'square'),
				n(E6, 0.6, 0.08, 0.07, 'square')
			];
		case 'jackpot':
			return [
				n(C5, 0, 0.12),
				n(E5, 0.1, 0.12),
				n(G5, 0.2, 0.12),
				n(C6, 0.3, 0.12),
				n(G5, 0.42, 0.1),
				n(C6, 0.52, 0.9),
				// The chord under the last note
				n(E5, 0.52, 0.9, 0.1),
				n(G5, 0.52, 0.9, 0.1),
				// Sparkles
				...[0.8, 0.95, 1.1, 1.25, 1.4].map((t, i) => n(i % 2 ? G6 : E6, t, 0.07, 0.05, 'square'))
			];
	}
}

/** Vibration patterns (ms on, off, on…) for a prize */
export function vibration(tier: Tier): number[] {
	switch (tier) {
		case 'none':
			return [];
		case 'small':
			return [25];
		case 'big':
			return [30, 50, 30, 50, 60];
		case 'jackpot':
			return [60, 60, 60, 60, 60, 60, 220];
	}
}

/** Haptic ticks while scratching: at most one every this long, only when moving */
export const TICK_EVERY = 45;
export const tickDue = (now: number, last: number, speed: number) =>
	speed > 0.15 && now - last >= TICK_EVERY;
