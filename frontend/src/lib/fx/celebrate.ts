/** How loud a win is celebrated: the bigger the prize, the bigger the show */

import type { Prize } from '#lib/types.ts';

export type Tier = 'none' | 'small' | 'big' | 'jackpot';

/** Coin prizes from these amounts up get the bigger shows */
export const BIG_AT = 200;
export const JACKPOT_AT = 1000;

export function tier(prize: Prize): Tier {
	switch (prize.kind) {
		case 'nothing':
			return 'none';
		case 'coins':
			return prize.amount >= JACKPOT_AT ? 'jackpot' : prize.amount >= BIG_AT ? 'big' : 'small';
		// Not coins, but worth a moment: a cheer, not a shower
		case 'ticket':
		case 'pocket_refill':
			return 'small';
		// A Golden Ticket among them is worth more noise
		case 'tickets':
			return prize.series.includes('golden') ? 'big' : 'small';
		// Something to wear: always a big moment
		case 'cosmetic':
			return 'big';
		// As loud as the coins in it, and never quieter than a cheer
		case 'bundle': {
			const coins = prize.items.reduce((n, p) => n + (p.kind === 'coins' ? p.amount : 0), 0);
			return coins >= JACKPOT_AT ? 'jackpot' : coins >= BIG_AT ? 'big' : 'small';
		}
	}
}

export interface Show {
	/** Paper bits bursting from the ticket */
	confetti: number;
	/** Coins raining from the top of the screen */
	coins: number;
	/** Gems among them */
	gems: number;
	/** Waves the rain comes in, a beat apart */
	waves: number;
	shake: boolean;
	/** A full-screen moment over everything */
	takeover: boolean;
	/** How long the prize amount takes to count up, ms */
	count: number;
}

const SHOWS: Record<Tier, Show> = {
	none: { confetti: 0, coins: 0, gems: 0, waves: 0, shake: false, takeover: false, count: 0 },
	small: { confetti: 60, coins: 0, gems: 0, waves: 1, shake: false, takeover: false, count: 500 },
	big: { confetti: 90, coins: 40, gems: 0, waves: 2, shake: true, takeover: false, count: 900 },
	jackpot: {
		confetti: 150,
		coins: 80,
		gems: 30,
		waves: 4,
		shake: true,
		takeover: true,
		count: 2200
	}
};

export const show = (t: Tier): Show => SHOWS[t];

/** A number counting up from `from` to `to`: fast first, settling on the end */
export function countUp(from: number, to: number, elapsed: number, duration: number): number {
	if (duration <= 0 || elapsed >= duration) return to;
	const t = Math.max(0, elapsed) / duration;
	const eased = 1 - Math.pow(1 - t, 3);
	return Math.round(from + (to - from) * eased);
}
